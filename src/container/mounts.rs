use std::path::{Path, PathBuf};

use crate::error::AppError;

const USER_HOME: &str = "/home/dungeon";

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
