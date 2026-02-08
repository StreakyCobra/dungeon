use std::collections::BTreeMap;

use crate::tests::support::{TestInput, run_input};
use crate::{cli, config};

#[test]
fn errors_on_unknown_config_keys() {
    let input = TestInput {
        toml: "unknown = 'value'",
        args: &[],
        env: &[],
        cwd_name: "unknown-config",
        cwd_entries: &[],
    };

    let result = std::panic::catch_unwind(|| run_input(input));
    assert!(result.is_err());
}

#[test]
fn errors_when_skip_cwd_with_paths() {
    let input = TestInput {
        toml: "",
        args: &["--skip-cwd", "folder1"],
        env: &[],
        cwd_name: "skip-cwd-paths",
        cwd_entries: &["folder1/"],
    };

    let result = std::panic::catch_unwind(|| run_input(input));
    assert!(result.is_err());
}

#[test]
fn errors_on_group_name_conflict() {
    let input = TestInput {
        toml: "[env]\nrun = 'bash'\n",
        args: &[],
        env: &[],
        cwd_name: "conflicting-group",
        cwd_entries: &[],
    };

    let result = std::panic::catch_unwind(|| run_input(input));
    assert!(result.is_err());
}

#[test]
fn persisted_allows_group_flags_without_overrides() {
    let defaults = config::Config::default();
    let mut file_cfg = config::Config::default();
    file_cfg.groups = BTreeMap::from([("x11".to_string(), config::GroupConfig::default())]);
    let env_cfg = config::Config::default();
    let args = vec!["--persisted".to_string()];

    let parsed =
        cli::parse_args_with_sources(args, defaults, file_cfg, env_cfg).expect("parse args");

    assert_eq!(
        parsed.persist_mode,
        crate::container::persist::PersistMode::Reuse
    );
}

#[test]
fn persisted_allows_engine_flag() {
    let defaults = config::Config::default();
    let file_cfg = config::Config::default();
    let env_cfg = config::Config::default();
    let args = vec![
        "--persisted".to_string(),
        "--engine".to_string(),
        "docker".to_string(),
    ];

    let parsed =
        cli::parse_args_with_sources(args, defaults, file_cfg, env_cfg).expect("parse args");

    assert_eq!(
        parsed.persist_mode,
        crate::container::persist::PersistMode::Reuse
    );
    assert_eq!(parsed.settings.engine, Some(config::Engine::Docker));
}

#[test]
fn debug_rejects_persistence_flags() {
    let defaults = config::Config::default();
    let file_cfg = config::Config::default();
    let env_cfg = config::Config::default();
    let args = vec!["--debug".to_string(), "--persist".to_string()];

    let result = cli::parse_args_with_sources(args, defaults, file_cfg, env_cfg);

    assert!(result.is_err());
}

#[test]
fn debug_rejects_reset_cache() {
    let defaults = config::Config::default();
    let file_cfg = config::Config::default();
    let env_cfg = config::Config::default();
    let args = vec!["--debug".to_string(), "--reset-cache".to_string()];

    let result = cli::parse_args_with_sources(args, defaults, file_cfg, env_cfg);

    assert!(result.is_err());
}
