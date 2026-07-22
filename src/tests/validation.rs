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
        fs_entries: &[],
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
        fs_entries: &[],
    };

    assert_input_error_contains(
        input,
        "ERROR: --skip-cwd cannot be used with explicit paths",
    );
}

fn rejects_removed_persistence_flags() {
    let defaults = config::Config::default();
    let file_cfg = config::Config::default();
    let env_cfg = config::Config::default();

    for flag in ["--persist", "--persisted", "--discard"] {
        let result = cli::parse_args_with_sources(
            vec!["run".to_string(), flag.to_string()],
            &defaults,
            &file_cfg,
            &env_cfg,
        );
        let err = result.expect_err("expected removed persistence flag to fail");
        assert!(err.to_string().contains(flag));
    }
}

#[test]
fn errors_on_group_name_conflict() {
    let input = TestInput {
        toml: r#"
[env]
command = "zsh"
"#,
        args: &["run"],
        env: &[],
        cwd_name: "conflicting-group",
        cwd_entries: &[],
        fs_entries: &[],
    };

    assert_input_error_contains(
        input,
        "ERROR: group name 'env' conflicts with a reserved CLI flag",
    );
}

#[test]
fn rejects_conflicting_git_metadata_flags() {
    let defaults = config::Config::default();
    let file_cfg = config::Config::default();
    let env_cfg = config::Config::default();
    let args = vec![
        "run".to_string(),
        "--mount-git-metadata".to_string(),
        "--no-mount-git-metadata".to_string(),
    ];

    let result = cli::parse_args_with_sources(args, &defaults, &file_cfg, &env_cfg);
    let err = result.expect_err("expected conflicting git metadata flag error");
    assert!(err.to_string().contains(
        "ERROR: --mount-git-metadata and --no-mount-git-metadata are mutually exclusive"
    ));
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
fn top_level_help_is_handled() {
    let defaults = config::Config::default();
    let file_cfg = config::Config::default();
    let env_cfg = config::Config::default();
    let args = vec!["--help".to_string()];

    let parsed =
        cli::parse_args_with_sources(args, &defaults, &file_cfg, &env_cfg).expect("parse args");

    assert!(parsed.show_help);
}

#[test]
fn top_level_version_is_handled() {
    let defaults = config::Config::default();
    let file_cfg = config::Config::default();
    let env_cfg = config::Config::default();
    let args = vec!["--version".to_string()];

    let parsed =
        cli::parse_args_with_sources(args, &defaults, &file_cfg, &env_cfg).expect("parse args");

    assert!(parsed.show_version);
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

#[test]
fn image_requires_subcommand() {
    let defaults = config::Config::default();
    let file_cfg = config::Config::default();
    let env_cfg = config::Config::default();
    let args = vec!["image".to_string()];

    let result = cli::parse_args_with_sources(args, &defaults, &file_cfg, &env_cfg);
    let err = result.expect_err("expected image subcommand error");
    assert!(
        err.to_string()
            .contains("ERROR: image requires a subcommand (use: image build)")
    );
}

#[test]
fn cache_requires_subcommand() {
    let defaults = config::Config::default();
    let file_cfg = config::Config::default();
    let env_cfg = config::Config::default();
    let args = vec!["cache".to_string()];

    let result = cli::parse_args_with_sources(args, &defaults, &file_cfg, &env_cfg);
    let err = result.expect_err("expected cache subcommand error");
    assert!(
        err.to_string()
            .contains("ERROR: cache requires a subcommand (use: cache reset)")
    );
}

#[test]
fn unknown_included_group_errors() {
    let defaults = config::Config::default();
    let file_cfg = config::Config {
        include_groups: Some(vec!["missing-group".to_string()]),
        ..config::Config::default()
    };
    let env_cfg = config::Config::default();
    let args = vec!["run".to_string()];

    let result = cli::parse_args_with_sources(args, &defaults, &file_cfg, &env_cfg);
    let err = result.expect_err("expected unknown group error");
    assert!(
        err.to_string()
            .contains("ERROR: include_groups includes unknown group \"missing-group\"")
    );
}

#[test]
fn unknown_group_dependency_errors() {
    let input = TestInput {
        toml: r#"
[ai]
include_groups = ["missing"]
"#,
        args: &["run"],
        env: &[],
        cwd_name: "unknown-group-dependency",
        cwd_entries: &[],
        fs_entries: &[],
    };

    assert_input_error_contains(input, "group \"ai\" includes unknown group \"missing\"");
}

#[test]
fn group_inclusion_cycle_errors() {
    let input = TestInput {
        toml: r#"
[ai]
include_groups = ["skills"]

[skills]
include_groups = ["difit"]

[difit]
include_groups = ["ai"]
"#,
        args: &["run"],
        env: &[],
        cwd_name: "group-inclusion-cycle",
        cwd_entries: &[],
        fs_entries: &[],
    };

    assert_input_error_contains(input, "group inclusion cycle: ai -> skills -> difit -> ai");
}

#[test]
fn group_self_inclusion_errors() {
    let input = TestInput {
        toml: r#"
[ai]
include_groups = ["ai"]
"#,
        args: &["run"],
        env: &[],
        cwd_name: "group-self-inclusion",
        cwd_entries: &[],
        fs_entries: &[],
    };

    assert_input_error_contains(input, "group inclusion cycle: ai -> ai");
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
