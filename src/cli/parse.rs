use std::collections::BTreeMap;

use clap::{ArgMatches, Command};

use crate::{
    config::{self, Settings},
    container::persist::PersistMode,
    error::AppError,
};

use super::{
    build::{base_command, print_targeted_help},
    constants::{
        ARG_PATHS, FLAG_ALLOW_DNS, FLAG_ALLOW_DOMAIN, FLAG_ALLOW_HOST, FLAG_CACHE, FLAG_COMMAND,
        FLAG_CONTEXT, FLAG_DEBUG, FLAG_DENY_DNS, FLAG_DISCARD, FLAG_ENGINE_ARG, FLAG_ENV,
        FLAG_ENV_FILE, FLAG_IMAGE, FLAG_IPV6, FLAG_MOUNT, FLAG_NO_CACHE, FLAG_NO_IPV6,
        FLAG_PERSIST, FLAG_PERSISTED, FLAG_PORT, FLAG_SKIP_CWD, FLAG_TAG, FLAG_VERSION,
        SUBCOMMAND_CACHE, SUBCOMMAND_CACHE_RESET, SUBCOMMAND_IMAGE, SUBCOMMAND_IMAGE_BUILD,
        SUBCOMMAND_RUN,
    },
    types::{Action, CacheResetAction, GroupFlag, ImageBuildAction, ParsedCLI},
    validate::{
        resolve_persist_mode_from_flags, validate_cli_flag_conflicts, validate_cli_settings,
        validate_debug_flags, validate_group_names, validate_persist_flags,
    },
};

pub fn parse_args(args: Vec<String>) -> Result<ParsedCLI, AppError> {
    let sources = config::load_sources()?;
    parse_args_with_sources(args, &sources.defaults, &sources.file, &sources.env)
}

pub fn parse_args_with_sources(
    args: Vec<String>,
    defaults: &config::Config,
    file_cfg: &config::Config,
    env_cfg: &config::Config,
) -> Result<ParsedCLI, AppError> {
    let group_defs = config::merge_group_definitions(&defaults.groups, &file_cfg.groups)?;
    validate_group_names(&group_defs)?;

    let base_groups =
        config::resolve_always_on_groups(defaults, file_cfg, env_cfg, &config::Config::default());
    let base_order = config::normalize_group_order(&base_groups)?;
    let _group_enabled = config::build_group_selection(&group_defs, &base_order)?;

    let mut cmd = base_command(&group_defs);
    let matches = parse_matches(&mut cmd, args)?;

    if print_targeted_help(&matches, &group_defs)? {
        return Ok(empty_parsed(true, false));
    }
    if matches.get_flag(FLAG_VERSION) {
        return Ok(empty_parsed(false, true));
    }

    match matches.subcommand() {
        Some((SUBCOMMAND_RUN, run_matches)) => parse_run_action(run_matches, &group_defs),
        Some((SUBCOMMAND_IMAGE, image_matches)) => parse_image_action(image_matches),
        Some((SUBCOMMAND_CACHE, cache_matches)) => parse_cache_action(cache_matches),
        Some((name, _)) => Err(AppError::message(format!(
            "ERROR: unknown subcommand '{}'",
            name
        ))),
        None => Err(AppError::message(
            "ERROR: missing subcommand (use: run, image, cache)",
        )),
    }
}

pub fn collect_group_flags_from_names(
    parsed: &ParsedCLI,
    groups: &BTreeMap<String, config::GroupConfig>,
) -> BTreeMap<String, GroupFlag> {
    let mut flags = BTreeMap::new();
    for name in groups.keys() {
        if let Some(flag) = parsed.group_flags.get(name) {
            flags.insert(name.clone(), flag.clone());
        } else {
            flags.insert(name.clone(), GroupFlag::default());
        }
    }
    flags
}

fn empty_parsed(show_help: bool, show_version: bool) -> ParsedCLI {
    ParsedCLI {
        action: Action::None,
        settings: Settings::default(),
        paths: Vec::new(),
        show_help,
        show_version,
        debug: false,
        persist_mode: PersistMode::None,
        group_flags: BTreeMap::new(),
        skip_cwd: false,
    }
}

fn parse_run_action(
    matches: &ArgMatches,
    group_defs: &BTreeMap<String, config::GroupConfig>,
) -> Result<ParsedCLI, AppError> {
    let persist_mode = resolve_persist_mode_from_flags(
        matches.get_flag(FLAG_PERSIST),
        matches.get_flag(FLAG_PERSISTED),
        matches.get_flag(FLAG_DISCARD),
    )?;
    validate_debug_flags(matches, persist_mode)?;

    let group_flags = collect_group_flags(matches, group_defs);
    let has_group_overrides = group_flags.values().any(|flag| flag.set);
    let has_config_overrides = has_config_override(matches);
    let paths = collect_paths(matches);

    validate_cli_flag_conflicts(matches)?;
    validate_persist_flags(matches, has_config_overrides, has_group_overrides, &paths)?;

    let settings = settings_from_matches(matches)?;
    validate_cli_settings(&settings)?;

    Ok(ParsedCLI {
        action: Action::Run,
        settings,
        paths,
        show_help: false,
        show_version: false,
        debug: matches.get_flag(FLAG_DEBUG),
        persist_mode,
        group_flags,
        skip_cwd: matches.get_flag(FLAG_SKIP_CWD),
    })
}

fn parse_image_action(matches: &ArgMatches) -> Result<ParsedCLI, AppError> {
    let (sub_name, sub_matches) = matches.subcommand().ok_or_else(|| {
        AppError::message("ERROR: image requires a subcommand (use: image build)")
    })?;

    if sub_name != SUBCOMMAND_IMAGE_BUILD {
        return Err(AppError::message(format!(
            "ERROR: unknown image subcommand '{}'",
            sub_name
        )));
    }

    let tag = sub_matches
        .get_one::<String>(FLAG_TAG)
        .map(|value| value.to_string())
        .unwrap_or_else(|| "localhost/dungeon".to_string());
    let no_cache = sub_matches.get_flag(FLAG_NO_CACHE);
    let context = sub_matches
        .get_one::<String>(FLAG_CONTEXT)
        .map(|value| value.to_string())
        .unwrap_or_else(|| ".".to_string());

    Ok(ParsedCLI {
        action: Action::ImageBuild(ImageBuildAction {
            tag,
            no_cache,
            context,
        }),
        settings: Settings::default(),
        paths: Vec::new(),
        show_help: false,
        show_version: false,
        debug: false,
        persist_mode: PersistMode::None,
        group_flags: BTreeMap::new(),
        skip_cwd: false,
    })
}

fn parse_cache_action(matches: &ArgMatches) -> Result<ParsedCLI, AppError> {
    let (sub_name, _sub_matches) = matches.subcommand().ok_or_else(|| {
        AppError::message("ERROR: cache requires a subcommand (use: cache reset)")
    })?;

    if sub_name != SUBCOMMAND_CACHE_RESET {
        return Err(AppError::message(format!(
            "ERROR: unknown cache subcommand '{}'",
            sub_name
        )));
    }

    Ok(ParsedCLI {
        action: Action::CacheReset(CacheResetAction),
        settings: Settings::default(),
        paths: Vec::new(),
        show_help: false,
        show_version: false,
        debug: false,
        persist_mode: PersistMode::None,
        group_flags: BTreeMap::new(),
        skip_cwd: false,
    })
}

fn parse_matches(cmd: &mut Command, args: Vec<String>) -> Result<ArgMatches, AppError> {
    let mut argv = vec!["dungeon".to_string()];
    argv.extend(args);

    cmd.clone()
        .try_get_matches_from(argv.iter())
        .map_err(|err| AppError::message(err.to_string()))
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

fn collect_paths(matches: &ArgMatches) -> Vec<String> {
    matches
        .get_many::<String>(ARG_PATHS)
        .map(|vals| vals.map(|value| value.to_string()).collect())
        .unwrap_or_default()
}

fn has_config_override(matches: &ArgMatches) -> bool {
    matches.get_one::<String>(FLAG_COMMAND).is_some()
        || matches.get_one::<String>(FLAG_IMAGE).is_some()
        || matches.get_many::<String>(FLAG_PORT).is_some()
        || matches.get_many::<String>(FLAG_CACHE).is_some()
        || matches.get_many::<String>(FLAG_MOUNT).is_some()
        || matches.get_many::<String>(FLAG_ENV).is_some()
        || matches.get_many::<String>(FLAG_ENV_FILE).is_some()
        || matches.get_many::<String>(FLAG_ENGINE_ARG).is_some()
        || matches.get_flag(FLAG_IPV6)
        || matches.get_flag(FLAG_NO_IPV6)
        || matches.get_flag(FLAG_ALLOW_DNS)
        || matches.get_flag(FLAG_DENY_DNS)
        || matches.get_many::<String>(FLAG_ALLOW_DOMAIN).is_some()
        || matches.get_many::<String>(FLAG_ALLOW_HOST).is_some()
        || matches.get_flag(FLAG_SKIP_CWD)
}

fn settings_from_matches(matches: &ArgMatches) -> Result<Settings, AppError> {
    let mut settings = Settings::default();

    if let Some(value) = matches.get_one::<String>(FLAG_COMMAND) {
        settings.command = Some(value.to_string());
    }
    if let Some(value) = matches.get_one::<String>(FLAG_IMAGE) {
        settings.image = Some(value.to_string());
    }
    if let Some(values) = matches.get_many::<String>(FLAG_PORT) {
        settings.ports = Some(values.map(|value| value.to_string()).collect());
    }
    if let Some(values) = matches.get_many::<String>(FLAG_CACHE) {
        settings.cache = Some(values.map(|value| value.to_string()).collect());
    }
    if let Some(values) = matches.get_many::<String>(FLAG_MOUNT) {
        settings.mounts = Some(values.map(|value| value.to_string()).collect());
    }
    if let Some(values) = matches.get_many::<String>(FLAG_ENV) {
        settings.env_vars = Some(values.map(|value| value.to_string()).collect());
    }
    if let Some(values) = matches.get_many::<String>(FLAG_ENV_FILE) {
        settings.env_files = Some(values.map(|value| value.to_string()).collect());
    }
    if let Some(values) = matches.get_many::<String>(FLAG_ENGINE_ARG) {
        settings.engine_args = Some(values.map(|value| value.to_string()).collect());
    }
    if matches.get_flag(FLAG_IPV6) {
        settings.ipv6 = Some(true);
    }
    if matches.get_flag(FLAG_NO_IPV6) {
        settings.ipv6 = Some(false);
    }
    if matches.get_flag(FLAG_ALLOW_DNS) {
        settings.allow_dns = Some(true);
    }
    if matches.get_flag(FLAG_DENY_DNS) {
        settings.allow_dns = Some(false);
    }
    if let Some(values) = matches.get_many::<String>(FLAG_ALLOW_DOMAIN) {
        settings.allowed_tcp_domains = Some(values.map(|value| value.to_string()).collect());
    }
    if let Some(values) = matches.get_many::<String>(FLAG_ALLOW_HOST) {
        settings.allowed_tcp_hosts = Some(values.map(|value| value.to_string()).collect());
    }

    Ok(settings)
}
