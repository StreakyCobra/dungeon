use std::{collections::BTreeMap, env};

use clap::{Arg, ArgAction, ArgMatches, Command};

use crate::{
    config::{self, Settings},
    container,
    container::persist::PersistMode,
    error::AppError,
};

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
    let always_on = config::resolve_always_on_groups(&defaults, &file_cfg, &env_cfg);
    let group_order = config::normalize_group_order(&always_on)?;
    let _group_enabled = config::build_group_selection(&group_defs, &group_order)?;

    let mut cmd = base_command(&group_defs);

    let mut argv = vec!["dungeon".to_string()];
    argv.extend(args);

    let matches = cmd
        .clone()
        .try_get_matches_from(argv.iter())
        .map_err(|err| AppError::message(err.to_string()))?;

    if matches.get_flag("help") {
        cmd.print_help().map_err(AppError::from)?;
        println!();
        return Ok(ParsedInput {
            settings: Settings::default(),
            paths: Vec::new(),
            show_version: false,
            reset_cache: false,
            persist_mode: PersistMode::None,
            container_name: String::new(),
        });
    }

    let persist_mode = config::resolve_persist_mode(
        matches.get_flag("persist"),
        matches.get_flag("persisted"),
        matches.get_flag("discard"),
    )?;

    let group_flags = collect_group_flags(&matches, &group_defs);
    let has_group_overrides = group_flags.values().any(|flag| flag.set);
    let has_config_overrides = has_config_override(&matches);

    let paths: Vec<String> = matches
        .get_many::<String>("paths")
        .map(|vals| vals.map(|s| s.to_string()).collect())
        .unwrap_or_default();

    if matches.get_flag("persisted") || matches.get_flag("discard") {
        if has_config_overrides || has_group_overrides || !paths.is_empty() {
            return Err(AppError::message(
                "ERROR: --persisted and --discard do not accept config, group, or path arguments",
            ));
        }
    }

    let cli_settings = config::Settings::from_cli(&matches);
    let group_order = config::resolve_group_order(&group_order, &group_flags);

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

    let container_name = if persist_mode == PersistMode::Create {
        container::persist::persisted_container_name(&paths)?
    } else if persist_mode != PersistMode::None {
        container::persist::persisted_container_name(&[])?
    } else {
        String::new()
    };

    if persist_mode == PersistMode::Reuse && !container::persist::container_exists(&container_name)?
    {
        return Err(AppError::message(format!(
            "ERROR: container \"{}\" does not exist",
            container_name
        )));
    }

    Ok(ParsedInput {
        settings: final_settings,
        paths,
        show_version: matches.get_flag("version"),
        reset_cache: matches.get_flag("reset-cache"),
        persist_mode,
        container_name,
    })
}

fn base_command(group_defs: &std::collections::BTreeMap<String, config::GroupConfig>) -> Command {
    let mut cmd = Command::new("dungeon")
        .disable_help_subcommand(true)
        .disable_help_flag(true)
        .disable_version_flag(true)
        .arg(
            Arg::new("help")
                .long("help")
                .help_heading("Options")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("reset-cache")
                .long("reset-cache")
                .help_heading("Options")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("version")
                .long("version")
                .help_heading("Options")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("persist")
                .long("persist")
                .help_heading("Options")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("persisted")
                .long("persisted")
                .help_heading("Options")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("discard")
                .long("discard")
                .help_heading("Options")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("run")
                .long("run")
                .help_heading("Configurations")
                .num_args(1)
                .action(ArgAction::Set),
        )
        .arg(
            Arg::new("image")
                .long("image")
                .help_heading("Configurations")
                .num_args(1)
                .action(ArgAction::Set),
        )
        .arg(
            Arg::new("port")
                .long("port")
                .help_heading("Configurations")
                .num_args(1)
                .action(ArgAction::Append),
        )
        .arg(
            Arg::new("cache")
                .long("cache")
                .help_heading("Configurations")
                .num_args(1)
                .action(ArgAction::Append),
        )
        .arg(
            Arg::new("mount")
                .long("mount")
                .help_heading("Configurations")
                .num_args(1)
                .action(ArgAction::Append),
        )
        .arg(
            Arg::new("env")
                .long("env")
                .help_heading("Configurations")
                .num_args(1)
                .action(ArgAction::Append),
        )
        .arg(
            Arg::new("podman-arg")
                .long("podman-arg")
                .help_heading("Configurations")
                .num_args(1)
                .action(ArgAction::Append),
        )
        .arg(Arg::new("paths").num_args(0..).action(ArgAction::Append));

    let group_names: Vec<String> = group_defs.keys().cloned().collect();
    for name in group_names.iter() {
        let leaked: &'static str = Box::leak(name.clone().into_boxed_str());
        cmd = cmd.arg(
            Arg::new(leaked)
                .long(leaked)
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
    matches.contains_id("run")
        || matches.contains_id("image")
        || matches.contains_id("port")
        || matches.contains_id("cache")
        || matches.contains_id("mount")
        || matches.contains_id("env")
        || matches.contains_id("podman-arg")
}
