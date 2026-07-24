use crate::tests::support::{TestInput, assert_command, run_input};

#[test]
fn uses_root_only_for_the_runtime_user_bootstrap() {
    let input = TestInput {
        toml: "",
        args: &["run"],
        env: &[],
        cwd_name: "security-args-project",
        cwd_entries: &[],
        fs_entries: &[],
    };

    let output = run_input(input);

    assert!(output.command.contains("--user root"));
    for fragment in [
        "--cap-add NET_ADMIN",
        "--cap-add NET_RAW",
        "--cap-add SYS_ADMIN",
        "--cap-add SYS_CHROOT",
        "--cap-add SETUID",
        "--cap-add SETGID",
        "--cap-add SYS_PTRACE",
        "--security-opt seccomp=unconfined",
    ] {
        assert!(
            !output.command.contains(fragment),
            "expected command to omit {fragment}: {}",
            output.command
        );
    }
}

#[test]
fn command_flag_runs_command_in_container() {
    let input = TestInput {
        toml: "",
        args: &["run", "--command", "echo ok"],
        env: &[],
        cwd_name: "command-flag-project",
        cwd_entries: &[],
        fs_entries: &[],
    };

    let expected = "podman run -it --userns=keep-id -w /workspace/command-flag-project --rm -v <CWD>:/workspace/command-flag-project localhost/dungeon zsh -ic echo ok";

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
        fs_entries: &[],
    };

    let expected = "podman run -it --userns=keep-id -w /workspace/command-env-project --rm -v <CWD>:/workspace/command-env-project localhost/dungeon zsh -ic echo env";

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
        fs_entries: &[],
    };

    let expected = "podman run -it --userns=keep-id -w /workspace/engine-env-project --rm -v <CWD>:/workspace/engine-env-project localhost/dungeon zsh";

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
        fs_entries: &[],
    };

    let expected = "podman run -it --userns=keep-id -w /workspace/engine-config-project --rm -v <CWD>:/workspace/engine-config-project localhost/dungeon zsh";

    assert_command(input, expected);
}

#[test]
fn passes_run_args_from_config_and_cli() {
    let input = TestInput {
        toml: r#"
[general]
run_args = ["--network=host"]
"#,
        args: &["run", "--run-arg=--security-opt=label=disable"],
        env: &[],
        cwd_name: "run-args-project",
        cwd_entries: &[],
        fs_entries: &[],
    };

    let expected = "podman run -it --userns=keep-id -w /workspace/run-args-project --rm --network=host --security-opt=label=disable -v <CWD>:/workspace/run-args-project localhost/dungeon zsh";

    assert_command(input, expected);
}

#[test]
fn passes_krun_runtime_as_a_run_arg() {
    let input = TestInput {
        toml: r#"
[general]
run_args = ["--runtime=krun"]
"#,
        args: &["run"],
        env: &[],
        cwd_name: "krun-project",
        cwd_entries: &[],
        fs_entries: &[],
    };

    let expected = "podman run -it --userns=keep-id -w /workspace/krun-project --rm --runtime=krun -v <CWD>:/workspace/krun-project localhost/dungeon zsh";

    assert_command(input, expected);
}

#[test]
fn passes_run_args_with_space_separated_hyphen_value() {
    let input = TestInput {
        toml: "",
        args: &["run", "--run-arg", "--network=host"],
        env: &[],
        cwd_name: "run-args-space-project",
        cwd_entries: &[],
        fs_entries: &[],
    };

    let expected = "podman run -it --userns=keep-id -w /workspace/run-args-space-project --rm --network=host -v <CWD>:/workspace/run-args-space-project localhost/dungeon zsh";

    assert_command(input, expected);
}

#[test]
fn passes_podman_args_before_run_subcommand() {
    let input = TestInput {
        toml: r#"
[general]
podman_args = ["-c", "agent-vm"]
"#,
        args: &["run", "--podman-arg=--log-level=debug"],
        env: &[],
        cwd_name: "podman-args-project",
        cwd_entries: &[],
        fs_entries: &[],
    };

    let expected = "podman -c agent-vm --log-level=debug run -it --userns=keep-id -w /workspace/podman-args-project --rm -v <CWD>:/workspace/podman-args-project localhost/dungeon zsh";

    assert_command(input, expected);
}

#[test]
fn passes_podman_args_with_space_separated_hyphen_value() {
    let input = TestInput {
        toml: "",
        args: &["run", "--podman-arg", "--log-level=debug"],
        env: &[],
        cwd_name: "podman-args-space-project",
        cwd_entries: &[],
        fs_entries: &[],
    };

    let expected = "podman --log-level=debug run -it --userns=keep-id -w /workspace/podman-args-space-project --rm -v <CWD>:/workspace/podman-args-space-project localhost/dungeon zsh";

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
        fs_entries: &[],
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
        fs_entries: &[],
    };

    let expected = "podman run -it --userns=keep-id -w /workspace/blank-command-project --rm -v <CWD>:/workspace/blank-command-project localhost/dungeon zsh";

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
        fs_entries: &[],
    };

    let expected = "podman run -it --userns=keep-id -w /workspace/blank-image-project --rm -v <CWD>:/workspace/blank-image-project localhost/dungeon zsh";

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
        fs_entries: &[],
    };

    let expected = "podman run -it --userns=keep-id -w /workspace/blank-env-values --rm --env FOO=bar --env-file ./.env -v <CWD>:/workspace/blank-env-values localhost/dungeon zsh";

    assert_command(input, expected);
}
