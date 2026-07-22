use std::collections::BTreeMap;
use std::net::IpAddr;

use clap::ArgMatches;

use crate::{
    config::{self, Settings},
    error::AppError,
};

use super::constants::{
    FLAG_ALLOW_DNS, FLAG_DENY_DNS, FLAG_IPV6, FLAG_MOUNT_GIT_METADATA, FLAG_NO_IPV6,
    FLAG_NO_MOUNT_GIT_METADATA, FLAG_SKIP_CWD, RESERVED_GROUP_NAMES,
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
    validate_network_settings(settings)?;
    validate_remote_runtime(settings)
}

pub(crate) fn validate_cli_settings(settings: &Settings) -> Result<(), AppError> {
    validate_settings(settings)
}

pub(crate) fn validate_cli_flag_conflicts(matches: &ArgMatches) -> Result<(), AppError> {
    if matches.get_flag(FLAG_IPV6) && matches.get_flag(FLAG_NO_IPV6) {
        return Err(AppError::message(
            "ERROR: --ipv6 and --no-ipv6 are mutually exclusive",
        ));
    }
    if matches.get_flag(FLAG_MOUNT_GIT_METADATA) && matches.get_flag(FLAG_NO_MOUNT_GIT_METADATA) {
        return Err(AppError::message(
            "ERROR: --mount-git-metadata and --no-mount-git-metadata are mutually exclusive",
        ));
    }
    if matches.get_flag(FLAG_ALLOW_DNS) && matches.get_flag(FLAG_DENY_DNS) {
        return Err(AppError::message(
            "ERROR: --allow-dns and --deny-dns are mutually exclusive",
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

fn validate_network_settings(settings: &Settings) -> Result<(), AppError> {
    if let Some(domains) = &settings.allowed_tcp_domains {
        for domain in domains {
            let trimmed = domain.trim();
            if trimmed.is_empty() {
                continue;
            }
            if !is_valid_domain(trimmed) {
                return Err(AppError::message(format!(
                    "ERROR: invalid network domain \"{}\"",
                    domain
                )));
            }
        }
    }

    if let Some(hosts) = &settings.allowed_tcp_hosts {
        for host in hosts {
            let trimmed = host.trim();
            if trimmed.is_empty() {
                continue;
            }
            if !is_valid_host_or_cidr(trimmed) {
                return Err(AppError::message(format!(
                    "ERROR: invalid network host \"{}\"",
                    host
                )));
            }
        }
    }

    Ok(())
}

fn validate_remote_runtime(settings: &Settings) -> Result<(), AppError> {
    let uses_connection = settings
        .podman_args
        .as_deref()
        .unwrap_or(&[])
        .iter()
        .any(|arg| arg == "-c" || arg == "--connection" || arg.starts_with("--connection="));
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

fn is_valid_domain(domain: &str) -> bool {
    if domain.len() > 253 || !domain.contains('.') {
        return false;
    }

    domain.split('.').all(|label| {
        !label.is_empty()
            && label.len() <= 63
            && !label.starts_with('-')
            && !label.ends_with('-')
            && label
                .chars()
                .all(|ch| ch.is_ascii_alphanumeric() || ch == '-')
    })
}

fn is_valid_host_or_cidr(value: &str) -> bool {
    if let Ok(_ip) = value.parse::<IpAddr>() {
        return true;
    }

    let Some((address, prefix)) = value.split_once('/') else {
        return false;
    };
    let Ok(address) = address.parse::<IpAddr>() else {
        return false;
    };
    let Ok(prefix) = prefix.parse::<u8>() else {
        return false;
    };

    match address {
        IpAddr::V4(_) => prefix <= 32,
        IpAddr::V6(_) => prefix <= 128,
    }
}
