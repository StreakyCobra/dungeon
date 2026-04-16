use crate::tests::support::{TestInput, assert_command, run_input};

#[test]
fn command_flag_runs_command_in_container() {
    let input = TestInput {
        toml: "",
        args: &["run", "--command", "echo ok"],
        env: &[],
        cwd_name: "command-flag-project",
        cwd_entries: &[],
    };

    let expected = "podman run -it --userns=keep-id -w /home/dungeon/command-flag-project --rm -v <CWD>:/home/dungeon/command-flag-project localhost/dungeon bash -ic echo ok";

    assert_command(input, expected);
}

#[test]
fn command_from_env_runs_command_in_container() {
    let input = TestInput {
        toml: "",
        args: &["run"],
        env: &[("DUNGEON_COMMAND", "echo env")],
        cwd_name: "command-env-project",
        cwd_entries: &[],
    };

    let expected = "podman run -it --userns=keep-id -w /home/dungeon/command-env-project --rm -v <CWD>:/home/dungeon/command-env-project localhost/dungeon bash -ic echo env";

    assert_command(input, expected);
}

#[test]
fn engine_from_env_accepts_podman() {
    let input = TestInput {
        toml: "",
        args: &["run"],
        env: &[("DUNGEON_ENGINE", "podman")],
        cwd_name: "engine-env-project",
        cwd_entries: &[],
    };

    let expected = "podman run -it --userns=keep-id -w /home/dungeon/engine-env-project --rm -v <CWD>:/home/dungeon/engine-env-project localhost/dungeon bash";

    assert_command(input, expected);
}

#[test]
fn engine_from_config_accepts_podman() {
    let input = TestInput {
        toml: r#"
[general]
engine = "podman"
"#,
        args: &["run"],
        env: &[],
        cwd_name: "engine-config-project",
        cwd_entries: &[],
    };

    let expected = "podman run -it --userns=keep-id -w /home/dungeon/engine-config-project --rm -v <CWD>:/home/dungeon/engine-config-project localhost/dungeon bash";

    assert_command(input, expected);
}

#[test]
fn passes_engine_args_from_config_and_cli() {
    let input = TestInput {
        toml: r#"
[general]
engine_args = ["--network=host"]
"#,
        args: &["run", "--engine-arg=--security-opt=label=disable"],
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
        toml: r#"
[general]
engine = "invalid"
"#,
        args: &["run"],
        env: &[],
        cwd_name: "invalid-engine",
        cwd_entries: &[],
    };

    let result = std::panic::catch_unwind(|| run_input(input));
    assert!(result.is_err());
}

#[test]
fn blank_command_does_not_append_shell_exec_flag() {
    let input = TestInput {
        toml: r#"
[general]
command = "   "
"#,
        args: &["run"],
        env: &[],
        cwd_name: "blank-command-project",
        cwd_entries: &[],
    };

    let expected = "podman run -it --userns=keep-id -w /home/dungeon/blank-command-project --rm -v <CWD>:/home/dungeon/blank-command-project localhost/dungeon bash";

    assert_command(input, expected);
}

#[test]
fn blank_image_falls_back_to_default_image() {
    let input = TestInput {
        toml: r#"
[general]
image = ""
"#,
        args: &["run"],
        env: &[],
        cwd_name: "blank-image-project",
        cwd_entries: &[],
    };

    let expected = "podman run -it --userns=keep-id -w /home/dungeon/blank-image-project --rm -v <CWD>:/home/dungeon/blank-image-project localhost/dungeon bash";

    assert_command(input, expected);
}

#[test]
fn skips_blank_env_and_env_file_values() {
    let input = TestInput {
        toml: r#"
[general]
envs = [" ", "FOO=bar", ""]
env_files = ["", "  ", "./.env"]
"#,
        args: &["run"],
        env: &[],
        cwd_name: "blank-env-values",
        cwd_entries: &[],
    };

    let expected = "podman run -it --userns=keep-id -w /home/dungeon/blank-env-values --rm --env FOO=bar --env-file ./.env -v <CWD>:/home/dungeon/blank-env-values localhost/dungeon bash";

    assert_command(input, expected);
}
