use crate::tests::support::{TestInput, assert_command};

#[test]
fn includes_env_and_env_files() {
    let input = TestInput {
        toml: r#"
[general]
command = "echo ok"
env_files = [".env", "config.env"]
"#,
        args: &["run"],
        env: &[("DUNGEON_ENVS", "FOO=bar")],
        cwd_name: "env-project",
        cwd_entries: &[],
        fs_entries: &[],
    };

    let expected = "podman run -it --userns=keep-id -w /workspace/env-project --rm --env FOO=bar --env-file .env --env-file config.env -v <CWD>:/workspace/env-project localhost/dungeon zsh -ic echo ok";

    assert_command(input, expected);
}

#[test]
fn env_list_from_env_var_trims_and_drops_empty_entries() {
    let input = TestInput {
        toml: "",
        args: &["run"],
        env: &[("DUNGEON_ENVS", " ,FOO=bar,  , BAR=baz ,")],
        cwd_name: "env-list-project",
        cwd_entries: &[],
        fs_entries: &[],
    };

    let expected = "podman run -it --userns=keep-id -w /workspace/env-list-project --rm --env FOO=bar --env BAR=baz -v <CWD>:/workspace/env-list-project localhost/dungeon zsh";

    assert_command(input, expected);
}
