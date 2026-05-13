use crate::tests::support::{TestInput, assert_command, run_input};

#[test]
fn includes_bootstrap_security_args() {
    let input = TestInput {
        toml: "",
        args: &["run"],
        env: &[],
        cwd_name: "security-args-project",
        cwd_entries: &[],
    };

    let output = run_input(input);

    for fragment in [
        "--user root",
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
            output.command.contains(fragment),
            "expected command to contain {fragment}: {}",
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
    };

    let expected = "podman run -it --userns=keep-id -w /workspace/command-flag-project --rm -v <CWD>:/workspace/command-flag-project localhost/dungeon bash -ic echo ok";

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

    let expected = "podman run -it --userns=keep-id -w /workspace/command-env-project --rm -v <CWD>:/workspace/command-env-project localhost/dungeon bash -ic echo env";

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

    let expected = "podman run -it --userns=keep-id -w /workspace/engine-env-project --rm -v <CWD>:/workspace/engine-env-project localhost/dungeon bash";

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

    let expected = "podman run -it --userns=keep-id -w /workspace/engine-config-project --rm -v <CWD>:/workspace/engine-config-project localhost/dungeon bash";

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
    };

    let expected = "podman run -it --userns=keep-id -w /workspace/run-args-project --rm --network=host --security-opt=label=disable -v <CWD>:/workspace/run-args-project localhost/dungeon bash";

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

    let expected = "podman run -it --userns=keep-id -w /workspace/blank-command-project --rm -v <CWD>:/workspace/blank-command-project localhost/dungeon bash";

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

    let expected = "podman run -it --userns=keep-id -w /workspace/blank-image-project --rm -v <CWD>:/workspace/blank-image-project localhost/dungeon bash";

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

    let expected = "podman run -it --userns=keep-id -w /workspace/blank-env-values --rm --env FOO=bar --env-file ./.env -v <CWD>:/workspace/blank-env-values localhost/dungeon bash";

    assert_command(input, expected);
}

#[test]
fn includes_network_env_for_non_default_settings() {
    let input = TestInput {
        toml: "",
        args: &[
            "run",
            "--ipv6",
            "--allow-dns",
            "--allow-domain",
            "crates.io",
            "--allow-host",
            "127.0.0.1",
        ],
        env: &[],
        cwd_name: "network-env-project",
        cwd_entries: &[],
    };

    let output = run_input(input);

    for fragment in [
        "--env DUNGEON_IPV6=1",
        "--env DUNGEON_ALLOWED_TCP_DOMAINS=crates.io",
        "--env DUNGEON_ALLOWED_TCP_HOSTS=127.0.0.1",
    ] {
        assert!(
            output.command.contains(fragment),
            "expected command to contain {fragment}: {}",
            output.command
        );
    }
}
