use std::{
    collections::HashSet,
    net::TcpListener,
    path::{Component, Path, PathBuf},
};

use crate::{
    config::{Engine, Settings},
    error::AppError,
};

const WORKSPACE_ROOT: &str = "/workspace";
const DEFAULT_NETWORK_IPV6: bool = false;
const DEFAULT_NETWORK_ALLOW_DNS: bool = true;

#[derive(Debug, Clone)]
pub struct CommandSpec {
    pub program: String,
    pub args: Vec<String>,
}

#[derive(Default)]
pub struct DynamicPortReservations {
    listeners: Vec<TcpListener>,
}

pub fn reserve_dynamic_ports(settings: &mut Settings) -> Result<DynamicPortReservations, AppError> {
    let mut reservations = DynamicPortReservations::default();
    let mut names = HashSet::new();

    for name in settings.dynamic_ports.as_deref().unwrap_or(&[]) {
        if !names.insert(name) {
            continue;
        }
        let listener = TcpListener::bind("127.0.0.1:0")?;
        let port = listener.local_addr()?.port();
        let env_key = format!("DUNGEON_PORT_FOR_{}", name.to_ascii_uppercase());
        settings
            .ports
            .get_or_insert_with(Vec::new)
            .push(format!("127.0.0.1:{0}:{0}", port));
        let env_vars = settings.env_vars.get_or_insert_with(Vec::new);
        env_vars.retain(|spec| !env_spec_has_name(spec, &env_key));
        env_vars.push(format!("{}={}", env_key, port));
        reservations.listeners.push(listener);
    }

    Ok(reservations)
}

pub fn run_reserved_container_command(
    spec: CommandSpec,
    reservations: DynamicPortReservations,
) -> Result<(), AppError> {
    drop(reservations);
    run_container_command(spec)
}

pub fn build_podman_command(settings: &Settings, args: Vec<String>) -> CommandSpec {
    let engine = settings.engine.unwrap_or_default();
    let mut full_args = settings.podman_args.clone().unwrap_or_default();
    full_args.extend(args);

    CommandSpec {
        program: engine.binary().to_string(),
        args: full_args,
    }
}

pub fn build_cache_reset_command(settings: &Settings) -> CommandSpec {
    build_podman_command(
        settings,
        vec![
            "volume".to_string(),
            "rm".to_string(),
            "-f".to_string(),
            "dungeon-cache".to_string(),
        ],
    )
}

pub fn reset_cache_volume(settings: &Settings) -> Result<(), AppError> {
    run_container_command(build_cache_reset_command(settings))
}

pub fn build_image_command(
    settings: &Settings,
    tag: &str,
    no_cache: bool,
    context: &str,
) -> CommandSpec {
    let mut args = vec![
        "build".to_string(),
        "-f".to_string(),
        "images/Containerfile".to_string(),
        "-t".to_string(),
        tag.to_string(),
    ];
    if no_cache {
        args.push("--no-cache".to_string());
    }
    args.push(context.to_string());

    build_podman_command(settings, args)
}

pub fn build_container_command(
    settings: &Settings,
    paths: &[String],
    skip_cwd: bool,
) -> Result<CommandSpec, AppError> {
    let cwd = std::env::current_dir()?;
    let home =
        dirs::home_dir().ok_or_else(|| AppError::message("unable to resolve home directory"))?;
    let engine = settings.engine.unwrap_or_default();
    let (workdir, mounts) = resolve_workdir_and_mounts(settings, paths, skip_cwd, &cwd, &home)?;

    let mut args = vec!["run".to_string(), "-it".to_string()];
    append_engine_security_args(&mut args);
    append_engine_identity_args(&mut args, engine);
    args.push("-w".to_string());
    args.push(workdir);

    args.push("--rm".to_string());

    append_env_args(&mut args, settings.env_vars.as_deref().unwrap_or(&[]));
    append_network_env_args(&mut args, settings);
    append_repeated_flag_args(
        &mut args,
        "--env-file",
        settings.env_files.as_deref().unwrap_or(&[]),
    );
    append_repeated_flag_args(&mut args, "-p", settings.ports.as_deref().unwrap_or(&[]));

    if let Some(args_list) = settings.run_args.as_deref() {
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

    Ok(build_podman_command(settings, args))
}

fn append_engine_security_args(args: &mut Vec<String>) {
    args.push("--user".to_string());
    args.push("root".to_string());
    args.push("--cap-add".to_string());
    args.push("NET_ADMIN".to_string());
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
    let mut workspace_dirs = Vec::new();

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
            return Ok((WORKSPACE_ROOT.to_string(), mounts));
        }

        let base = cwd
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("project");
        let workdir = format!("{}/{}", WORKSPACE_ROOT, base);
        push_mount(&mut mounts, format!("{}:{}", cwd.display(), workdir));
        workspace_dirs.push(cwd.to_path_buf());
        append_git_metadata_mounts(&mut mounts, settings, &workspace_dirs)?;
        return Ok((workdir, mounts));
    }

    let workdir = format!("{}/project", WORKSPACE_ROOT);
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
        if abs.is_dir() {
            workspace_dirs.push(abs);
        }
    }
    append_git_metadata_mounts(&mut mounts, settings, &workspace_dirs)?;
    Ok((workdir, mounts))
}

fn append_git_metadata_mounts(
    mounts: &mut Vec<String>,
    settings: &Settings,
    workspace_dirs: &[PathBuf],
) -> Result<(), AppError> {
    if settings.mount_git_metadata != Some(true) {
        return Ok(());
    }

    let mut generated = HashSet::new();
    for workspace_dir in workspace_dirs {
        let Some(source) = resolve_git_metadata_mount_source(workspace_dir)? else {
            continue;
        };
        let spec = format!("{}:{}", source.display(), source.display());
        if generated.insert(spec.clone()) {
            push_mount(mounts, spec);
        }
    }

    Ok(())
}

fn resolve_git_metadata_mount_source(workspace_dir: &Path) -> Result<Option<PathBuf>, AppError> {
    let git_entry = workspace_dir.join(".git");
    if !git_entry.exists() {
        return Ok(None);
    }
    if git_entry.is_dir() {
        return Ok(None);
    }
    if !git_entry.is_file() {
        return Err(AppError::message(format!(
            "ERROR: unsupported git metadata entry {}",
            git_entry.display()
        )));
    }

    let git_dir = parse_gitdir_file(&git_entry)?;
    let mount_source = resolve_git_mount_source(&git_dir)?;
    Ok(Some(mount_source))
}

fn parse_gitdir_file(git_file: &Path) -> Result<PathBuf, AppError> {
    let raw = std::fs::read_to_string(git_file).map_err(|err| {
        AppError::message(format!("read git metadata {}: {}", git_file.display(), err))
    })?;
    let value = raw.trim();
    let Some(path) = value.strip_prefix("gitdir:") else {
        return Err(AppError::message(format!(
            "ERROR: malformed git metadata file {}",
            git_file.display()
        )));
    };
    let git_dir = PathBuf::from(path.trim());
    if !git_dir.is_absolute() {
        return Err(AppError::message(format!(
            "ERROR: relative gitdir paths are unsupported in {}",
            git_file.display()
        )));
    }

    let git_dir = normalize_absolute_path(&git_dir);
    if !git_dir.exists() {
        return Err(AppError::message(format!(
            "ERROR: gitdir path does not exist: {}",
            git_dir.display()
        )));
    }
    Ok(git_dir)
}

fn resolve_git_mount_source(git_dir: &Path) -> Result<PathBuf, AppError> {
    let common_dir_file = git_dir.join("commondir");
    if !common_dir_file.exists() {
        return Ok(git_dir.to_path_buf());
    }
    if !common_dir_file.is_file() {
        return Err(AppError::message(format!(
            "ERROR: malformed git commondir file {}",
            common_dir_file.display()
        )));
    }

    let raw = std::fs::read_to_string(&common_dir_file).map_err(|err| {
        AppError::message(format!(
            "read git metadata {}: {}",
            common_dir_file.display(),
            err
        ))
    })?;
    let value = raw.trim();
    if value.is_empty() {
        return Err(AppError::message(format!(
            "ERROR: malformed git commondir file {}",
            common_dir_file.display()
        )));
    }

    let common_dir = if Path::new(value).is_absolute() {
        normalize_absolute_path(Path::new(value))
    } else {
        normalize_absolute_path(&git_dir.join(value))
    };
    if !common_dir.exists() {
        return Err(AppError::message(format!(
            "ERROR: git commondir path does not exist: {}",
            common_dir.display()
        )));
    }

    Ok(common_dir)
}

fn append_engine_identity_args(args: &mut Vec<String>, engine: Engine) {
    match engine {
        Engine::Podman => args.push("--userns=keep-id".to_string()),
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

fn env_spec_has_name(spec: &str, name: &str) -> bool {
    let trimmed = spec.trim();
    let key = trimmed
        .split_once('=')
        .map(|(key, _)| key)
        .unwrap_or(trimmed);
    key == name
}

fn append_network_env_args(args: &mut Vec<String>, settings: &Settings) {
    if let Some(ipv6) = settings.ipv6
        && ipv6 != DEFAULT_NETWORK_IPV6
    {
        push_env_arg(args, "DUNGEON_IPV6", if ipv6 { "1" } else { "0" });
    }
    if let Some(allow_dns) = settings.allow_dns
        && allow_dns != DEFAULT_NETWORK_ALLOW_DNS
    {
        push_env_arg(args, "DUNGEON_ALLOW_DNS", if allow_dns { "1" } else { "0" });
    }
    if let Some(domains) = settings
        .allowed_tcp_domains
        .as_ref()
        .filter(|values| !values.is_empty())
    {
        push_env_arg(args, "DUNGEON_ALLOWED_TCP_DOMAINS", &domains.join(","));
    }
    if let Some(hosts) = settings
        .allowed_tcp_hosts
        .as_ref()
        .filter(|values| !values.is_empty())
    {
        push_env_arg(args, "DUNGEON_ALLOWED_TCP_HOSTS", &hosts.join(","));
    }
}

fn push_env_arg(args: &mut Vec<String>, key: &str, value: &str) {
    args.push("--env".to_string());
    args.push(format!("{}={}", key, value));
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
    args.push("zsh".to_string());
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

fn normalize_absolute_path(path: &Path) -> PathBuf {
    let mut normalized = PathBuf::new();
    for component in path.components() {
        match component {
            Component::Prefix(prefix) => normalized.push(prefix.as_os_str()),
            Component::RootDir => normalized.push(component.as_os_str()),
            Component::CurDir => {}
            Component::ParentDir => {
                normalized.pop();
            }
            Component::Normal(part) => normalized.push(part),
        }
    }
    normalized
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
