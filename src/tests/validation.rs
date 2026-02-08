use std::collections::BTreeMap;

use crate::tests::support::{TestInput, try_run_input};
use crate::{cli, config};

#[test]
fn errors_on_unknown_config_keys() {
    let input = TestInput {
        toml: "unknown = 'value'",
        args: &["run"],
        env: &[],
        cwd_name: "unknown-config",
        cwd_entries: &[],
    };

    assert_input_error_contains(input, "group \"unknown\" must be a table");
}

#[test]
fn errors_when_skip_cwd_with_paths() {
    let input = TestInput {
        toml: "",
        args: &["run", "--skip-cwd", "folder1"],
        env: &[],
        cwd_name: "skip-cwd-paths",
        cwd_entries: &["folder1/"],
    };

    assert_input_error_contains(
        input,
        "ERROR: --skip-cwd cannot be used with explicit paths",
    );
}

#[test]
fn errors_on_group_name_conflict() {
    let input = TestInput {
        toml: "[env]\ncommand = 'bash'\n",
        args: &["run"],
        env: &[],
        cwd_name: "conflicting-group",
        cwd_entries: &[],
    };

    assert_input_error_contains(
        input,
        "ERROR: group name 'env' conflicts with a reserved CLI flag",
    );
}

#[test]
fn persisted_allows_group_flags_without_overrides() {
    let defaults = config::Config::default();
    let mut file_cfg = config::Config::default();
    file_cfg.groups = BTreeMap::from([("x11".to_string(), config::GroupConfig::default())]);
    let env_cfg = config::Config::default();
    let args = vec!["run".to_string(), "--persisted".to_string()];

    let parsed =
        cli::parse_args_with_sources(args, &defaults, &file_cfg, &env_cfg).expect("parse args");

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
        "run".to_string(),
        "--persisted".to_string(),
        "--engine".to_string(),
        "docker".to_string(),
    ];

    let parsed =
        cli::parse_args_with_sources(args, &defaults, &file_cfg, &env_cfg).expect("parse args");

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
    let args = vec![
        "run".to_string(),
        "--debug".to_string(),
        "--persist".to_string(),
    ];

    let result = cli::parse_args_with_sources(args, &defaults, &file_cfg, &env_cfg);

    assert!(result.is_err());
}

#[test]
fn requires_subcommand() {
    let defaults = config::Config::default();
    let file_cfg = config::Config::default();
    let env_cfg = config::Config::default();
    let args = vec!["--debug".to_string()];

    let result = cli::parse_args_with_sources(args, &defaults, &file_cfg, &env_cfg);

    assert!(result.is_err());
}

#[test]
fn run_help_is_handled() {
    let defaults = config::Config::default();
    let file_cfg = config::Config::default();
    let env_cfg = config::Config::default();
    let args = vec!["run".to_string(), "--help".to_string()];

    let parsed =
        cli::parse_args_with_sources(args, &defaults, &file_cfg, &env_cfg).expect("parse args");

    assert!(parsed.show_help);
}

#[test]
fn image_build_help_is_handled() {
    let defaults = config::Config::default();
    let file_cfg = config::Config::default();
    let env_cfg = config::Config::default();
    let args = vec![
        "image".to_string(),
        "build".to_string(),
        "--help".to_string(),
    ];

    let parsed =
        cli::parse_args_with_sources(args, &defaults, &file_cfg, &env_cfg).expect("parse args");

    assert!(parsed.show_help);
}

#[test]
fn cache_reset_help_is_handled() {
    let defaults = config::Config::default();
    let file_cfg = config::Config::default();
    let env_cfg = config::Config::default();
    let args = vec![
        "cache".to_string(),
        "reset".to_string(),
        "--help".to_string(),
    ];

    let parsed =
        cli::parse_args_with_sources(args, &defaults, &file_cfg, &env_cfg).expect("parse args");

    assert!(parsed.show_help);
}

fn assert_input_error_contains(input: TestInput<'_>, expected_substring: &str) {
    let err = match try_run_input(input) {
        Ok(_) => panic!("expected input to fail"),
        Err(err) => err,
    };
    let message = err.to_string();
    assert!(
        message.contains(expected_substring),
        "expected error containing '{expected_substring}', got '{message}'"
    );
}
