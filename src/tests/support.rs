use std::{
    collections::BTreeMap,
    env,
    path::PathBuf,
    sync::{Mutex, OnceLock},
};

use crate::{cli, config, container, error::AppError};

pub struct TestInput<'a> {
    pub toml: &'a str,
    pub args: &'a [&'a str],
    pub env: &'a [(&'a str, &'a str)],
    pub cwd_name: &'a str,
    pub cwd_entries: &'a [&'a str],
    pub fs_entries: &'a [(&'a str, Option<&'a str>)],
}

#[derive(Debug)]
pub struct TestOutput {
    pub command: String,
    pub cwd: PathBuf,
    pub home: PathBuf,
    pub root: PathBuf,
}

#[derive(Debug)]
pub struct ResolvedTestOutput {
    pub resolved: config::ResolvedConfig,
}

pub fn assert_command(input: TestInput<'_>, expected: &str) {
    let output = run_input(input);
    let normalized = normalize_command(&output.command, &output.cwd, &output.home, &output.root);
    assert_eq!(normalized, expected);
}

pub fn run_input(input: TestInput<'_>) -> TestOutput {
    try_run_input(input).expect("run input")
}

pub fn resolve_input(input: TestInput<'_>) -> ResolvedTestOutput {
    try_resolve_input(input).expect("resolve input")
}

pub fn try_run_input(input: TestInput<'_>) -> Result<TestOutput, AppError> {
    let _guard = acquire_test_lock();
    let temp_dir = tempfile::tempdir().expect("tempdir");
    let cwd = temp_dir.path().join(input.cwd_name);
    let home = temp_dir.path().join("home");
    let config_home = temp_dir.path().join("config");
    let config_path = config_home.join("dungeon").join("config.toml");

    std::fs::create_dir_all(&cwd).expect("create cwd");
    std::fs::create_dir_all(&home).expect("create home");
    std::fs::create_dir_all(config_path.parent().expect("config parent")).expect("config parent");

    create_cwd_entries(&cwd, input.cwd_entries).expect("create entries");
    create_fs_entries(temp_dir.path(), input.fs_entries).expect("create fs entries");

    let _env_guard = EnvGuard::new(&home, &config_home, input.env);

    if !input.toml.trim().is_empty() {
        std::fs::write(&config_path, input.toml).expect("write config");
    }

    let command = with_cwd(&cwd, || build_command_string(input))?;

    Ok(TestOutput {
        command,
        cwd,
        home,
        root: temp_dir.keep(),
    })
}

pub fn try_resolve_input(input: TestInput<'_>) -> Result<ResolvedTestOutput, AppError> {
    let _guard = acquire_test_lock();
    let temp_dir = tempfile::tempdir().expect("tempdir");
    let cwd = temp_dir.path().join(input.cwd_name);
    let home = temp_dir.path().join("home");
    let config_home = temp_dir.path().join("config");
    let config_path = config_home.join("dungeon").join("config.toml");

    std::fs::create_dir_all(&cwd).expect("create cwd");
    std::fs::create_dir_all(&home).expect("create home");
    std::fs::create_dir_all(config_path.parent().expect("config parent")).expect("config parent");

    create_cwd_entries(&cwd, input.cwd_entries).expect("create entries");
    create_fs_entries(temp_dir.path(), input.fs_entries).expect("create fs entries");

    let _env_guard = EnvGuard::new(&home, &config_home, input.env);

    if !input.toml.trim().is_empty() {
        std::fs::write(&config_path, input.toml).expect("write config");
    }

    let resolved = with_cwd(&cwd, || resolve_settings(input))?;

    Ok(ResolvedTestOutput { resolved })
}

fn build_command_string(input: TestInput<'_>) -> Result<String, AppError> {
    let resolved = resolve_settings(input)?;

    let spec = container::engine::build_container_command(
        &resolved.settings,
        &resolved.paths,
        resolved.persist_mode == container::persist::PersistMode::Create,
        if resolved.persist_mode == container::persist::PersistMode::Create {
            Some(resolved.container_name.as_str())
        } else {
            None
        },
        resolved.skip_cwd,
    )?;

    Ok(format!("{} {}", spec.program, spec.args.join(" ")))
}

fn resolve_settings(input: TestInput<'_>) -> Result<config::ResolvedConfig, AppError> {
    let sources = config::load_sources()?;

    let argv = input.args.iter().map(|arg| arg.to_string()).collect();
    let parsed =
        cli::parse_args_with_sources(argv, &sources.defaults, &sources.file, &sources.env)?;
    config::resolve(&parsed, &sources)
}

pub fn acquire_test_lock() -> std::sync::MutexGuard<'static, ()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
        .lock()
        .unwrap_or_else(|err| err.into_inner())
}

fn normalize_command(command: &str, cwd: &PathBuf, home: &PathBuf, root: &PathBuf) -> String {
    let normalized = command
        .replace(cwd.to_string_lossy().as_ref(), "<CWD>")
        .replace(home.to_string_lossy().as_ref(), "<HOME>")
        .replace(root.to_string_lossy().as_ref(), "<TMP>")
        .replace("--user root ", "")
        .replace("--cap-add NET_ADMIN ", "")
        .replace("--cap-add NET_RAW ", "")
        .replace("--cap-add SYS_ADMIN ", "")
        .replace("--cap-add SYS_CHROOT ", "")
        .replace("--cap-add SETUID ", "")
        .replace("--cap-add SETGID ", "")
        .replace("--cap-add SYS_PTRACE ", "")
        .replace("--security-opt seccomp=unconfined ", "");

    let (uid, gid) = host_uid_gid();
    normalized.replace(&format!("--user {}:{}", uid, gid), "--user <UID>:<GID>")
}

fn host_uid_gid() -> (u32, u32) {
    #[cfg(unix)]
    {
        let uid = unsafe { libc::geteuid() };
        let gid = unsafe { libc::getegid() };
        (uid, gid)
    }

    #[cfg(not(unix))]
    {
        (1000, 1000)
    }
}

fn create_cwd_entries(cwd: &PathBuf, entries: &[&str]) -> Result<(), std::io::Error> {
    for entry in entries {
        let trimmed = entry.trim();
        if trimmed.is_empty() {
            continue;
        }
        if trimmed.ends_with('/') {
            let dir = cwd.join(trimmed.trim_end_matches('/'));
            std::fs::create_dir_all(dir)?;
        } else {
            let file = cwd.join(trimmed);
            if let Some(parent) = file.parent() {
                std::fs::create_dir_all(parent)?;
            }
            if file.exists() {
                continue;
            }
            if trimmed.contains('.') {
                std::fs::write(file, "test")?;
            } else {
                std::fs::create_dir_all(file)?;
            }
        }
    }
    Ok(())
}

fn create_fs_entries(
    root: &std::path::Path,
    entries: &[(&str, Option<&str>)],
) -> Result<(), std::io::Error> {
    for (path, contents) in entries {
        let full_path = root.join(path);
        match contents {
            Some(contents) => {
                if let Some(parent) = full_path.parent() {
                    std::fs::create_dir_all(parent)?;
                }
                let rendered = contents.replace("<TMP>", root.to_string_lossy().as_ref());
                std::fs::write(full_path, rendered)?;
            }
            None => std::fs::create_dir_all(full_path)?,
        }
    }
    Ok(())
}

fn with_cwd<T>(cwd: &PathBuf, f: impl FnOnce() -> Result<T, AppError>) -> Result<T, AppError> {
    let original = env::current_dir().map_err(AppError::Io)?;
    env::set_current_dir(cwd).map_err(AppError::Io)?;
    let result = f();
    env::set_current_dir(original).map_err(AppError::Io)?;
    result
}

struct EnvGuard {
    previous: BTreeMap<String, Option<String>>,
}

impl EnvGuard {
    fn new(home: &PathBuf, config_home: &PathBuf, vars: &[(&str, &str)]) -> Self {
        let mut previous = BTreeMap::new();
        let mut set_var = |key: &str, value: Option<&str>| {
            previous.insert(key.to_string(), env::var(key).ok());
            unsafe {
                if let Some(value) = value {
                    env::set_var(key, value);
                } else {
                    env::remove_var(key);
                }
            }
        };

        set_var("HOME", Some(home.to_string_lossy().as_ref()));
        set_var(
            "XDG_CONFIG_HOME",
            Some(config_home.to_string_lossy().as_ref()),
        );

        for key in DUNGEON_ENV_KEYS {
            set_var(key, None);
        }

        for (key, value) in vars {
            set_var(key, Some(value));
        }

        Self { previous }
    }
}

impl Drop for EnvGuard {
    fn drop(&mut self) {
        for (key, value) in self.previous.clone() {
            unsafe {
                if let Some(value) = value {
                    env::set_var(key, value);
                } else {
                    env::remove_var(key);
                }
            }
        }
    }
}

const DUNGEON_ENV_KEYS: &[&str] = &[
    "DUNGEON_ENGINE",
    "DUNGEON_COMMAND",
    "DUNGEON_IMAGE",
    "DUNGEON_PORTS",
    "DUNGEON_CACHES",
    "DUNGEON_MOUNTS",
    "DUNGEON_ENVS",
    "DUNGEON_ENV_FILES",
    "DUNGEON_PODMAN_ARGS",
    "DUNGEON_RUN_ARGS",
    "DUNGEON_MOUNT_GIT_METADATA",
    "DUNGEON_IPV6",
    "DUNGEON_ALLOW_DNS",
    "DUNGEON_ALLOWED_TCP_DOMAINS",
    "DUNGEON_ALLOWED_TCP_HOSTS",
    "DUNGEON_ALWAYS_ON_GROUPS",
];
