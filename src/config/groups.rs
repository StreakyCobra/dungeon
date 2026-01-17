use std::collections::BTreeMap;

use crate::error::AppError;

use super::GroupConfig;

pub fn merge_group_definitions(
    base: &BTreeMap<String, GroupConfig>,
    overrides: &BTreeMap<String, GroupConfig>,
) -> Result<BTreeMap<String, GroupConfig>, AppError> {
    let mut merged = BTreeMap::new();
    for (name, group) in base.iter() {
        let trimmed = normalize_group_name(name)?;
        merged.insert(trimmed, group.clone());
    }
    for (name, group) in overrides.iter() {
        let trimmed = normalize_group_name(name)?;
        if group.disabled {
            merged.remove(&trimmed);
            continue;
        }
        let mut adjusted = group.clone();
        adjusted.disabled = false;
        merged.insert(trimmed, adjusted);
    }
    Ok(merged)
}

pub fn build_group_selection(
    groups: &BTreeMap<String, GroupConfig>,
    defaults: &[String],
) -> Result<BTreeMap<String, bool>, AppError> {
    let mut enabled = BTreeMap::new();
    for name in groups.keys() {
        enabled.insert(name.clone(), false);
    }
    for name in defaults {
        let trimmed = normalize_group_name(name)?;
        if !groups.contains_key(&trimmed) {
            return Err(AppError::message(format!(
                "ERROR: always_on_groups includes unknown group \"{}\"",
                trimmed
            )));
        }
        enabled.insert(trimmed, true);
    }
    Ok(enabled)
}

pub fn normalize_group_order(groups: &[String]) -> Result<Vec<String>, AppError> {
    let mut normalized = Vec::new();
    for name in groups {
        normalized.push(normalize_group_name(name)?);
    }
    Ok(normalized)
}

pub fn resolve_group_order(
    default_groups: &[String],
    flags: &BTreeMap<String, crate::cli::GroupFlag>,
) -> Vec<String> {
    let mut order = default_groups.to_vec();
    let selected: Vec<(String, usize)> = flags
        .iter()
        .filter(|(_, flag)| flag.set)
        .map(|(name, flag)| (name.clone(), flag.order))
        .collect();

    if selected.is_empty() {
        return order;
    }

    let mut selected_sorted = selected;
    selected_sorted.sort_by_key(|(_, order)| *order);
    let selected_set: std::collections::BTreeSet<String> = selected_sorted
        .iter()
        .map(|(name, _)| name.clone())
        .collect();

    order.retain(|name| !selected_set.contains(name));
    for (name, _) in selected_sorted {
        order.push(name);
    }
    order
}

fn normalize_group_name(name: &str) -> Result<String, AppError> {
    let trimmed = name.trim();
    if trimmed.is_empty() {
        return Err(AppError::message("ERROR: group name cannot be empty"));
    }
    Ok(trimmed.to_string())
}
