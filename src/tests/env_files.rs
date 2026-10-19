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

#[test]
fn cache_subpaths_are_parsed_from_environment() {
    let input = TestInput {
        toml: "",
        args: &["run"],
        env: &[(
            "DUNGEON_CACHES",
            "npm:/home/dungeon/.cache/npm,uv:/home/dungeon/.cache/uv",
        )],
        cwd_name: "cache-environment-project",
        cwd_entries: &[],
        fs_entries: &[],
    };

    let expected = "podman run -it --userns=keep-id -w /workspace/cache-environment-project --rm --mount type=volume,src=dungeon-cache,dst=/home/dungeon/.cache/npm,subpath=npm --mount type=volume,src=dungeon-cache,dst=/home/dungeon/.cache/uv,subpath=uv -v <CWD>:/workspace/cache-environment-project localhost/dungeon zsh";

    assert_command(input, expected);
}

#[test]
fn expands_project_placeholder_in_environment_values() {
    let input = TestInput {
        toml: r#"
[general]
envs = [
    "OPENCODE_DB=/opencode-db/${project}/opencode.db",
    "REPEATED=${project}-${project}",
    "SHELL=$(basename $PWD)",
    "HOME=${HOME}",
    "INHERITED",
]
"#,
        args: &["run"],
        env: &[],
        cwd_name: "my-project-folder",
        cwd_entries: &[],
        fs_entries: &[],
    };

    let expected = "podman run -it --userns=keep-id -w /workspace/my-project-folder --rm --env OPENCODE_DB=/opencode-db/my-project-folder/opencode.db --env REPEATED=my-project-folder-my-project-folder --env SHELL=$(basename $PWD) --env HOME=${HOME} --env INHERITED -v <CWD>:/workspace/my-project-folder localhost/dungeon zsh";

    assert_command(input, expected);
}

#[test]
fn merges_secrets_from_config_environment_and_cli_without_duplicates() {
    let input = TestInput {
        toml: r#"
[general]
secrets = ["GH_TOKEN", "SHARED", "GH_TOKEN"]
"#,
        args: &["run", "--secret", "SHARED", "--secret", "CLI_SECRET"],
        env: &[("DUNGEON_SECRETS", "SHARED,ENV_SECRET")],
        cwd_name: "secrets-project",
        cwd_entries: &[],
        fs_entries: &[],
    };

    let expected = "podman run -it --userns=keep-id -w /workspace/secrets-project --rm --secret=GH_TOKEN,type=env,target=GH_TOKEN --secret=SHARED,type=env,target=SHARED --secret=ENV_SECRET,type=env,target=ENV_SECRET --secret=CLI_SECRET,type=env,target=CLI_SECRET -v <CWD>:/workspace/secrets-project localhost/dungeon zsh";

    assert_command(input, expected);
}
