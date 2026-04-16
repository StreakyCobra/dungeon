use std::collections::BTreeMap;
use std::net::IpAddr;

use clap::ArgMatches;

use crate::{
    config::{self, Settings},
    container::persist::PersistMode,
    error::AppError,
};

use super::constants::{
    FLAG_ALLOW_DNS, FLAG_DEBUG, FLAG_DENY_DNS, FLAG_DISCARD, FLAG_NETWORK_IPV6,
    FLAG_NETWORK_NO_IPV6, FLAG_PERSISTED, FLAG_SKIP_CWD, RESERVED_GROUP_NAMES,
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

pub fn validate_settings(settings: &Settings) -> Result<(), AppError> {
    validate_network_settings(&settings.network)
}

pub(crate) fn validate_cli_settings(settings: &Settings) -> Result<(), AppError> {
    validate_settings(settings)
}

pub(crate) fn validate_cli_flag_conflicts(matches: &ArgMatches) -> Result<(), AppError> {
    if matches.get_flag(FLAG_NETWORK_IPV6) && matches.get_flag(FLAG_NETWORK_NO_IPV6) {
        return Err(AppError::message(
            "ERROR: --network-ipv6 and --network-no-ipv6 are mutually exclusive",
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

fn validate_network_settings(network: &crate::config::NetworkSettings) -> Result<(), AppError> {
    if let Some(domains) = &network.allowed_tcp_domains {
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

    if let Some(hosts) = &network.allowed_tcp_hosts {
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
