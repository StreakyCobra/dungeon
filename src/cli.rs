use std::{collections::BTreeMap, env};

use clap::{Arg, ArgAction, ArgMatches, Command};

use crate::{
    config::{self, Settings},
    container,
    container::persist::PersistMode,
    error::AppError,
};

const FLAG_HELP: &str = "help";
const FLAG_RESET_CACHE: &str = "reset-cache";
const FLAG_VERSION: &str = "version";
const FLAG_PERSIST: &str = "persist";
const FLAG_PERSISTED: &str = "persisted";
const FLAG_DISCARD: &str = "discard";
const FLAG_RUN: &str = "run";
const FLAG_IMAGE: &str = "image";
const FLAG_PORT: &str = "port";
const FLAG_CACHE: &str = "cache";
const FLAG_MOUNT: &str = "mount";
const FLAG_ENV: &str = "env";
const FLAG_ENV_FILE: &str = "env-file";
const FLAG_PODMAN_ARG: &str = "podman-arg";
const ARG_PATHS: &str = "paths";

pub struct ParsedInput {
    pub settings: Settings,
    pub paths: Vec<String>,
    pub show_version: bool,
    pub reset_cache: bool,
    pub persist_mode: PersistMode,
    pub container_name: String,
}

pub fn build_version() -> String {
    let version = env!("CARGO_PKG_VERSION");
    if version != "" {
        return version.to_string();
    }
    "dev".to_string()
}

pub fn parse_args(args: Vec<String>) -> Result<ParsedInput, AppError> {
    let defaults = config::load_defaults()?;
    let file_cfg = config::load_from_file()?;
    let env_cfg = config::load_from_env()?;

    parse_args_with_sources(args, defaults, file_cfg, env_cfg)
}

pub fn parse_args_with_sources(
    args: Vec<String>,
    defaults: config::Config,
    file_cfg: config::Config,
    env_cfg: config::Config,
) -> Result<ParsedInput, AppError> {
    let group_defs = config::merge_group_definitions(&defaults.groups, &file_cfg.groups)?;
    let (_always_on, group_order) = resolve_group_selection(&defaults, &file_cfg, &env_cfg)?;
    let _group_enabled = config::build_group_selection(&group_defs, &group_order)?;

    let mut cmd = base_command(&group_defs);

    let matches = parse_matches(&mut cmd, args)?;

    if matches.get_flag(FLAG_HELP) {
        return print_help(cmd);
    }

    let persist_mode = resolve_persist_mode(&matches)?;
    let group_flags = collect_group_flags(&matches, &group_defs);
    let has_group_overrides = group_flags.values().any(|flag| flag.set);
    let has_config_overrides = has_config_override(&matches);
    let paths = collect_paths(&matches);

    validate_persist_flags(&matches, has_config_overrides, has_group_overrides, &paths)?;

    let cli_settings = config::Settings::from_cli(&matches);
    validate_cli_settings(&cli_settings)?;
    let group_names: Vec<String> = group_defs.keys().cloned().collect();
    let cli_groups = config::Settings::always_on_groups_from_cli(&matches, &group_names);
    let cli_group_cfg = config::Config {
        always_on_groups: cli_groups,
        ..config::Config::default()
    };
    let always_on = config::resolve_always_on_groups(&defaults, &file_cfg, &env_cfg, &cli_group_cfg);
    let group_order = config::resolve_group_order(&always_on, &group_flags);

    let final_settings = config::resolve_settings(
        config::Sources {
            defaults: defaults.settings.clone(),
            file: file_cfg.settings.clone(),
            env: env_cfg.settings.clone(),
            cli: cli_settings,
        },
        &group_defs,
        &group_order,
    )?;

    let container_name = resolve_container_name(persist_mode, &paths)?;

    ensure_container_exists(persist_mode, &container_name)?;

    Ok(ParsedInput {
        settings: final_settings,
        paths,
        show_version: matches.get_flag(FLAG_VERSION),
        reset_cache: matches.get_flag(FLAG_RESET_CACHE),
        persist_mode,
        container_name,
    })
}

fn parse_matches(cmd: &mut Command, args: Vec<String>) -> Result<ArgMatches, AppError> {
    let mut argv = vec!["dungeon".to_string()];
    argv.extend(args);

    cmd.clone()
        .try_get_matches_from(argv.iter())
        .map_err(|err| AppError::message(err.to_string()))
}

fn print_help(mut cmd: Command) -> Result<ParsedInput, AppError> {
    cmd.print_help().map_err(AppError::from)?;
    println!();
    Ok(ParsedInput {
        settings: Settings::default(),
        paths: Vec::new(),
        show_version: false,
        reset_cache: false,
        persist_mode: PersistMode::None,
        container_name: String::new(),
    })
}

fn resolve_group_selection(
    defaults: &config::Config,
    file_cfg: &config::Config,
    env_cfg: &config::Config,
) -> Result<(Vec<String>, Vec<String>), AppError> {
    let always_on =
        config::resolve_always_on_groups(defaults, file_cfg, env_cfg, &config::Config::default());
    let group_order = config::normalize_group_order(&always_on)?;
    Ok((always_on, group_order))
}

fn resolve_persist_mode(matches: &ArgMatches) -> Result<PersistMode, AppError> {
    config::resolve_persist_mode(
        matches.get_flag(FLAG_PERSIST),
        matches.get_flag(FLAG_PERSISTED),
        matches.get_flag(FLAG_DISCARD),
    )
}

fn collect_paths(matches: &ArgMatches) -> Vec<String> {
    matches
        .get_many::<String>(ARG_PATHS)
        .map(|vals| vals.map(|s| s.to_string()).collect())
        .unwrap_or_default()
}

fn validate_persist_flags(
    matches: &ArgMatches,
    has_config_overrides: bool,
    has_group_overrides: bool,
    paths: &[String],
) -> Result<(), AppError> {
    if matches.get_flag(FLAG_PERSISTED) || matches.get_flag(FLAG_DISCARD) {
        if has_config_overrides || has_group_overrides || !paths.is_empty() {
            return Err(AppError::message(
                "ERROR: --persisted and --discard do not accept config, group, or path arguments",
            ));
        }
    }
    Ok(())
}

fn resolve_container_name(persist_mode: PersistMode, paths: &[String]) -> Result<String, AppError> {
    if persist_mode == PersistMode::Create {
        return container::persist::persisted_container_name(paths);
    }
    if persist_mode != PersistMode::None {
        return container::persist::persisted_container_name(&[]);
    }
    Ok(String::new())
}

fn ensure_container_exists(persist_mode: PersistMode, container_name: &str) -> Result<(), AppError> {
    if persist_mode == PersistMode::Reuse && !container::persist::container_exists(container_name)? {
        return Err(AppError::message(format!(
            "ERROR: container \"{}\" does not exist",
            container_name
        )));
    }
    Ok(())
}

fn base_command(group_defs: &std::collections::BTreeMap<String, config::GroupConfig>) -> Command {
    let mut cmd = Command::new("dungeon")
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
            Arg::new(FLAG_RESET_CACHE)
                .long(FLAG_RESET_CACHE)
                .help("Clear the dungeon-cache volume before running")
                .help_heading("Options")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new(FLAG_VERSION)
                .long(FLAG_VERSION)
                .help("Show version information")
                .help_heading("Options")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new(FLAG_PERSIST)
                .long(FLAG_PERSIST)
                .help("Create a persisted container (fails if it already exists)")
                .help_heading("Persistence")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new(FLAG_PERSISTED)
                .long(FLAG_PERSISTED)
                .help("Connect to the existing persisted container")
                .help_heading("Persistence")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new(FLAG_DISCARD)
                .long(FLAG_DISCARD)
                .help("Remove the persisted container")
                .help_heading("Persistence")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new(FLAG_RUN)
                .long(FLAG_RUN)
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
                .help("Add a podman env-file (repeatable)")
                .help_heading("Configurations")
                .num_args(1)
                .action(ArgAction::Append),
        )
        .arg(
            Arg::new(FLAG_PODMAN_ARG)
                .long(FLAG_PODMAN_ARG)
                .help("Append an extra podman run argument (repeatable)")
                .help_heading("Configurations")
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
    for name in group_names.iter() {
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

#[derive(Default, Clone)]
pub struct GroupFlag {
    pub set: bool,
    pub order: usize,
}

fn collect_group_flags(
    matches: &ArgMatches,
    groups: &BTreeMap<String, config::GroupConfig>,
) -> BTreeMap<String, GroupFlag> {
    let mut flags = BTreeMap::new();
    let mut order = 0;
    for name in groups.keys() {
        let set = matches.get_flag(name);
        if set {
            order += 1;
            flags.insert(name.clone(), GroupFlag { set: true, order });
        } else {
            flags.insert(name.clone(), GroupFlag::default());
        }
    }
    flags
}

fn has_config_override(matches: &ArgMatches) -> bool {
    matches.contains_id(FLAG_RUN)
        || matches.contains_id(FLAG_IMAGE)
        || matches.contains_id(FLAG_PORT)
        || matches.contains_id(FLAG_CACHE)
        || matches.contains_id(FLAG_MOUNT)
        || matches.contains_id(FLAG_ENV)
        || matches.contains_id(FLAG_ENV_FILE)
        || matches.contains_id(FLAG_PODMAN_ARG)
}

fn validate_cli_settings(settings: &Settings) -> Result<(), AppError> {
    if let Some(run) = &settings.run_command {
        if run.trim().is_empty() {
            return Err(AppError::message("ERROR: --run cannot be empty"));
        }
    }
    if let Some(image) = &settings.image {
        if image.trim().is_empty() {
            return Err(AppError::message("ERROR: --image cannot be empty"));
        }
    }
    validate_string_list(settings.ports.as_ref(), "--port")?;
    validate_string_list(settings.cache.as_ref(), "--cache")?;
    validate_string_list(settings.mounts.as_ref(), "--mount")?;
    validate_string_list(settings.env_files.as_ref(), "--env-file")?;
    validate_string_list(settings.podman_args.as_ref(), "--podman-arg")?;
    if let Some(env_vars) = &settings.env_vars {
        for env in env_vars {
            let trimmed = env.trim();
            if trimmed.is_empty() {
                return Err(AppError::message("ERROR: --env cannot be empty"));
            }
            if trimmed.contains('=') {
                let (name, _) = trimmed
                    .split_once('=')
                    .ok_or_else(|| AppError::message("ERROR: invalid --env value"))?;
                if name.trim().is_empty() {
                    return Err(AppError::message("ERROR: invalid --env value"));
                }
            } else if trimmed.is_empty() {
                return Err(AppError::message("ERROR: --env cannot be empty"));
            }
        }
    }
    Ok(())
}

fn validate_string_list(values: Option<&Vec<String>>, flag: &str) -> Result<(), AppError> {
    if let Some(values) = values {
        for value in values {
            if value.trim().is_empty() {
                return Err(AppError::message(format!("ERROR: {} cannot be empty", flag)));
            }
        }
    }
    Ok(())
}
