use std::collections::{BTreeMap, BTreeSet};

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

pub fn validate_group_selection(
    groups: &BTreeMap<String, GroupConfig>,
    included_groups: &[String],
) -> Result<(), AppError> {
    for name in included_groups {
        if !groups.contains_key(name) {
            return Err(AppError::message(format!(
                "ERROR: include_groups includes unknown group \"{}\"",
                name
            )));
        }
    }
    validate_group_inclusions(groups)
}

pub fn normalize_group_order(groups: &[String]) -> Result<Vec<String>, AppError> {
    let mut normalized = Vec::new();
    for name in groups {
        normalized.push(normalize_group_name(name)?);
    }
    Ok(normalized)
}

pub fn resolve_group_order(
    root_groups: &[String],
    flags: &BTreeMap<String, crate::cli::GroupFlag>,
    groups: &BTreeMap<String, GroupConfig>,
) -> Result<Vec<String>, AppError> {
    let mut order = root_groups.to_vec();
    let selected: Vec<(String, usize)> = flags
        .iter()
        .filter(|(_, flag)| flag.set)
        .map(|(name, flag)| (name.clone(), flag.order))
        .collect();

    if selected.is_empty() {
        return expand_group_order(groups, &order);
    }

    let mut selected_sorted = selected;
    selected_sorted.sort_by_key(|(_, order)| *order);
    let selected_set: BTreeSet<String> = selected_sorted
        .iter()
        .map(|(name, _)| name.clone())
        .collect();

    order.retain(|name| !selected_set.contains(name));
    for (name, _) in selected_sorted {
        order.push(name);
    }
    expand_group_order(groups, &order)
}

fn validate_group_inclusions(groups: &BTreeMap<String, GroupConfig>) -> Result<(), AppError> {
    let mut visited = BTreeSet::new();
    let mut active = Vec::new();
    for name in groups.keys() {
        validate_group_inclusions_from(name, groups, &mut visited, &mut active)?;
    }
    Ok(())
}

fn validate_group_inclusions_from(
    name: &str,
    groups: &BTreeMap<String, GroupConfig>,
    visited: &mut BTreeSet<String>,
    active: &mut Vec<String>,
) -> Result<(), AppError> {
    if visited.contains(name) {
        return Ok(());
    }
    if let Some(start) = active.iter().position(|active_name| active_name == name) {
        let mut cycle = active[start..].to_vec();
        cycle.push(name.to_string());
        return Err(AppError::message(format!(
            "ERROR: group inclusion cycle: {}",
            cycle.join(" -> ")
        )));
    }

    let group = groups
        .get(name)
        .expect("all group inclusion roots must exist");
    active.push(name.to_string());
    for included in &group.include_groups {
        let included = normalize_group_name(included)?;
        if !groups.contains_key(&included) {
            return Err(AppError::message(format!(
                "ERROR: group \"{}\" includes unknown group \"{}\"",
                name, included
            )));
        }
        validate_group_inclusions_from(&included, groups, visited, active)?;
    }
    active.pop();
    visited.insert(name.to_string());
    Ok(())
}

fn expand_group_order(
    groups: &BTreeMap<String, GroupConfig>,
    roots: &[String],
) -> Result<Vec<String>, AppError> {
    let mut expanded = Vec::new();
    let mut included = BTreeSet::new();
    for name in roots {
        expand_group(name, groups, &mut included, &mut expanded)?;
    }
    Ok(expanded)
}

fn expand_group(
    name: &str,
    groups: &BTreeMap<String, GroupConfig>,
    included: &mut BTreeSet<String>,
    expanded: &mut Vec<String>,
) -> Result<(), AppError> {
    if !included.insert(name.to_string()) {
        return Ok(());
    }

    let group = groups
        .get(name)
        .ok_or_else(|| AppError::message(format!("ERROR: unknown group \"{}\"", name)))?;
    for dependency in &group.include_groups {
        let dependency = normalize_group_name(dependency)?;
        expand_group(&dependency, groups, included, expanded)?;
    }
    expanded.push(name.to_string());
    Ok(())
}

fn normalize_group_name(name: &str) -> Result<String, AppError> {
    let trimmed = name.trim();
    if trimmed.is_empty() {
        return Err(AppError::message("ERROR: group name cannot be empty"));
    }
    Ok(trimmed.to_string())
}
