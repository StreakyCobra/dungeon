use std::{
    path::PathBuf,
    process::{Command, Stdio},
};

use sha2::{Digest, Sha256};

use crate::{container::podman::CommandSpec, error::AppError};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PersistMode {
    None,
    Create,
    Reuse,
    Discard,
}

pub fn resolve_container_name(
    persist_mode: PersistMode,
    paths: &[String],
) -> Result<String, AppError> {
    if persist_mode == PersistMode::Create {
        return persisted_container_name(paths);
    }
    if persist_mode != PersistMode::None {
        return persisted_container_name(&[]);
    }
    Ok(String::new())
}

pub fn ensure_container_exists(
    persist_mode: PersistMode,
    container_name: &str,
) -> Result<(), AppError> {
    if persist_mode == PersistMode::Reuse && !container_exists(container_name)? {
        return Err(AppError::message(format!(
            "ERROR: container \"{}\" does not exist",
            container_name
        )));
    }
    Ok(())
}

pub fn persisted_container_name(paths: &[String]) -> Result<String, AppError> {
    let cwd = std::env::current_dir()?;
    let abs_cwd = cwd.canonicalize()?;

    let mut hash_inputs = vec![abs_cwd.to_string_lossy().to_string()];
    for path in paths {
        let abs_path = PathBuf::from(path).canonicalize()?;
        hash_inputs.push(abs_path.to_string_lossy().to_string());
    }

    let mut hasher = Sha256::new();
    hasher.update(hash_inputs.join("\n"));
    let hash = hasher.finalize();
    let short_hash = hex::encode(hash)[..8].to_string();

    let base = sanitize_container_base(
        abs_cwd
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("project"),
    );

    let name = format!("dungeon-{}-{}", base, short_hash);
    validate_container_name(&name)?;
    Ok(name)
}

pub fn validate_container_name(name: &str) -> Result<(), AppError> {
    if name.trim().is_empty() {
        return Err(AppError::message("ERROR: container name cannot be empty"));
    }
    let max_len = 63usize;
    if name.len() > max_len {
        return Err(AppError::message(format!(
            "ERROR: container name exceeds {} characters",
            max_len
        )));
    }
    Ok(())
}

pub fn sanitize_container_base(name: &str) -> String {
    let mut cleaned = String::new();
    for ch in name.chars() {
        if ch.is_ascii_alphanumeric() || ch == '-' || ch == '_' || ch == '.' {
            cleaned.push(ch);
        } else {
            cleaned.push('-');
        }
    }
    let trimmed = cleaned.trim_matches('-');
    let fallback = "project";
    let base = if trimmed.is_empty() { fallback } else { trimmed };
    let max_len = 32usize;
    if base.len() <= max_len {
        return base.to_string();
    }
    let mut shortened = String::new();
    for ch in base.chars() {
        if shortened.len() + ch.len_utf8() > max_len {
            break;
        }
        shortened.push(ch);
    }
    if shortened.is_empty() {
        fallback.to_string()
    } else {
        shortened
    }
}

pub fn container_exists(name: &str) -> Result<bool, AppError> {
    validate_container_name(name)?;
    let status = Command::new("podman")
        .arg("container")
        .arg("exists")
        .arg(name)
        .status()?;
    Ok(status.success())
}

pub fn container_running(name: &str) -> Result<bool, AppError> {
    validate_container_name(name)?;
    let output = Command::new("podman")
        .arg("inspect")
        .arg("-f")
        .arg("{{.State.Running}}")
        .arg(name)
        .output()?;
    Ok(String::from_utf8_lossy(&output.stdout).trim() == "true")
}

pub fn start_container(name: &str) -> Result<(), AppError> {
    validate_container_name(name)?;
    run_podman(&["start", name])
}

pub fn exec_into_container(name: &str) -> Result<(), AppError> {
    validate_container_name(name)?;
    run_podman(&["exec", "-it", name, "bash"])
}

pub fn ensure_container_session(name: &str) -> Result<(), AppError> {
    let running = container_running(name)?;
    if !running {
        start_container(name)?;
    }
    exec_into_container(name)
}

pub fn discard_container(name: &str) -> Result<(), AppError> {
    validate_container_name(name)?;
    run_podman(&["rm", "-f", name])
}

pub fn run_persisted_session(name: &str, spec: CommandSpec) -> Result<(), AppError> {
    if container_exists(name)? {
        return Err(AppError::message(format!(
            "ERROR: container \"{}\" already exists, use --persisted to connect",
            name
        )));
    }
    let mut cmd = Command::new(spec.program);
    cmd.args(spec.args);
    cmd.stdin(Stdio::inherit());
    cmd.stdout(Stdio::inherit());
    cmd.stderr(Stdio::inherit());
    let status = cmd.status()?;
    if status.success() {
        Ok(())
    } else {
        Err(AppError::Subprocess(
            status.code().unwrap_or(1),
            "podman exited with error".to_string(),
        ))
    }
}

fn run_podman(args: &[&str]) -> Result<(), AppError> {
    let mut cmd = Command::new("podman");
    cmd.args(args);
    cmd.stdin(Stdio::inherit());
    cmd.stdout(Stdio::inherit());
    cmd.stderr(Stdio::inherit());
    let status = cmd.status()?;
    if status.success() {
        Ok(())
    } else {
        Err(AppError::Subprocess(
            status.code().unwrap_or(1),
            "podman exited with error".to_string(),
        ))
    }
}
