use std::collections::BTreeMap;

use clap::ArgMatches;

use crate::{
    config::{self, Settings},
    error::AppError,
};

use super::constants::{
    FLAG_MOUNT_GIT_METADATA, FLAG_NO_MOUNT_GIT_METADATA, FLAG_SKIP_CWD, RESERVED_GROUP_NAMES,
};

pub(crate) fn validate_skip_cwd_with_paths(
    matches: &ArgMatches,
    paths: &[String],
) -> Result<(), AppError> {
    if matches.get_flag(FLAG_SKIP_CWD) && !paths.is_empty() {
        return Err(AppError::message(
            "ERROR: --skip-cwd cannot be used with explicit paths",
        ));
    }
    Ok(())
}

pub fn validate_settings(settings: &Settings) -> Result<(), AppError> {
    validate_exposed_host_ports(settings)?;
    validate_remote_runtime(settings)
}

pub(crate) fn validate_cli_settings(settings: &Settings) -> Result<(), AppError> {
    validate_settings(settings)
}

pub(crate) fn validate_cli_flag_conflicts(matches: &ArgMatches) -> Result<(), AppError> {
    if matches.get_flag(FLAG_MOUNT_GIT_METADATA) && matches.get_flag(FLAG_NO_MOUNT_GIT_METADATA) {
        return Err(AppError::message(
            "ERROR: --mount-git-metadata and --no-mount-git-metadata are mutually exclusive",
        ));
    }
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

fn validate_exposed_host_ports(settings: &Settings) -> Result<(), AppError> {
    let exposed_host_ports = settings.expose_host_ports.as_deref().unwrap_or(&[]);
    for spec in exposed_host_ports {
        if !is_valid_exposed_host_port_spec(spec.trim()) {
            return Err(AppError::message(format!(
                "ERROR: invalid exposed host port specification \"{}\"; expected PORT, PORT:HOST_PORT, RANGE, or RANGE:HOST_RANGE",
                spec
            )));
        }
    }

    if !exposed_host_ports.is_empty() && uses_explicit_network(settings) {
        return Err(AppError::message(
            "ERROR: expose_host_ports cannot be combined with --network or --net in run_args",
        ));
    }

    Ok(())
}

fn is_valid_exposed_host_port_spec(spec: &str) -> bool {
    let mut parts = spec.split(':');
    let Some(namespace_range) = parts.next().and_then(parse_port_range) else {
        return false;
    };
    let host_range = parts.next().map(parse_port_range);
    if parts.next().is_some() {
        return false;
    }

    match host_range {
        None => true,
        Some(Some(host_range)) => {
            namespace_range.1 - namespace_range.0 == host_range.1 - host_range.0
        }
        Some(None) => false,
    }
}

fn parse_port_range(value: &str) -> Option<(u16, u16)> {
    let (first, last) = match value.split_once('-') {
        Some((first, last)) => (parse_port(first)?, parse_port(last)?),
        None => {
            let port = parse_port(value)?;
            (port, port)
        }
    };

    (first <= last).then_some((first, last))
}

fn parse_port(value: &str) -> Option<u16> {
    value.parse::<u16>().ok().filter(|port| *port != 0)
}

fn uses_explicit_network(settings: &Settings) -> bool {
    settings
        .run_args
        .as_deref()
        .unwrap_or(&[])
        .iter()
        .any(|arg| {
            matches!(arg.as_str(), "--network" | "--net")
                || arg.starts_with("--network=")
                || arg.starts_with("--net=")
        })
}

fn uses_podman_connection(settings: &Settings) -> bool {
    settings
        .podman_args
        .as_deref()
        .unwrap_or(&[])
        .iter()
        .any(|arg| {
            arg == "-c"
                || arg == "-r"
                || arg == "--connection"
                || arg == "--remote"
                || arg == "--url"
                || arg.starts_with("-c=")
                || arg.starts_with("--connection=")
                || arg
                    .strip_prefix("--remote=")
                    .is_some_and(|value| value != "false")
                || arg.starts_with("--url=")
        })
}

fn uses_remote_podman_environment() -> bool {
    ["CONTAINER_HOST", "CONTAINER_CONNECTION"]
        .into_iter()
        .any(|key| std::env::var_os(key).is_some_and(|value| !value.is_empty()))
}

fn validate_remote_runtime(settings: &Settings) -> Result<(), AppError> {
    let uses_connection = uses_podman_connection(settings) || uses_remote_podman_environment();
    let uses_runtime = settings
        .run_args
        .as_deref()
        .unwrap_or(&[])
        .iter()
        .any(|arg| arg == "--runtime" || arg.starts_with("--runtime="));

    if uses_connection && uses_runtime {
        return Err(AppError::message(
            "ERROR: --runtime cannot be used with a Podman connection (-c/--connection); it is unsupported by Podman's remote client, including Podman machines",
        ));
    }

    Ok(())
}
