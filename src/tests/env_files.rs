use crate::tests::support::{assert_command, TestInput};

#[test]
fn includes_env_and_env_files() {
    let input = TestInput {
        toml: r#"
run = "echo ok"
env_files = [".env", "config.env"]
"#,
        args: &[],
        env: &[("DUNGEON_ENVS", "FOO=bar")],
        cwd_name: "env-project",
        cwd_entries: &[],
    };

    let expected = "podman run -it --userns=keep-id -w /home/dungeon/env-project --rm --env FOO=bar --env-file .env --env-file config.env -v dungeon-cache:/home/dungeon/.cache -v dungeon-cache:/home/dungeon/.npm -v <CWD>:/home/dungeon/env-project localhost/dungeon bash -ic echo ok";

    assert_command(input, expected);
}
