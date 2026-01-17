use std::path::{Path, PathBuf};

use crate::error::AppError;

const USER_HOME: &str = "/home/dungeon";

pub fn parse_host_mount_spec(spec: &str) -> Result<(String, String, String), AppError> {
    let parts: Vec<_> = spec.split(':').collect();
    if parts.len() < 2 || parts.len() > 3 {
        return Err(AppError::message(format!(
            "ERROR: invalid mount spec \"{}\" (expected source:target[:ro|rw])",
            spec
        )));
    }
    let source = parts[0].trim();
    let target = parts[1].trim();
    if source.is_empty() || target.is_empty() {
        return Err(AppError::message(format!(
            "ERROR: invalid mount spec \"{}\" (source and target required)",
            spec
        )));
    }
    let mode = if parts.len() == 3 {
        mount_mode(parts[2])?
    } else {
        String::new()
    };
    Ok((source.to_string(), target.to_string(), mode))
}

pub fn parse_cache_mount_spec(spec: &str) -> Result<(String, String), AppError> {
    let parts: Vec<_> = spec.split(':').collect();
    if parts.is_empty() || parts.len() > 2 {
        return Err(AppError::message(format!(
            "ERROR: invalid cache mount spec \"{}\" (expected target[:ro|rw])",
            spec
        )));
    }
    let target = parts[0].trim();
    if target.is_empty() {
        return Err(AppError::message(format!(
            "ERROR: invalid cache mount spec \"{}\" (target required)",
            spec
        )));
    }
    let mode = if parts.len() == 2 {
        mount_mode(parts[1])?
    } else {
        String::new()
    };
    Ok((container_path(target), mode))
}

pub fn resolve_host_path(home: &Path, path: &str) -> Result<PathBuf, AppError> {
    let trimmed = path.trim();
    if trimmed.is_empty() {
        return Err(AppError::message("ERROR: mount source cannot be empty"));
    }
    if trimmed == "~" {
        return Ok(home.to_path_buf());
    }
    if trimmed.starts_with("~/") {
        let resolved = home.join(&trimmed[2..]).clean();
        return Ok(resolved);
    }
    let path = PathBuf::from(trimmed);
    if path.is_absolute() {
        return Ok(path.clean());
    }
    Ok(home.join(path).clean())
}

pub fn container_path(path: &str) -> String {
    let trimmed = path.trim();
    let path = PathBuf::from(trimmed);
    if path.is_absolute() {
        return path.clean().to_string_lossy().to_string();
    }
    PathBuf::from(USER_HOME)
        .join(path)
        .clean()
        .to_string_lossy()
        .to_string()
}

fn mount_mode(mode: &str) -> Result<String, AppError> {
    let trimmed = mode.trim().to_lowercase();
    if trimmed.is_empty() || trimmed == "rw" {
        return Ok(String::new());
    }
    if trimmed == "ro" {
        return Ok(":ro".to_string());
    }
    Err(AppError::message(format!(
        "ERROR: invalid mount mode '{}' (use 'ro' or 'rw')",
        mode
    )))
}

trait CleanPath {
    fn clean(&self) -> PathBuf;
}

impl CleanPath for PathBuf {
    fn clean(&self) -> PathBuf {
        let mut components = self.components().peekable();
        let mut result = PathBuf::new();
        while let Some(component) = components.next() {
            match component {
                std::path::Component::ParentDir => {
                    result.pop();
                }
                std::path::Component::CurDir => {}
                _ => result.push(component),
            }
        }
        result
    }
}
