use std::path::{Component, Path};

use crate::error::AppError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CacheSpec {
    Legacy(String),
    Subpath {
        subpath: String,
        target: String,
        mode: Option<CacheMode>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CacheMode {
    ReadOnly,
    ReadWrite,
}

impl CacheMode {
    pub fn podman_option(self) -> &'static str {
        match self {
            CacheMode::ReadOnly => "ro",
            CacheMode::ReadWrite => "rw",
        }
    }
}

pub fn parse_cache_spec(spec: &str) -> Result<CacheSpec, AppError> {
    let trimmed = spec.trim();
    let parts = trimmed.split(':').collect::<Vec<_>>();

    if parts.len() < 2 || parts[0].starts_with('/') || !parts[1].starts_with('/') {
        return Ok(CacheSpec::Legacy(trimmed.to_string()));
    }

    if parts.len() > 3 {
        return Err(invalid_cache_spec(
            trimmed,
            "expected SUBPATH:ABSOLUTE_TARGET[:ro|rw]",
        ));
    }

    let subpath = normalize_subpath(parts[0], trimmed)?;
    let target = parts[1];
    validate_target(target, trimmed)?;
    let mode = match parts.get(2).copied() {
        None => None,
        Some("ro") => Some(CacheMode::ReadOnly),
        Some("rw") => Some(CacheMode::ReadWrite),
        Some(_) => {
            return Err(invalid_cache_spec(trimmed, "mode must be either ro or rw"));
        }
    };

    Ok(CacheSpec::Subpath {
        subpath,
        target: target.to_string(),
        mode,
    })
}

pub fn validate_cache_specs(specs: &[String], scope: &str) -> Result<(), AppError> {
    for spec in specs {
        parse_cache_spec(spec).map_err(|err| AppError::message(format!("{}: {}", scope, err)))?;
    }
    Ok(())
}

fn normalize_subpath(value: &str, spec: &str) -> Result<String, AppError> {
    if value.contains(',') || value.chars().any(char::is_control) {
        return Err(invalid_cache_spec(
            spec,
            "subpath must not contain commas or control characters",
        ));
    }

    let path = Path::new(value);
    let mut components = Vec::new();
    for component in path.components() {
        match component {
            Component::Normal(part) => components.push(part.to_string_lossy().into_owned()),
            Component::CurDir
            | Component::ParentDir
            | Component::RootDir
            | Component::Prefix(_) => {
                return Err(invalid_cache_spec(
                    spec,
                    "subpath must be a relative path without . or .. components",
                ));
            }
        }
    }

    if components.is_empty() {
        return Err(invalid_cache_spec(spec, "subpath must not be empty"));
    }

    Ok(components.join("/"))
}

fn validate_target(target: &str, spec: &str) -> Result<(), AppError> {
    if !Path::new(target).is_absolute()
        || target.contains(',')
        || target.chars().any(char::is_control)
    {
        return Err(invalid_cache_spec(
            spec,
            "target must be an absolute path without commas or control characters",
        ));
    }
    Ok(())
}

fn invalid_cache_spec(spec: &str, reason: &str) -> AppError {
    AppError::message(format!("invalid cache entry \"{}\": {}", spec, reason))
}
