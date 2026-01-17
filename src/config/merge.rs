use crate::error::AppError;

use super::{Settings, Sources};

pub fn resolve_settings(
    sources: Sources,
    groups: &std::collections::BTreeMap<String, super::GroupConfig>,
    group_order: &[String],
) -> Result<Settings, AppError> {
    let mut settings = sources.defaults;
    for name in group_order {
        let group = groups
            .get(name)
            .ok_or_else(|| AppError::message(format!("ERROR: unknown group \"{}\"", name)))?;
        settings = merge_settings(settings, group.settings.clone());
    }
    settings = merge_settings(settings, sources.file);
    settings = merge_settings(settings, sources.env);
    settings = merge_settings(settings, sources.cli);
    Ok(settings)
}

pub fn resolve_always_on_groups(
    defaults: &super::Config,
    file: &super::Config,
    env: &super::Config,
) -> Vec<String> {
    let mut groups = Vec::new();
    if let Some(list) = &defaults.always_on_groups {
        groups.extend(list.clone());
    }
    if let Some(list) = &file.always_on_groups {
        groups.extend(list.clone());
    }
    if let Some(list) = &env.always_on_groups {
        groups.extend(list.clone());
    }
    groups
}

pub fn resolve_persist_mode(
    persist: bool,
    persisted: bool,
    discard: bool,
) -> Result<crate::container::persist::PersistMode, AppError> {
    let total = [persist, persisted, discard].iter().filter(|x| **x).count();
    if total > 1 {
        return Err(AppError::message(
            "ERROR: --persist, --persisted, and --discard are mutually exclusive",
        ));
    }
    if discard {
        return Ok(crate::container::persist::PersistMode::Discard);
    }
    if persisted {
        return Ok(crate::container::persist::PersistMode::Reuse);
    }
    if persist {
        return Ok(crate::container::persist::PersistMode::Create);
    }
    Ok(crate::container::persist::PersistMode::None)
}

fn merge_settings(base: Settings, override_settings: Settings) -> Settings {
    let mut merged = base;
    if let Some(value) = override_settings.run_command {
        merged.run_command = Some(value);
    }
    if let Some(value) = override_settings.image {
        merged.image = Some(value);
    }
    if let Some(values) = override_settings.ports {
        merged.ports = Some(append_strings(merged.ports, values));
    }
    if let Some(values) = override_settings.cache {
        merged.cache = Some(append_strings(merged.cache, values));
    }
    if let Some(values) = override_settings.mounts {
        merged.mounts = Some(append_strings(merged.mounts, values));
    }
    if let Some(values) = override_settings.env_vars {
        merged.env_vars = Some(append_strings(merged.env_vars, values));
    }
    if let Some(values) = override_settings.podman_args {
        merged.podman_args = Some(append_strings(merged.podman_args, values));
    }
    merged
}

fn append_strings(base: Option<Vec<String>>, extra: Vec<String>) -> Vec<String> {
    let mut merged = base.unwrap_or_default();
    merged.extend(extra);
    merged
}
