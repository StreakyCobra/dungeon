use std::path::{Path, PathBuf};

use crate::{
    config::{Engine, Settings},
    error::AppError,
};

const USER_HOME: &str = "/home/dungeon";

#[derive(Debug, Clone)]
pub struct CommandSpec {
    pub program: String,
    pub args: Vec<String>,
}

pub fn build_cache_reset_command(engine: Engine) -> CommandSpec {
    CommandSpec {
        program: engine.binary().to_string(),
        args: vec![
            "volume".to_string(),
            "rm".to_string(),
            "-f".to_string(),
            "dungeon-cache".to_string(),
        ],
    }
}

pub fn reset_cache_volume(engine: Engine) -> Result<(), AppError> {
    run_container_command(build_cache_reset_command(engine))
}

pub fn build_image_command(
    engine: Engine,
    containerfile: &str,
    tag: &str,
    no_cache: bool,
    context: &str,
) -> CommandSpec {
    let mut args = vec![
        "build".to_string(),
        "-f".to_string(),
        containerfile.to_string(),
        "-t".to_string(),
        tag.to_string(),
    ];
    if no_cache {
        args.push("--no-cache".to_string());
    }
    args.push(context.to_string());

    CommandSpec {
        program: engine.binary().to_string(),
        args,
    }
}

pub fn build_container_command(
    settings: &Settings,
    paths: &[String],
    keep_container: bool,
    container_name: Option<&str>,
    skip_cwd: bool,
) -> Result<CommandSpec, AppError> {
    let cwd = std::env::current_dir()?;
    let home =
        dirs::home_dir().ok_or_else(|| AppError::message("unable to resolve home directory"))?;
    let engine = settings.engine.unwrap_or_default();

    let (workdir, mounts) = resolve_workdir_and_mounts(settings, paths, skip_cwd, &cwd, &home)?;

    let mut args = vec!["run".to_string(), "-it".to_string()];
    append_engine_identity_args(&mut args, engine);
    args.push("-w".to_string());
    args.push(workdir);

    if !keep_container {
        args.push("--rm".to_string());
    }
    if let Some(name) = container_name {
        if !name.trim().is_empty() {
            args.push("--name".to_string());
            args.push(name.to_string());
        }
    }

    append_env_args(&mut args, settings.env_vars.as_deref().unwrap_or(&[]));
    append_repeated_flag_args(
        &mut args,
        "--env-file",
        settings.env_files.as_deref().unwrap_or(&[]),
    );
    append_repeated_flag_args(&mut args, "-p", settings.ports.as_deref().unwrap_or(&[]));

    if let Some(args_list) = settings.engine_args.as_deref() {
        args.extend(args_list.iter().cloned());
    }

    args.extend(mounts);

    let image = settings
        .image
        .as_deref()
        .filter(|value| !value.trim().is_empty())
        .unwrap_or("localhost/dungeon");
    args.push(image.to_string());

    append_command(&mut args, settings.command.as_deref());

    Ok(CommandSpec {
        program: engine.binary().to_string(),
        args,
    })
}

pub fn run_container_command(spec: CommandSpec) -> Result<(), AppError> {
    crate::container::run_attached_command(&spec.program, &spec.args)
}

fn resolve_workdir_and_mounts(
    settings: &Settings,
    paths: &[String],
    skip_cwd: bool,
    cwd: &Path,
    home: &Path,
) -> Result<(String, Vec<String>), AppError> {
    let mut mounts = Vec::new();

    for spec in settings.mounts.as_deref().unwrap_or(&[]) {
        push_mount(&mut mounts, expand_mount_spec(spec, home));
    }
    for spec in settings.cache.as_deref().unwrap_or(&[]) {
        push_mount(&mut mounts, format!("dungeon-cache:{}", spec));
    }

    if paths.is_empty() {
        if !skip_cwd && same_dir(cwd, home) {
            return Err(AppError::message(
                "ERROR: refusing to run from home directory",
            ));
        }
        if skip_cwd {
            return Ok((USER_HOME.to_string(), mounts));
        }

        let base = cwd
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("project");
        let workdir = format!("{}/{}", USER_HOME, base);
        push_mount(&mut mounts, format!("{}:{}", cwd.display(), workdir));
        return Ok((workdir, mounts));
    }

    let workdir = format!("{}/project", USER_HOME);
    for path in paths {
        let abs = absolute_path(cwd, path);
        let base = abs
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("project");
        push_mount(
            &mut mounts,
            format!("{}:{}/{}", abs.display(), workdir, base),
        );
    }
    Ok((workdir, mounts))
}

fn append_engine_identity_args(args: &mut Vec<String>, engine: Engine) {
    match engine {
        Engine::Podman => args.push("--userns=keep-id".to_string()),
        Engine::Docker => {
            let (uid, gid) = host_uid_gid();
            args.push("--user".to_string());
            args.push(format!("{}:{}", uid, gid));
        }
    }
}

fn append_env_args(args: &mut Vec<String>, env_specs: &[String]) {
    for spec in env_specs {
        let trimmed = spec.trim();
        if trimmed.is_empty() {
            continue;
        }
        args.push("--env".to_string());
        args.push(trimmed.to_string());
    }
}

fn append_repeated_flag_args(args: &mut Vec<String>, flag: &str, values: &[String]) {
    for value in values {
        let trimmed = value.trim();
        if trimmed.is_empty() {
            continue;
        }
        args.push(flag.to_string());
        args.push(trimmed.to_string());
    }
}

fn append_command(args: &mut Vec<String>, command: Option<&str>) {
    args.push("bash".to_string());
    if let Some(command) = command
        && !command.trim().is_empty()
    {
        args.push("-ic".to_string());
        args.push(command.to_string());
    }
}

fn push_mount(mounts: &mut Vec<String>, spec: String) {
    mounts.push("-v".to_string());
    mounts.push(spec);
}

fn absolute_path(cwd: &Path, path: &str) -> PathBuf {
    let raw = PathBuf::from(path);
    if raw.is_absolute() {
        raw
    } else {
        cwd.join(raw)
    }
}

fn same_dir(a: &Path, b: &Path) -> bool {
    a.canonicalize().ok() == b.canonicalize().ok()
}

fn expand_mount_spec(spec: &str, home: &Path) -> String {
    let trimmed = spec.trim();
    if trimmed.is_empty() {
        return spec.to_string();
    }
    let (source, rest) = match trimmed.split_once(':') {
        Some((source, rest)) => (source, Some(rest)),
        None => (trimmed, None),
    };
    let expanded = expand_home_or_env(source, home);
    match rest {
        Some(remaining) => format!("{}:{}", expanded, remaining),
        None => expanded,
    }
}

fn expand_home_or_env(source: &str, home: &Path) -> String {
    if source == "~" || source.starts_with("~/") {
        let suffix = source.trim_start_matches('~');
        return format!("{}{}", home.display(), suffix);
    }
    if let Some(stripped) = source.strip_prefix("$HOME") {
        return format!("{}{}", home.display(), stripped);
    }
    source.to_string()
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
