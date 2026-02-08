mod groups;
mod merge;
mod parse;
mod types;

pub use groups::{
    build_group_selection, merge_group_definitions, normalize_group_order, resolve_group_order,
};
pub use merge::{resolve_always_on_groups, resolve_settings};
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
    let base_order = normalize_group_order(&resolve_always_on_groups(
        &sources.defaults,
        &sources.file,
        &sources.env,
        &Config::default(),
    ))?;
    build_group_selection(&group_defs, &base_order)?;

    let group_flags = cli::collect_group_flags_from_names(parsed, &group_defs);
    let group_order = resolve_group_order(&base_order, &group_flags);

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

    let container_name =
        crate::container::persist::resolve_container_name(parsed.persist_mode, &parsed.paths)?;
    let engine = final_settings.engine.unwrap_or_default();
    crate::container::persist::ensure_container_exists(
        parsed.persist_mode,
        &container_name,
        engine,
    )?;

    Ok(ResolvedConfig {
        settings: final_settings,
        paths: parsed.paths.clone(),
        persist_mode: parsed.persist_mode,
        container_name,
        skip_cwd: parsed.skip_cwd,
    })
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
