use std::{
    path::{Path, PathBuf},
    process::{Command, Stdio},
};

use crate::{
    config::Settings,
    container::mounts::{
        container_path, parse_cache_mount_spec, parse_host_mount_spec, resolve_host_path,
    },
    error::AppError,
};

const USER_HOME: &str = "/home/dungeon";

#[derive(Debug, Clone)]
pub struct CommandSpec {
    pub program: String,
    pub args: Vec<String>,
}

pub fn reset_cache_volume() -> Result<(), AppError> {
    let mut cmd = Command::new("podman");
    cmd.arg("volume").arg("rm").arg("-f").arg("dungeon-cache");
    run_command(&mut cmd)
}

pub fn build_podman_command(
    settings: &Settings,
    paths: &[String],
    keep_container: bool,
    container_name: Option<&str>,
) -> Result<CommandSpec, AppError> {
    let cwd = std::env::current_dir()?;
    let home =
        dirs::home_dir().ok_or_else(|| AppError::message("unable to resolve home directory"))?;

    let mut mounts = vec![
        "-v".to_string(),
        format!("dungeon-cache:{}/.cache", USER_HOME),
        "-v".to_string(),
        format!("dungeon-cache:{}/.npm", USER_HOME),
    ];

    let cache_specs = settings.cache.clone().unwrap_or_default();
    let env_specs = settings.env_vars.clone().unwrap_or_default();
    let ports = settings.ports.clone().unwrap_or_default();
    let run_command = settings.run_command.clone().unwrap_or_default();
    let mut image = settings.image.clone().unwrap_or_default();

    for spec in settings.mounts.clone().unwrap_or_default() {
        let (source, target, mode) = parse_host_mount_spec(&spec)?;
        let host_path = resolve_host_path(&home, &source)?;
        if !host_path.exists() {
            return Err(AppError::message(format!(
                "ERROR: mount source '{}' does not exist",
                host_path.display()
            )));
        }
        let target_path = container_path(&target);
        mounts.push("-v".to_string());
        mounts.push(format!("{}:{}{}", host_path.display(), target_path, mode));
    }

    for spec in cache_specs {
        let (target, mode) = parse_cache_mount_spec(&spec)?;
        mounts.push("-v".to_string());
        mounts.push(format!("dungeon-cache:{}{}", target, mode));
    }

    let env_args = build_env_args(&env_specs)?;

    let workdir;
    if paths.is_empty() {
        if same_dir(&cwd, &home) {
            return Err(AppError::message(
                "ERROR: refusing to run from home directory",
            ));
        }
        let base = cwd
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("project");
        workdir = format!("{}/{}", USER_HOME, base);
        mounts.push("-v".to_string());
        mounts.push(format!("{}:{}", cwd.display(), workdir));
    } else {
        workdir = format!("{}/project", USER_HOME);
        for path in paths {
            let abs = PathBuf::from(path);
            if !abs.exists() {
                return Err(AppError::message(format!(
                    "ERROR: '{}' does not exist",
                    path
                )));
            }
            let abs = abs.canonicalize()?;
            let base = abs
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or("project");
            mounts.push("-v".to_string());
            mounts.push(format!("{}:{}/{}", abs.display(), workdir, base));
        }
    }

    let mut args = vec![
        "run".to_string(),
        "-it".to_string(),
        "--userns=keep-id".to_string(),
        "-w".to_string(),
        workdir.clone(),
    ];
    if !keep_container {
        args.push("--rm".to_string());
    }
    if let Some(name) = container_name {
        if !name.trim().is_empty() {
            args.push("--name".to_string());
            args.push(name.to_string());
        }
    }

    if !env_args.is_empty() {
        args.extend(env_args);
    }

    for port in ports {
        let trimmed = port.trim();
        if trimmed.is_empty() {
            continue;
        }
        args.push("-p".to_string());
        args.push(trimmed.to_string());
    }

    if let Some(args_list) = settings.podman_args.clone() {
        args.extend(args_list);
    }

    args.extend(mounts);

    if image.trim().is_empty() {
        image = "localhost/dungeon".to_string();
    }
    args.push(image);

    if run_command.trim().is_empty() {
        args.push("bash".to_string());
    } else {
        args.push("bash".to_string());
        args.push("-ic".to_string());
        args.push(run_command);
    }

    Ok(CommandSpec {
        program: "podman".to_string(),
        args,
    })
}

pub fn run_podman_command(spec: CommandSpec) -> Result<(), AppError> {
    let mut cmd = Command::new(spec.program);
    cmd.args(spec.args);
    run_command(&mut cmd)
}

fn run_command(cmd: &mut Command) -> Result<(), AppError> {
    cmd.stdin(Stdio::inherit());
    cmd.stdout(Stdio::inherit());
    cmd.stderr(Stdio::inherit());
    match cmd.status() {
        Ok(status) if status.success() => Ok(()),
        Ok(status) => {
            let code = status.code().unwrap_or(1);
            Err(AppError::Subprocess(
                code,
                format!("podman exited with code {}", code),
            ))
        }
        Err(err) => Err(AppError::Io(err)),
    }
}

fn build_env_args(env_specs: &[String]) -> Result<Vec<String>, AppError> {
    let mut args = Vec::new();
    for spec in env_specs {
        let trimmed = spec.trim();
        if trimmed.is_empty() {
            continue;
        }
        if !trimmed.contains('=') {
            let value = std::env::var(trimmed).map_err(|_| {
                AppError::message(format!("ERROR: env \"{}\" is not set on host", trimmed))
            })?;
            args.push("--env".to_string());
            args.push(format!("{}={}", trimmed, value));
            continue;
        }
        let (name, value) = trimmed
            .split_once('=')
            .ok_or_else(|| AppError::message(format!("ERROR: invalid env spec \"{}\"", trimmed)))?;
        if name.trim().is_empty() {
            return Err(AppError::message(format!(
                "ERROR: invalid env spec \"{}\"",
                trimmed
            )));
        }
        args.push("--env".to_string());
        args.push(format!("{}={}", name.trim(), value));
    }
    Ok(args)
}

fn same_dir(a: &Path, b: &Path) -> bool {
    a.canonicalize().ok() == b.canonicalize().ok()
}
