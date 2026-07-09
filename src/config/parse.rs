use crate::error::AppError;
use std::{env, fs, path::PathBuf};

use super::{Config, Engine, GroupConfig, Settings};

const ENV_PREFIX: &str = "DUNGEON_";

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
            )));
        }
    };
    parse_config(&data)
        .map_err(|err| AppError::message(format!("parse config {}: {}", path.display(), err)))
}

pub fn load_from_env() -> Result<Config, AppError> {
    let mut cfg = Config::default();

    if let Ok(value) = env::var(format!("{}ENGINE", ENV_PREFIX)) {
        cfg.settings.engine = Some(parse_engine_value("engine", value.trim())?);
    }
    if let Ok(value) = env::var(format!("{}COMMAND", ENV_PREFIX)) {
        cfg.settings.command = Some(value.trim().to_string());
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
    if let Ok(value) = env::var(format!("{}RUN_ARGS", ENV_PREFIX)) {
        cfg.settings.run_args = Some(split_env_list(&value));
    }
    if let Ok(value) = env::var(format!("{}MOUNT_GIT_METADATA", ENV_PREFIX)) {
        cfg.settings.mount_git_metadata =
            Some(parse_bool_value("mount_git_metadata", value.trim())?);
    }
    if let Ok(value) = env::var(format!("{}IPV6", ENV_PREFIX)) {
        cfg.settings.ipv6 = Some(parse_bool_value("ipv6", value.trim())?);
    }
    if let Ok(value) = env::var(format!("{}ALLOW_DNS", ENV_PREFIX)) {
        cfg.settings.allow_dns = Some(parse_bool_value("allow_dns", value.trim())?);
    }
    if let Ok(value) = env::var(format!("{}ALLOWED_TCP_DOMAINS", ENV_PREFIX)) {
        cfg.settings.allowed_tcp_domains = Some(split_env_list(&value));
    }
    if let Ok(value) = env::var(format!("{}ALLOWED_TCP_HOSTS", ENV_PREFIX)) {
        cfg.settings.allowed_tcp_hosts = Some(split_env_list(&value));
    }
    if let Ok(value) = env::var(format!("{}ALWAYS_ON_GROUPS", ENV_PREFIX)) {
        cfg.always_on_groups = Some(split_env_list(&value));
    }

    Ok(cfg)
}

fn parse_config(data: &str) -> Result<Config, AppError> {
    let raw: toml::Value = toml::from_str(data)?;
    let table = raw
        .as_table()
        .ok_or_else(|| AppError::message("config root must be a table"))?;
    let mut cfg = Config::default();

    for (name, value) in table {
        if name == "general" {
            parse_general_config(value, &mut cfg)?;
            continue;
        }

        let group = parse_group_config(name, value)?;
        cfg.groups.insert(name.to_string(), group);
    }

    Ok(cfg)
}

fn parse_general_config(value: &toml::Value, cfg: &mut Config) -> Result<(), AppError> {
    let table = value
        .as_table()
        .ok_or_else(|| AppError::message("[general] must be a table"))?;

    for (key, value) in table {
        if key == "always_on_groups" {
            cfg.always_on_groups = Some(parse_string_vec("general", key, value)?);
            continue;
        }

        if !parse_settings_key(&mut cfg.settings, "general", key, value)? {
            return Err(AppError::message(format!(
                "[general] has unknown key \"{}\"",
                key
            )));
        }
    }

    Ok(())
}

fn parse_group_config(name: &str, value: &toml::Value) -> Result<GroupConfig, AppError> {
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
        if !parse_settings_key(&mut settings, name, key, value)? {
            return Err(AppError::message(format!(
                "group \"{}\" has unknown key \"{}\"",
                name, key
            )));
        }
    }

    Ok(GroupConfig {
        settings,
        disabled: false,
    })
}

fn parse_settings_key(
    settings: &mut Settings,
    scope: &str,
    key: &str,
    value: &toml::Value,
) -> Result<bool, AppError> {
    match key {
        "engine" => {
            let raw = parse_string(scope, key, value)?;
            settings.engine = Some(parse_engine_value(
                &format!("{}.{}", scope, key),
                raw.trim(),
            )?);
            Ok(true)
        }
        "mounts" => {
            settings.mounts = Some(parse_string_vec(scope, key, value)?);
            Ok(true)
        }
        "caches" => {
            settings.cache = Some(parse_string_vec(scope, key, value)?);
            Ok(true)
        }
        "envs" => {
            settings.env_vars = Some(parse_string_vec(scope, key, value)?);
            Ok(true)
        }
        "env_files" => {
            settings.env_files = Some(parse_string_vec(scope, key, value)?);
            Ok(true)
        }
        "command" => {
            settings.command = Some(parse_string(scope, key, value)?);
            Ok(true)
        }
        "image" => {
            settings.image = Some(parse_string(scope, key, value)?);
            Ok(true)
        }
        "ports" => {
            settings.ports = Some(parse_string_vec(scope, key, value)?);
            Ok(true)
        }
        "podman_args" => {
            settings.podman_args = Some(parse_string_vec(scope, key, value)?);
            Ok(true)
        }
        "run_args" => {
            settings.run_args = Some(parse_string_vec(scope, key, value)?);
            Ok(true)
        }
        "mount_git_metadata" => {
            settings.mount_git_metadata = Some(parse_bool(scope, key, value)?);
            Ok(true)
        }
        "ipv6" => {
            settings.ipv6 = Some(parse_bool(scope, key, value)?);
            Ok(true)
        }
        "allow_dns" => {
            settings.allow_dns = Some(parse_bool(scope, key, value)?);
            Ok(true)
        }
        "allowed_tcp_domains" => {
            settings.allowed_tcp_domains = Some(parse_string_vec(scope, key, value)?);
            Ok(true)
        }
        "allowed_tcp_hosts" => {
            settings.allowed_tcp_hosts = Some(parse_string_vec(scope, key, value)?);
            Ok(true)
        }
        _ => Ok(false),
    }
}

fn parse_string(group: &str, key: &str, value: &toml::Value) -> Result<String, AppError> {
    value
        .as_str()
        .map(|s| s.to_string())
        .ok_or_else(|| AppError::message(format!("{}.{} must be a string", group, key)))
}

fn parse_bool(scope: &str, key: &str, value: &toml::Value) -> Result<bool, AppError> {
    value
        .as_bool()
        .ok_or_else(|| AppError::message(format!("{}.{} must be a boolean", scope, key)))
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

fn parse_bool_value(scope: &str, value: &str) -> Result<bool, AppError> {
    match value {
        "1" | "true" | "TRUE" | "yes" | "YES" | "on" | "ON" => Ok(true),
        "0" | "false" | "FALSE" | "no" | "NO" | "off" | "OFF" => Ok(false),
        _ => Err(AppError::message(format!("{} must be a boolean", scope))),
    }
}

fn parse_engine_value(scope: &str, value: &str) -> Result<Engine, AppError> {
    match value {
        "podman" => Ok(Engine::Podman),
        _ => Err(AppError::message(format!(
            "{} must be one of: podman",
            scope
        ))),
    }
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
