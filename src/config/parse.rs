use crate::error::AppError;
use serde::Deserialize;
use std::{collections::BTreeMap, env, fs, path::PathBuf};

use super::{Config, GroupConfig, Settings};

const ENV_PREFIX: &str = "DUNGEON_";

#[derive(Debug, Deserialize)]
struct RawConfig {
    run: Option<String>,
    image: Option<String>,
    ports: Option<Vec<String>>,
    caches: Option<Vec<String>>,
    mounts: Option<Vec<String>>,
    envs: Option<Vec<String>>,
    env_files: Option<Vec<String>>,
    podman_args: Option<Vec<String>>,
    always_on_groups: Option<Vec<String>>,
    #[serde(flatten)]
    groups: BTreeMap<String, toml::Value>,
}

pub fn load_defaults() -> Result<Config, AppError> {
    let data = include_str!("defaults.toml");
    if data.trim().is_empty() {
        return Ok(Config::default());
    }
    parse_config(data)
}

pub fn load_from_file() -> Result<Config, AppError> {
    let path = config_path()?;
    let data = match fs::read_to_string(&path) {
        Ok(data) => data,
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => return Ok(Config::default()),
        Err(err) => {
            return Err(AppError::message(format!(
                "read config {}: {}",
                path.display(),
                err
            )))
        }
    };
    parse_config(&data)
        .map_err(|err| AppError::message(format!("parse config {}: {}", path.display(), err)))
}

pub fn load_from_env() -> Result<Config, AppError> {
    let mut cfg = Config::default();

    if let Ok(value) = env::var(format!("{}RUN", ENV_PREFIX)) {
        cfg.settings.run_command = Some(value.trim().to_string());
    }
    if let Ok(value) = env::var(format!("{}IMAGE", ENV_PREFIX)) {
        cfg.settings.image = Some(value.trim().to_string());
    }
    if let Ok(value) = env::var(format!("{}PORTS", ENV_PREFIX)) {
        cfg.settings.ports = Some(split_env_list(&value));
    }
    if let Ok(value) = env::var(format!("{}CACHES", ENV_PREFIX)) {
        cfg.settings.cache = Some(split_env_list(&value));
    }
    if let Ok(value) = env::var(format!("{}MOUNTS", ENV_PREFIX)) {
        cfg.settings.mounts = Some(split_env_list(&value));
    }
    if let Ok(value) = env::var(format!("{}ENVS", ENV_PREFIX)) {
        cfg.settings.env_vars = Some(split_env_list(&value));
    }
    if let Ok(value) = env::var(format!("{}ENV_FILES", ENV_PREFIX)) {
        cfg.settings.env_files = Some(split_env_list(&value));
    }
    if let Ok(value) = env::var(format!("{}PODMAN_ARGS", ENV_PREFIX)) {
        cfg.settings.podman_args = Some(split_env_list(&value));
    }
    if let Ok(value) = env::var(format!("{}ALWAYS_ON_GROUPS", ENV_PREFIX)) {
        cfg.always_on_groups = Some(split_env_list(&value));
    }

    Ok(cfg)
}

fn parse_config(data: &str) -> Result<Config, AppError> {
    let raw: RawConfig = toml::from_str(data)?;
    let mut cfg = Config::default();

    cfg.settings.run_command = raw.run;
    cfg.settings.image = raw.image;
    cfg.settings.ports = raw.ports;
    cfg.settings.cache = raw.caches;
    cfg.settings.mounts = raw.mounts;
    cfg.settings.env_vars = raw.envs;
    cfg.settings.env_files = raw.env_files;
    cfg.settings.podman_args = raw.podman_args;
    cfg.always_on_groups = raw.always_on_groups;

    for (name, value) in raw.groups {
        if is_reserved_key(&name) {
            continue;
        }
        let group = parse_group_config(&name, value)?;
        cfg.groups.insert(name, group);
    }

    Ok(cfg)
}

fn parse_group_config(name: &str, value: toml::Value) -> Result<GroupConfig, AppError> {
    let table = value
        .as_table()
        .ok_or_else(|| AppError::message(format!("group \"{}\" must be a table", name)))?;
    if table.is_empty() {
        return Ok(GroupConfig {
            disabled: true,
            ..GroupConfig::default()
        });
    }

    let mut settings = Settings::default();
    for (key, value) in table {
        match key.as_str() {
            "mounts" => settings.mounts = Some(parse_string_vec(name, key, value)?),
            "caches" => settings.cache = Some(parse_string_vec(name, key, value)?),
            "envs" => settings.env_vars = Some(parse_string_vec(name, key, value)?),
            "env_files" => settings.env_files = Some(parse_string_vec(name, key, value)?),
            "run" => settings.run_command = Some(parse_string(name, key, value)?),
            "image" => settings.image = Some(parse_string(name, key, value)?),
            "ports" => settings.ports = Some(parse_string_vec(name, key, value)?),
            "podman_args" => settings.podman_args = Some(parse_string_vec(name, key, value)?),
            _ => {
                return Err(AppError::message(format!(
                    "group \"{}\" has unknown key \"{}\"",
                    name, key
                )))
            }
        }
    }

    Ok(GroupConfig {
        settings,
        disabled: false,
    })
}

fn parse_string(group: &str, key: &str, value: &toml::Value) -> Result<String, AppError> {
    value
        .as_str()
        .map(|s| s.to_string())
        .ok_or_else(|| AppError::message(format!("{}.{} must be a string", group, key)))
}

fn parse_string_vec(group: &str, key: &str, value: &toml::Value) -> Result<Vec<String>, AppError> {
    match value {
        toml::Value::Array(values) => values
            .iter()
            .map(|item| {
                item.as_str().map(|s| s.to_string()).ok_or_else(|| {
                    AppError::message(format!("{}.{} must be a list of strings", group, key))
                })
            })
            .collect(),
        _ => Err(AppError::message(format!(
            "{}.{} must be a list of strings",
            group, key
        ))),
    }
}

fn split_env_list(value: &str) -> Vec<String> {
    value
        .split(',')
        .map(|part| part.trim())
        .filter(|part| !part.is_empty())
        .map(|part| part.to_string())
        .collect()
}

fn is_reserved_key(key: &str) -> bool {
    matches!(
        key,
        "run"
            | "image"
            | "ports"
            | "caches"
            | "mounts"
            | "envs"
            | "env_files"
            | "podman_args"
            | "always_on_groups"
    )
}

fn config_path() -> Result<PathBuf, AppError> {
    let config_home = env::var("XDG_CONFIG_HOME").ok();
    let base = if let Some(path) = config_home {
        PathBuf::from(path)
    } else {
        dirs::home_dir()
            .ok_or_else(|| AppError::message("unable to resolve home directory"))?
            .join(".config")
    };
    Ok(base.join("dungeon").join("config.toml"))
}
