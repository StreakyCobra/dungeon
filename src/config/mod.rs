mod groups;
mod merge;
mod parse;
mod types;

pub use groups::{
    build_group_selection, merge_group_definitions, normalize_group_order, resolve_group_order,
};
pub use merge::{resolve_always_on_groups, resolve_settings};
pub use types::{Config, GroupConfig, ResolvedConfig, Settings, Sources};

use crate::cli;
use crate::error::AppError;

pub fn resolve(
    parsed: &cli::ParsedCLI,
    defaults: Config,
    file_cfg: Config,
    env_cfg: Config,
) -> Result<ResolvedConfig, AppError> {
    let group_defs = merge_group_definitions(&defaults.groups, &file_cfg.groups)?;
    let base_order = normalize_group_order(&resolve_always_on_groups(
        &defaults,
        &file_cfg,
        &env_cfg,
        &Config::default(),
    ))?;
    build_group_selection(&group_defs, &base_order)?;

    let group_flags = cli::collect_group_flags_from_names(parsed, &group_defs);
    let group_order = resolve_group_order(&base_order, &group_flags);

    let final_settings = resolve_settings(
        Sources {
            defaults: defaults.settings.clone(),
            file: file_cfg.settings.clone(),
            env: env_cfg.settings.clone(),
            cli: parsed.settings.clone(),
        },
        &group_defs,
        &group_order,
    )?;

    let container_name = crate::container::persist::resolve_container_name(
        parsed.persist_mode,
        &parsed.paths,
    )?;
    crate::container::persist::ensure_container_exists(parsed.persist_mode, &container_name)?;

    Ok(ResolvedConfig {
        settings: final_settings,
        paths: parsed.paths.clone(),
        persist_mode: parsed.persist_mode,
        container_name,
    })
}

pub fn resolve_with_defaults(parsed: &cli::ParsedCLI) -> Result<ResolvedConfig, AppError> {
    let defaults = load_defaults()?;
    let file_cfg = load_from_file()?;
    let env_cfg = load_from_env()?;
    resolve(parsed, defaults, file_cfg, env_cfg)
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
