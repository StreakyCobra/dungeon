use std::{collections::BTreeMap, env};

use clap::{Arg, ArgAction, ArgMatches, Command};

use crate::{
    config::{self, Engine, Settings},
    container::persist::PersistMode,
    error::AppError,
};

const SUBCOMMAND_RUN: &str = "run";
const SUBCOMMAND_IMAGE: &str = "image";
const SUBCOMMAND_IMAGE_BUILD: &str = "build";
const SUBCOMMAND_CACHE: &str = "cache";
const SUBCOMMAND_CACHE_RESET: &str = "reset";

const FLAG_HELP: &str = "help";
const FLAG_VERSION: &str = "version";
const FLAG_DEBUG: &str = "debug";
const FLAG_PERSIST: &str = "persist";
const FLAG_PERSISTED: &str = "persisted";
const FLAG_DISCARD: &str = "discard";
const FLAG_ENGINE: &str = "engine";
const FLAG_RUN: &str = "run";
const FLAG_IMAGE: &str = "image";
const FLAG_PORT: &str = "port";
const FLAG_CACHE: &str = "cache";
const FLAG_MOUNT: &str = "mount";
const FLAG_ENV: &str = "env";
const FLAG_ENV_FILE: &str = "env-file";
const FLAG_ENGINE_ARG: &str = "engine-arg";
const FLAG_SKIP_CWD: &str = "skip-cwd";
const FLAG_TAG: &str = "tag";
const FLAG_NO_CACHE: &str = "no-cache";
const FLAG_CONTEXT: &str = "context";
const ARG_PATHS: &str = "paths";
const ARG_FLAVOR: &str = "flavor";

const RESERVED_GROUP_NAMES: &[&str] = &[
    FLAG_HELP,
    FLAG_VERSION,
    FLAG_DEBUG,
    FLAG_PERSIST,
    FLAG_PERSISTED,
    FLAG_DISCARD,
    FLAG_ENGINE,
    FLAG_RUN,
    FLAG_IMAGE,
    FLAG_PORT,
    FLAG_CACHE,
    FLAG_MOUNT,
    FLAG_ENV,
    FLAG_ENV_FILE,
    FLAG_ENGINE_ARG,
    FLAG_SKIP_CWD,
    FLAG_TAG,
    FLAG_NO_CACHE,
    FLAG_CONTEXT,
    ARG_PATHS,
    ARG_FLAVOR,
    SUBCOMMAND_RUN,
    SUBCOMMAND_IMAGE,
    SUBCOMMAND_IMAGE_BUILD,
    SUBCOMMAND_CACHE,
    SUBCOMMAND_CACHE_RESET,
];

#[derive(Debug, Clone)]
pub struct ParsedCLI {
    pub action: Action,
    pub settings: Settings,
    pub paths: Vec<String>,
    pub show_help: bool,
    pub show_version: bool,
    pub debug: bool,
    pub persist_mode: PersistMode,
    pub group_flags: BTreeMap<String, GroupFlag>,
    pub skip_cwd: bool,
}

#[derive(Debug, Clone)]
pub enum Action {
    None,
    Run,
    ImageBuild(ImageBuildAction),
    CacheReset(CacheResetAction),
}

#[derive(Debug, Clone)]
pub struct ImageBuildAction {
    pub engine: Engine,
    pub flavor: ImageFlavor,
    pub tag: String,
    pub no_cache: bool,
    pub context: String,
}

#[derive(Debug, Clone)]
pub struct CacheResetAction {
    pub engine: Engine,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImageFlavor {
    Archlinux,
    Ubuntu,
}

impl ImageFlavor {
    pub fn containerfile_path(self) -> &'static str {
        match self {
            ImageFlavor::Archlinux => "images/Containerfile.archlinux",
            ImageFlavor::Ubuntu => "images/Containerfile.ubuntu",
        }
    }
}

#[derive(Default, Clone, Debug)]
pub struct GroupFlag {
    pub set: bool,
    pub order: usize,
}

pub fn build_version() -> String {
    let version = env!("CARGO_PKG_VERSION");
    if version != "" {
        return version.to_string();
    }
    "dev".to_string()
}

pub fn parse_args(args: Vec<String>) -> Result<ParsedCLI, AppError> {
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
) -> Result<ParsedCLI, AppError> {
    let group_defs = config::merge_group_definitions(&defaults.groups, &file_cfg.groups)?;
    validate_group_names(&group_defs)?;
    let base_groups = config::resolve_always_on_groups(
        &defaults,
        &file_cfg,
        &env_cfg,
        &config::Config::default(),
    );
    let base_order = config::normalize_group_order(&base_groups)?;
    let _group_enabled = config::build_group_selection(&group_defs, &base_order)?;

    let mut cmd = base_command(&group_defs);
    let matches = parse_matches(&mut cmd, args)?;

    if matches.get_flag(FLAG_HELP) {
        print_help(cmd)?;
        return Ok(ParsedCLI {
            action: Action::None,
            settings: Settings::default(),
            paths: Vec::new(),
            show_help: true,
            show_version: false,
            debug: false,
            persist_mode: PersistMode::None,
            group_flags: BTreeMap::new(),
            skip_cwd: false,
        });
    }

    if matches.get_flag(FLAG_VERSION) {
        return Ok(ParsedCLI {
            action: Action::None,
            settings: Settings::default(),
            paths: Vec::new(),
            show_help: false,
            show_version: true,
            debug: false,
            persist_mode: PersistMode::None,
            group_flags: BTreeMap::new(),
            skip_cwd: false,
        });
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

    let engine = parse_optional_engine(sub_matches.get_one::<String>(FLAG_ENGINE))?;
    let flavor = parse_image_flavor(
        sub_matches
            .get_one::<String>(ARG_FLAVOR)
            .ok_or_else(|| AppError::message("ERROR: missing image flavor"))?,
    )?;
    let tag = sub_matches
        .get_one::<String>(FLAG_TAG)
        .map(|s| s.to_string())
        .unwrap_or_else(|| "localhost/dungeon".to_string());
    let no_cache = sub_matches.get_flag(FLAG_NO_CACHE);
    let context = sub_matches
        .get_one::<String>(FLAG_CONTEXT)
        .map(|s| s.to_string())
        .unwrap_or_else(|| ".".to_string());

    Ok(ParsedCLI {
        action: Action::ImageBuild(ImageBuildAction {
            engine,
            flavor,
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
    let (sub_name, sub_matches) = matches.subcommand().ok_or_else(|| {
        AppError::message("ERROR: cache requires a subcommand (use: cache reset)")
    })?;

    if sub_name != SUBCOMMAND_CACHE_RESET {
        return Err(AppError::message(format!(
            "ERROR: unknown cache subcommand '{}'",
            sub_name
        )));
    }

    let engine = parse_optional_engine(sub_matches.get_one::<String>(FLAG_ENGINE))?;

    Ok(ParsedCLI {
        action: Action::CacheReset(CacheResetAction { engine }),
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

fn print_help(mut cmd: Command) -> Result<(), AppError> {
    cmd.print_help().map_err(AppError::from)?;
    println!();
    Ok(())
}

fn base_command(group_defs: &BTreeMap<String, config::GroupConfig>) -> Command {
    Command::new("dungeon")
        .disable_help_subcommand(true)
        .disable_help_flag(true)
        .disable_version_flag(true)
        .arg(
            Arg::new(FLAG_HELP)
                .long(FLAG_HELP)
                .help("Show help information")
                .help_heading("Options")
                .global(true)
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

fn run_subcommand(group_defs: &BTreeMap<String, config::GroupConfig>) -> Command {
    let mut cmd = Command::new(SUBCOMMAND_RUN)
        .about("Run a container session")
        .arg(
            Arg::new(FLAG_DEBUG)
                .long(FLAG_DEBUG)
                .help("Print the engine command without running")
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
            Arg::new(FLAG_ENGINE)
                .long(FLAG_ENGINE)
                .help("Select the container engine (podman or docker)")
                .help_heading("Configurations")
                .value_parser(["podman", "docker"])
                .num_args(1)
                .action(ArgAction::Set),
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
                .help("Add a container env-file (repeatable)")
                .help_heading("Configurations")
                .num_args(1)
                .action(ArgAction::Append),
        )
        .arg(
            Arg::new(FLAG_ENGINE_ARG)
                .long(FLAG_ENGINE_ARG)
                .help("Append an extra engine run argument (repeatable)")
                .help_heading("Configurations")
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
        .about("Manage dungeon images")
        .subcommand(
            Command::new(SUBCOMMAND_IMAGE_BUILD)
                .about("Build a provided image")
                .arg(
                    Arg::new(ARG_FLAVOR)
                        .help("Image flavor to build")
                        .value_parser(["archlinux", "ubuntu"])
                        .required(true)
                        .action(ArgAction::Set),
                )
                .arg(
                    Arg::new(FLAG_ENGINE)
                        .long(FLAG_ENGINE)
                        .help("Select the container engine (podman or docker)")
                        .value_parser(["podman", "docker"])
                        .num_args(1)
                        .action(ArgAction::Set),
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
                ),
        )
}

fn cache_subcommand() -> Command {
    Command::new(SUBCOMMAND_CACHE)
        .about("Manage dungeon cache")
        .subcommand(
            Command::new(SUBCOMMAND_CACHE_RESET)
                .about("Delete dungeon-cache volume")
                .arg(
                    Arg::new(FLAG_ENGINE)
                        .long(FLAG_ENGINE)
                        .help("Select the container engine (podman or docker)")
                        .value_parser(["podman", "docker"])
                        .num_args(1)
                        .action(ArgAction::Set),
                ),
        )
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
        .map(|vals| vals.map(|s| s.to_string()).collect())
        .unwrap_or_default()
}

fn has_config_override(matches: &ArgMatches) -> bool {
    matches.get_one::<String>(FLAG_RUN).is_some()
        || matches.get_one::<String>(FLAG_IMAGE).is_some()
        || matches.get_many::<String>(FLAG_PORT).is_some()
        || matches.get_many::<String>(FLAG_CACHE).is_some()
        || matches.get_many::<String>(FLAG_MOUNT).is_some()
        || matches.get_many::<String>(FLAG_ENV).is_some()
        || matches.get_many::<String>(FLAG_ENV_FILE).is_some()
        || matches.get_many::<String>(FLAG_ENGINE_ARG).is_some()
        || matches.get_flag(FLAG_SKIP_CWD)
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
    if matches.get_flag(FLAG_SKIP_CWD) && !paths.is_empty() {
        return Err(AppError::message(
            "ERROR: --skip-cwd cannot be used with explicit paths",
        ));
    }
    Ok(())
}

fn validate_debug_flags(matches: &ArgMatches, persist_mode: PersistMode) -> Result<(), AppError> {
    if matches.get_flag(FLAG_DEBUG) && persist_mode != PersistMode::None {
        return Err(AppError::message(
            "ERROR: --debug cannot be combined with persistence flags",
        ));
    }
    Ok(())
}

fn validate_cli_settings(_settings: &Settings) -> Result<(), AppError> {
    Ok(())
}

fn validate_group_names(
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

fn settings_from_matches(matches: &ArgMatches) -> Result<Settings, AppError> {
    let mut settings = Settings::default();

    if let Some(value) = matches.get_one::<String>(FLAG_ENGINE) {
        settings.engine = Some(parse_engine(value)?);
    }
    if let Some(value) = matches.get_one::<String>(FLAG_RUN) {
        settings.run_command = Some(value.to_string());
    }
    if let Some(value) = matches.get_one::<String>(FLAG_IMAGE) {
        settings.image = Some(value.to_string());
    }
    if let Some(values) = matches.get_many::<String>(FLAG_PORT) {
        settings.ports = Some(values.map(|s| s.to_string()).collect());
    }
    if let Some(values) = matches.get_many::<String>(FLAG_CACHE) {
        settings.cache = Some(values.map(|s| s.to_string()).collect());
    }
    if let Some(values) = matches.get_many::<String>(FLAG_MOUNT) {
        settings.mounts = Some(values.map(|s| s.to_string()).collect());
    }
    if let Some(values) = matches.get_many::<String>(FLAG_ENV) {
        settings.env_vars = Some(values.map(|s| s.to_string()).collect());
    }
    if let Some(values) = matches.get_many::<String>(FLAG_ENV_FILE) {
        settings.env_files = Some(values.map(|s| s.to_string()).collect());
    }
    if let Some(values) = matches.get_many::<String>(FLAG_ENGINE_ARG) {
        settings.engine_args = Some(values.map(|s| s.to_string()).collect());
    }

    Ok(settings)
}

fn parse_optional_engine(value: Option<&String>) -> Result<Engine, AppError> {
    if let Some(value) = value {
        parse_engine(value)
    } else {
        Ok(Engine::Podman)
    }
}

fn parse_engine(value: &str) -> Result<Engine, AppError> {
    match value {
        "podman" => Ok(Engine::Podman),
        "docker" => Ok(Engine::Docker),
        _ => Err(AppError::message(
            "ERROR: --engine must be podman or docker",
        )),
    }
}

fn parse_image_flavor(value: &str) -> Result<ImageFlavor, AppError> {
    match value {
        "archlinux" => Ok(ImageFlavor::Archlinux),
        "ubuntu" => Ok(ImageFlavor::Ubuntu),
        _ => Err(AppError::message(
            "ERROR: flavor must be one of: archlinux, ubuntu",
        )),
    }
}

fn resolve_persist_mode_from_flags(
    persist: bool,
    persisted: bool,
    discard: bool,
) -> Result<PersistMode, AppError> {
    let total = [persist, persisted, discard].iter().filter(|x| **x).count();
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
