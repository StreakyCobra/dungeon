use std::collections::BTreeMap;

use clap::ArgMatches;

use crate::{
    config::{self, Settings},
    container::persist::PersistMode,
    error::AppError,
};

use super::constants::{
    FLAG_DEBUG, FLAG_DISCARD, FLAG_PERSISTED, FLAG_SKIP_CWD, RESERVED_GROUP_NAMES,
};

pub(crate) fn validate_persist_flags(
    matches: &ArgMatches,
    has_config_overrides: bool,
    has_group_overrides: bool,
    paths: &[String],
) -> Result<(), AppError> {
    if (matches.get_flag(FLAG_PERSISTED) || matches.get_flag(FLAG_DISCARD))
        && (has_config_overrides || has_group_overrides || !paths.is_empty())
    {
        return Err(AppError::message(
            "ERROR: --persisted and --discard do not accept config, group, or path arguments",
        ));
    }
    if matches.get_flag(FLAG_SKIP_CWD) && !paths.is_empty() {
        return Err(AppError::message(
            "ERROR: --skip-cwd cannot be used with explicit paths",
        ));
    }
    Ok(())
}

pub(crate) fn validate_debug_flags(
    matches: &ArgMatches,
    persist_mode: PersistMode,
) -> Result<(), AppError> {
    if matches.get_flag(FLAG_DEBUG) && persist_mode != PersistMode::None {
        return Err(AppError::message(
            "ERROR: --debug cannot be combined with persistence flags",
        ));
    }
    Ok(())
}

pub(crate) fn validate_cli_settings(_settings: &Settings) -> Result<(), AppError> {
    Ok(())
}

pub(crate) fn validate_group_names(
    group_defs: &BTreeMap<String, config::GroupConfig>,
) -> Result<(), AppError> {
    for name in group_defs.keys() {
        if RESERVED_GROUP_NAMES.contains(&name.as_str()) {
            return Err(AppError::message(format!(
                "ERROR: group name '{}' conflicts with a reserved CLI flag",
                name
            )));
        }
    }
    Ok(())
}

pub(crate) fn resolve_persist_mode_from_flags(
    persist: bool,
    persisted: bool,
    discard: bool,
) -> Result<PersistMode, AppError> {
    let total = [persist, persisted, discard]
        .iter()
        .filter(|flag| **flag)
        .count();
    if total > 1 {
        return Err(AppError::message(
            "ERROR: --persist, --persisted, and --discard are mutually exclusive",
        ));
    }
    if discard {
        return Ok(PersistMode::Discard);
    }
    if persisted {
        return Ok(PersistMode::Reuse);
    }
    if persist {
        return Ok(PersistMode::Create);
    }
    Ok(PersistMode::None)
}
