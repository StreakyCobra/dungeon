use std::{
    path::{Path, PathBuf},
    process::{Command, Stdio},
};

use crate::{
    config::Settings,
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
    skip_cwd: bool,
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
    let env_files = settings.env_files.clone().unwrap_or_default();
    let ports = settings.ports.clone().unwrap_or_default();
    let run_command = settings.run_command.clone().unwrap_or_default();
    let mut image = settings.image.clone().unwrap_or_default();

    for spec in settings.mounts.clone().unwrap_or_default() {
        mounts.push("-v".to_string());
        mounts.push(spec);
    }

    for spec in cache_specs {
        mounts.push("-v".to_string());
        mounts.push(format!("dungeon-cache:{}", spec));
    }

    let env_args = build_env_args(&env_specs);

    let workdir;
    if paths.is_empty() {
        if !skip_cwd && same_dir(&cwd, &home) {
            return Err(AppError::message(
                "ERROR: refusing to run from home directory",
            ));
        }
        if skip_cwd {
            workdir = USER_HOME.to_string();
        } else {
            let base = cwd
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or("project");
            workdir = format!("{}/{}", USER_HOME, base);
            mounts.push("-v".to_string());
            mounts.push(format!("{}:{}", cwd.display(), workdir));
        }
    } else {
        workdir = format!("{}/project", USER_HOME);
        for path in paths {
            let abs = PathBuf::from(path);
            let abs = if abs.is_absolute() { abs } else { cwd.join(&abs) };
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

    if !env_files.is_empty() {
        for env_file in env_files {
            let trimmed = env_file.trim();
            if trimmed.is_empty() {
                continue;
            }
            args.push("--env-file".to_string());
            args.push(trimmed.to_string());
        }
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

fn build_env_args(env_specs: &[String]) -> Vec<String> {
    let mut args = Vec::new();
    for spec in env_specs {
        let trimmed = spec.trim();
        if trimmed.is_empty() {
            continue;
        }
        args.push("--env".to_string());
        args.push(trimmed.to_string());
    }
    args
}

fn same_dir(a: &Path, b: &Path) -> bool {
    a.canonicalize().ok() == b.canonicalize().ok()
}
