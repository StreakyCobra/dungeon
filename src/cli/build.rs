use std::collections::BTreeMap;

use clap::{Arg, ArgAction, ArgMatches, Command};

use crate::{config, error::AppError};

use super::constants::{
    ARG_PATHS, FLAG_ALLOW_DNS, FLAG_ALLOW_DOMAIN, FLAG_ALLOW_HOST, FLAG_CACHE, FLAG_COMMAND,
    FLAG_CONTEXT, FLAG_DEBUG, FLAG_DENY_DNS, FLAG_DYNAMIC_PORT, FLAG_ENV, FLAG_ENV_FILE, FLAG_HELP,
    FLAG_IMAGE, FLAG_IPV6, FLAG_MOUNT, FLAG_MOUNT_GIT_METADATA, FLAG_NO_CACHE, FLAG_NO_IPV6,
    FLAG_NO_MOUNT_GIT_METADATA, FLAG_PODMAN_ARG, FLAG_PORT, FLAG_RUN_ARG, FLAG_SKIP_CWD, FLAG_TAG,
    FLAG_VERSION, SUBCOMMAND_CACHE, SUBCOMMAND_CACHE_RESET, SUBCOMMAND_IMAGE,
    SUBCOMMAND_IMAGE_BUILD, SUBCOMMAND_RUN,
};

pub(crate) fn print_targeted_help(
    matches: &ArgMatches,
    group_defs: &BTreeMap<String, config::GroupConfig>,
) -> Result<bool, AppError> {
    if matches.get_flag(FLAG_HELP) {
        print_help(base_command(group_defs))?;
        return Ok(true);
    }

    if let Some((sub_name, sub_matches)) = matches.subcommand() {
        match sub_name {
            SUBCOMMAND_RUN => {
                if sub_matches.get_flag(FLAG_HELP) {
                    print_help(run_subcommand(group_defs))?;
                    return Ok(true);
                }
            }
            SUBCOMMAND_IMAGE => {
                if sub_matches.get_flag(FLAG_HELP) {
                    print_help(image_subcommand())?;
                    return Ok(true);
                }
                if let Some((image_sub_name, image_sub_matches)) = sub_matches.subcommand()
                    && image_sub_name == SUBCOMMAND_IMAGE_BUILD
                    && image_sub_matches.get_flag(FLAG_HELP)
                {
                    print_help(image_build_subcommand())?;
                    return Ok(true);
                }
            }
            SUBCOMMAND_CACHE => {
                if sub_matches.get_flag(FLAG_HELP) {
                    print_help(cache_subcommand())?;
                    return Ok(true);
                }
                if let Some((cache_sub_name, cache_sub_matches)) = sub_matches.subcommand()
                    && cache_sub_name == SUBCOMMAND_CACHE_RESET
                    && cache_sub_matches.get_flag(FLAG_HELP)
                {
                    print_help(cache_reset_subcommand())?;
                    return Ok(true);
                }
            }
            _ => {}
        }
    }

    Ok(false)
}

pub(crate) fn base_command(group_defs: &BTreeMap<String, config::GroupConfig>) -> Command {
    Command::new("dungeon")
        .disable_help_subcommand(true)
        .disable_help_flag(true)
        .disable_version_flag(true)
        .arg(
            Arg::new(FLAG_HELP)
                .long(FLAG_HELP)
                .help("Show help information")
                .help_heading("Options")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new(FLAG_VERSION)
                .long(FLAG_VERSION)
                .help("Show version information")
                .help_heading("Options")
                .global(true)
                .action(ArgAction::SetTrue),
        )
        .subcommand(run_subcommand(group_defs))
        .subcommand(image_subcommand())
        .subcommand(cache_subcommand())
}

fn print_help(mut cmd: Command) -> Result<(), AppError> {
    cmd.print_help().map_err(AppError::from)?;
    println!();
    Ok(())
}

fn run_subcommand(group_defs: &BTreeMap<String, config::GroupConfig>) -> Command {
    let mut cmd = Command::new(SUBCOMMAND_RUN)
        .disable_help_flag(true)
        .about("Run a container session")
        .arg(
            Arg::new(FLAG_HELP)
                .long(FLAG_HELP)
                .help("Show help information")
                .help_heading("Options")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new(FLAG_DEBUG)
                .long(FLAG_DEBUG)
                .help("Print the engine command without running")
                .help_heading("Options")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new(FLAG_COMMAND)
                .long(FLAG_COMMAND)
                .help("Run a command inside the container")
                .help_heading("Configurations")
                .num_args(1)
                .action(ArgAction::Set),
        )
        .arg(
            Arg::new(FLAG_IMAGE)
                .long(FLAG_IMAGE)
                .help("Select the container image")
                .help_heading("Configurations")
                .num_args(1)
                .action(ArgAction::Set),
        )
        .arg(
            Arg::new(FLAG_PORT)
                .long(FLAG_PORT)
                .help("Publish a container port (repeatable)")
                .help_heading("Configurations")
                .num_args(1)
                .action(ArgAction::Append),
        )
        .arg(
            Arg::new(FLAG_DYNAMIC_PORT)
                .long(FLAG_DYNAMIC_PORT)
                .help(
                    "Publish a dynamic loopback port and set its environment variable (repeatable)",
                )
                .help_heading("Configurations")
                .num_args(1)
                .action(ArgAction::Append),
        )
        .arg(
            Arg::new(FLAG_CACHE)
                .long(FLAG_CACHE)
                .help("Mount a cache volume target (repeatable)")
                .help_heading("Configurations")
                .num_args(1)
                .action(ArgAction::Append),
        )
        .arg(
            Arg::new(FLAG_MOUNT)
                .long(FLAG_MOUNT)
                .help("Bind-mount a host path (repeatable)")
                .help_heading("Configurations")
                .num_args(1)
                .action(ArgAction::Append),
        )
        .arg(
            Arg::new(FLAG_ENV)
                .long(FLAG_ENV)
                .help("Add a container environment variable (repeatable)")
                .help_heading("Configurations")
                .num_args(1)
                .action(ArgAction::Append),
        )
        .arg(
            Arg::new(FLAG_ENV_FILE)
                .long(FLAG_ENV_FILE)
                .help("Add a container env-file (repeatable)")
                .help_heading("Configurations")
                .num_args(1)
                .action(ArgAction::Append),
        )
        .arg(
            Arg::new(FLAG_PODMAN_ARG)
                .long(FLAG_PODMAN_ARG)
                .help("Append an extra podman argument before the subcommand (repeatable)")
                .help_heading("Configurations")
                .allow_hyphen_values(true)
                .num_args(1)
                .action(ArgAction::Append),
        )
        .arg(
            Arg::new(FLAG_RUN_ARG)
                .long(FLAG_RUN_ARG)
                .help("Append an extra podman run argument (repeatable)")
                .help_heading("Configurations")
                .allow_hyphen_values(true)
                .num_args(1)
                .action(ArgAction::Append),
        )
        .arg(
            Arg::new(FLAG_SKIP_CWD)
                .long(FLAG_SKIP_CWD)
                .help("Skip mounting the current directory by default")
                .help_heading("Configurations")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new(FLAG_MOUNT_GIT_METADATA)
                .long(FLAG_MOUNT_GIT_METADATA)
                .help("Mount external Git metadata for worktrees")
                .help_heading("Configurations")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new(FLAG_NO_MOUNT_GIT_METADATA)
                .long(FLAG_NO_MOUNT_GIT_METADATA)
                .help("Do not mount external Git metadata")
                .help_heading("Configurations")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new(FLAG_IPV6)
                .long(FLAG_IPV6)
                .help("Enable IPv6 egress filtering and traffic")
                .help_heading("Network")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new(FLAG_NO_IPV6)
                .long(FLAG_NO_IPV6)
                .help("Disable IPv6 traffic entirely")
                .help_heading("Network")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new(FLAG_ALLOW_DNS)
                .long(FLAG_ALLOW_DNS)
                .help("Allow DNS queries from inside the container")
                .help_heading("Network")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new(FLAG_DENY_DNS)
                .long(FLAG_DENY_DNS)
                .help("Block DNS queries from inside the container")
                .help_heading("Network")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new(FLAG_ALLOW_DOMAIN)
                .long(FLAG_ALLOW_DOMAIN)
                .help("Allow outbound TCP to a domain (repeatable)")
                .help_heading("Network")
                .num_args(1)
                .action(ArgAction::Append),
        )
        .arg(
            Arg::new(FLAG_ALLOW_HOST)
                .long(FLAG_ALLOW_HOST)
                .help("Allow outbound TCP to an IP or CIDR (repeatable)")
                .help_heading("Network")
                .num_args(1)
                .action(ArgAction::Append),
        )
        .arg(
            Arg::new(ARG_PATHS)
                .help("Paths to mount inside the container (default: current directory)")
                .num_args(0..)
                .action(ArgAction::Append),
        );

    let group_names: Vec<String> = group_defs.keys().cloned().collect();
    for name in group_names {
        let leaked: &'static str = Box::leak(name.clone().into_boxed_str());
        cmd = cmd.arg(
            Arg::new(leaked)
                .long(leaked)
                .help(format!("Enable the {} group", name))
                .help_heading("Groups")
                .action(ArgAction::SetTrue),
        );
    }

    cmd
}

fn image_subcommand() -> Command {
    Command::new(SUBCOMMAND_IMAGE)
        .disable_help_flag(true)
        .about("Manage dungeon images")
        .arg(
            Arg::new(FLAG_HELP)
                .long(FLAG_HELP)
                .help("Show help information")
                .help_heading("Options")
                .action(ArgAction::SetTrue),
        )
        .subcommand(image_build_subcommand())
}

fn image_build_subcommand() -> Command {
    Command::new(SUBCOMMAND_IMAGE_BUILD)
        .disable_help_flag(true)
        .about("Build a provided image")
        .arg(
            Arg::new(FLAG_HELP)
                .long(FLAG_HELP)
                .help("Show help information")
                .help_heading("Options")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new(FLAG_PODMAN_ARG)
                .long(FLAG_PODMAN_ARG)
                .help("Append an extra podman argument before the subcommand (repeatable)")
                .allow_hyphen_values(true)
                .num_args(1)
                .action(ArgAction::Append),
        )
        .arg(
            Arg::new(FLAG_TAG)
                .long(FLAG_TAG)
                .help("Image tag to produce")
                .num_args(1)
                .default_value("localhost/dungeon")
                .action(ArgAction::Set),
        )
        .arg(
            Arg::new(FLAG_NO_CACHE)
                .long(FLAG_NO_CACHE)
                .help("Build without using cached layers")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new(FLAG_CONTEXT)
                .long(FLAG_CONTEXT)
                .help("Build context path")
                .num_args(1)
                .default_value(".")
                .action(ArgAction::Set),
        )
}

fn cache_subcommand() -> Command {
    Command::new(SUBCOMMAND_CACHE)
        .disable_help_flag(true)
        .about("Manage dungeon cache")
        .arg(
            Arg::new(FLAG_HELP)
                .long(FLAG_HELP)
                .help("Show help information")
                .help_heading("Options")
                .action(ArgAction::SetTrue),
        )
        .subcommand(cache_reset_subcommand())
}

fn cache_reset_subcommand() -> Command {
    Command::new(SUBCOMMAND_CACHE_RESET)
        .disable_help_flag(true)
        .about("Delete dungeon-cache volume")
        .arg(
            Arg::new(FLAG_HELP)
                .long(FLAG_HELP)
                .help("Show help information")
                .help_heading("Options")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new(FLAG_PODMAN_ARG)
                .long(FLAG_PODMAN_ARG)
                .help("Append an extra podman argument before the subcommand (repeatable)")
                .allow_hyphen_values(true)
                .num_args(1)
                .action(ArgAction::Append),
        )
}
