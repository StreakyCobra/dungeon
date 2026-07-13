mod groups;
mod merge;
mod parse;
mod types;

pub use groups::{
    merge_group_definitions, normalize_group_order, resolve_group_order, validate_group_selection,
};
pub use merge::{resolve_include_groups, resolve_settings};
pub use types::{Config, Engine, GroupConfig, ResolvedConfig, Settings, Sources};

use crate::cli;
use crate::error::AppError;

#[derive(Debug, Clone)]
pub struct LoadedConfigSources {
    pub defaults: Config,
    pub file: Config,
    pub env: Config,
}

pub fn resolve(
    parsed: &cli::ParsedCLI,
    sources: &LoadedConfigSources,
) -> Result<ResolvedConfig, AppError> {
    let group_defs = merge_group_definitions(&sources.defaults.groups, &sources.file.groups)?;
    let base_order = normalize_group_order(&resolve_include_groups(
        &sources.defaults,
        &sources.file,
        &sources.env,
        &Config::default(),
    ))?;
    validate_group_selection(&group_defs, &base_order)?;

    let group_flags = cli::collect_group_flags_from_names(parsed, &group_defs);
    let group_order = resolve_group_order(&base_order, &group_flags, &group_defs)?;

    let final_settings = resolve_settings(
        Sources {
            defaults: sources.defaults.settings.clone(),
            file: sources.file.settings.clone(),
            env: sources.env.settings.clone(),
            cli: parsed.settings.clone(),
        },
        &group_defs,
        &group_order,
    )?;
    crate::cli::validate_settings(&final_settings)?;

    let container_name =
        crate::container::persist::resolve_container_name(parsed.persist_mode, &parsed.paths)?;
    crate::container::persist::ensure_container_exists(
        parsed.persist_mode,
        &container_name,
        &final_settings,
    )?;

    Ok(ResolvedConfig {
        settings: final_settings,
        paths: parsed.paths.clone(),
        persist_mode: parsed.persist_mode,
        container_name,
        skip_cwd: parsed.skip_cwd,
    })
}

pub fn resolve_global_settings(
    cli_settings: &Settings,
    sources: &LoadedConfigSources,
) -> Result<Settings, AppError> {
    let group_defs = merge_group_definitions(&sources.defaults.groups, &sources.file.groups)?;
    let group_order = normalize_group_order(&resolve_include_groups(
        &sources.defaults,
        &sources.file,
        &sources.env,
        &Config::default(),
    ))?;
    validate_group_selection(&group_defs, &group_order)?;
    let group_order = resolve_group_order(
        &group_order,
        &std::collections::BTreeMap::new(),
        &group_defs,
    )?;

    resolve_settings(
        Sources {
            defaults: sources.defaults.settings.clone(),
            file: sources.file.settings.clone(),
            env: sources.env.settings.clone(),
            cli: cli_settings.clone(),
        },
        &group_defs,
        &group_order,
    )
}

pub fn load_defaults() -> Result<Config, AppError> {
    parse::load_defaults()
}

pub fn load_from_file() -> Result<Config, AppError> {
    parse::load_from_file()
}

pub fn load_from_env() -> Result<Config, AppError> {
    parse::load_from_env()
}

pub fn load_sources() -> Result<LoadedConfigSources, AppError> {
    Ok(LoadedConfigSources {
        defaults: load_defaults()?,
        file: load_from_file()?,
        env: load_from_env()?,
    })
}

pub fn validate_dynamic_port_names(names: &[String], field: &str) -> Result<(), AppError> {
    for name in names {
        let mut chars = name.bytes();
        if !matches!(chars.next(), Some(b'a'..=b'z'))
            || !chars.all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_')
        {
            return Err(AppError::message(format!(
                "{} entries must be lower-case ASCII identifiers ([a-z][a-z0-9_]*)",
                field
            )));
        }
    }
    Ok(())
}
