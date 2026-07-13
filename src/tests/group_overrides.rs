use crate::tests::support::{TestInput, assert_command, resolve_input};

#[test]
fn applies_group_and_cli_overrides() {
    let input = TestInput {
        toml: r#"
[general]
include_groups = ["codex"]
image = "localhost/dungeon-base"

[codex]
image = "localhost/dungeon-codex"
[obsidian]
image = "localhost/dungeon-obsidian"
"#,
        args: &["run", "--obsidian"],
        env: &[],
        cwd_name: "group-project",
        cwd_entries: &[],
        fs_entries: &[],
    };

    let expected = "podman run -it --userns=keep-id -w /workspace/group-project --rm -v <CWD>:/workspace/group-project localhost/dungeon-obsidian zsh";

    assert_command(input, expected);
}

#[test]
fn included_groups_apply_in_order_with_last_winning_scalars() {
    let input = TestInput {
        toml: r#"
[general]
include_groups = ["alpha", "beta"]

[alpha]
image = "localhost/dungeon-alpha"
command = "echo alpha"

[beta]
image = "localhost/dungeon-beta"
command = "echo beta"
"#,
        args: &["run"],
        env: &[],
        cwd_name: "group-order-project",
        cwd_entries: &[],
        fs_entries: &[],
    };

    let expected = "podman run -it --userns=keep-id -w /workspace/group-order-project --rm -v <CWD>:/workspace/group-order-project localhost/dungeon-beta zsh -ic echo beta";

    assert_command(input, expected);
}

#[test]
fn groups_include_dependencies_before_their_own_settings() {
    let output = resolve_input(TestInput {
        toml: r#"
[general]
include_groups = ["ai"]

[ai]
include_groups = ["skills", "difit"]
envs = ["AI=1"]
command = "echo ai"

[skills]
envs = ["SKILLS=1"]
command = "echo skills"

[difit]
envs = ["DIFIT=1"]
command = "echo difit"
"#,
        args: &["run"],
        env: &[],
        cwd_name: "nested-groups-project",
        cwd_entries: &[],
        fs_entries: &[],
    });

    assert_eq!(
        output.resolved.settings.env_vars,
        Some(vec![
            "SKILLS=1".to_string(),
            "DIFIT=1".to_string(),
            "AI=1".to_string(),
        ])
    );
    assert_eq!(output.resolved.settings.command.as_deref(), Some("echo ai"));
}

#[test]
fn shared_included_groups_are_applied_once() {
    let output = resolve_input(TestInput {
        toml: r#"
[general]
include_groups = ["skills", "difit"]

[skills]
include_groups = ["common"]
envs = ["SKILLS=1"]

[difit]
include_groups = ["common"]
envs = ["DIFIT=1"]

[common]
envs = ["COMMON=1"]
"#,
        args: &["run"],
        env: &[],
        cwd_name: "shared-groups-project",
        cwd_entries: &[],
        fs_entries: &[],
    });

    assert_eq!(
        output.resolved.settings.env_vars,
        Some(vec![
            "COMMON=1".to_string(),
            "SKILLS=1".to_string(),
            "DIFIT=1".to_string(),
        ])
    );
}

#[test]
fn cli_selected_group_includes_its_dependencies() {
    let output = resolve_input(TestInput {
        toml: r#"
[ai]
include_groups = ["skills"]
command = "echo ai"

[skills]
envs = ["SKILLS=1"]
"#,
        args: &["run", "--ai"],
        env: &[],
        cwd_name: "cli-nested-groups-project",
        cwd_entries: &[],
        fs_entries: &[],
    });

    assert_eq!(
        output.resolved.settings.env_vars,
        Some(vec!["SKILLS=1".to_string()])
    );
    assert_eq!(output.resolved.settings.command.as_deref(), Some("echo ai"));
}

#[test]
fn environment_included_groups_are_loaded() {
    let output = resolve_input(TestInput {
        toml: r#"
[ai]
command = "echo ai"
"#,
        args: &["run"],
        env: &[("DUNGEON_INCLUDE_GROUPS", "ai")],
        cwd_name: "environment-included-groups-project",
        cwd_entries: &[],
        fs_entries: &[],
    });

    assert_eq!(output.resolved.settings.command.as_deref(), Some("echo ai"));
}
