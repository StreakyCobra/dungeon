use crate::tests::support::{TestInput, assert_command, run_input};

#[test]
fn docker_engine_uses_host_uid_gid() {
    let input = TestInput {
        toml: "",
        args: &["--engine", "docker"],
        env: &[],
        cwd_name: "docker-project",
        cwd_entries: &[],
    };

    let expected = "docker run -it --user <UID>:<GID> -w /home/dungeon/docker-project --rm -v <CWD>:/home/dungeon/docker-project localhost/dungeon bash";

    assert_command(input, expected);
}

#[test]
fn engine_from_env_overrides_default() {
    let input = TestInput {
        toml: "",
        args: &[],
        env: &[("DUNGEON_ENGINE", "docker")],
        cwd_name: "engine-env-project",
        cwd_entries: &[],
    };

    let expected = "docker run -it --user <UID>:<GID> -w /home/dungeon/engine-env-project --rm -v <CWD>:/home/dungeon/engine-env-project localhost/dungeon bash";

    assert_command(input, expected);
}

#[test]
fn engine_from_config_overrides_default() {
    let input = TestInput {
        toml: "engine = 'docker'",
        args: &[],
        env: &[],
        cwd_name: "engine-config-project",
        cwd_entries: &[],
    };

    let expected = "docker run -it --user <UID>:<GID> -w /home/dungeon/engine-config-project --rm -v <CWD>:/home/dungeon/engine-config-project localhost/dungeon bash";

    assert_command(input, expected);
}

#[test]
fn passes_engine_args_from_config_and_cli() {
    let input = TestInput {
        toml: r#"
engine_args = ["--network=host"]
"#,
        args: &["--engine-arg=--security-opt=label=disable"],
        env: &[],
        cwd_name: "engine-args-project",
        cwd_entries: &[],
    };

    let expected = "podman run -it --userns=keep-id -w /home/dungeon/engine-args-project --rm --network=host --security-opt=label=disable -v <CWD>:/home/dungeon/engine-args-project localhost/dungeon bash";

    assert_command(input, expected);
}

#[test]
fn errors_on_invalid_engine_value() {
    let input = TestInput {
        toml: "engine = 'invalid'",
        args: &[],
        env: &[],
        cwd_name: "invalid-engine",
        cwd_entries: &[],
    };

    let result = std::panic::catch_unwind(|| run_input(input));
    assert!(result.is_err());
}
