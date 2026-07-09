use crate::tests::support::{TestInput, assert_command};

#[test]
fn applies_group_and_cli_overrides() {
    let input = TestInput {
        toml: r#"
[general]
always_on_groups = ["codex"]
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
fn always_on_groups_apply_in_order_with_last_winning_scalars() {
    let input = TestInput {
        toml: r#"
[general]
always_on_groups = ["alpha", "beta"]

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
