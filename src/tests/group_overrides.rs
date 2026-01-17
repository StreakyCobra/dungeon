use crate::tests::support::{assert_command, TestInput};

#[test]
fn applies_group_and_cli_overrides() {
    let input = TestInput {
        toml: r#"
always_on_groups = ["codex"]
image = "localhost/dungeon-base"

[codex]
image = "localhost/dungeon-codex"
[obsidian]
image = "localhost/dungeon-obsidian"
"#,
        args: &["--obsidian"],
        env: &[],
        cwd_name: "group-project",
        cwd_entries: &[],
    };

    let expected = "podman run -it --userns=keep-id -w /home/dungeon/group-project --rm -v dungeon-cache:/home/dungeon/.cache -v dungeon-cache:/home/dungeon/.npm -v <CWD>:/home/dungeon/group-project localhost/dungeon-obsidian bash";

    assert_command(input, expected);
}
