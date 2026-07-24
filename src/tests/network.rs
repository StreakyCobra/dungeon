use crate::tests::support::{TestInput, assert_command, resolve_input, try_resolve_input};

#[test]
fn merges_exposed_host_ports_across_config_layers() {
    let input = TestInput {
        toml: r#"
[general]
expose_host_ports = ["8080"]

[host-tools]
expose_host_ports = ["18080:8080"]
"#,
        args: &[
            "run",
            "--host-tools",
            "--expose-host-port",
            "10000-10010:20000-20010",
        ],
        env: &[("DUNGEON_EXPOSE_HOST_PORTS", "5432,6379")],
        cwd_name: "exposed-host-port-merge",
        cwd_entries: &[],
        fs_entries: &[],
    };

    let output = resolve_input(input);
    assert_eq!(
        output.resolved.settings.expose_host_ports,
        Some(vec![
            "8080".to_string(),
            "18080:8080".to_string(),
            "5432".to_string(),
            "6379".to_string(),
            "10000-10010:20000-20010".to_string(),
        ])
    );
}

#[test]
fn generates_pasta_reverse_tcp_forwarding() {
    let input = TestInput {
        toml: r#"
[general]
expose_host_ports = ["8080", "18080:8080", "8000-8010"]
"#,
        args: &["run"],
        env: &[],
        cwd_name: "exposed-host-port-command",
        cwd_entries: &[],
        fs_entries: &[],
    };

    let expected = "podman run -it --userns=keep-id -w /workspace/exposed-host-port-command --rm --network=pasta:-T,8080,-T,18080:8080,-T,8000-8010 -v <CWD>:/workspace/exposed-host-port-command localhost/dungeon zsh";

    assert_command(input, expected);
}

#[test]
fn rejects_invalid_exposed_host_port_specifications() {
    for spec in [
        "",
        "0",
        "65536",
        "8080:0",
        "9000-8000",
        "8000-8010:9000-9005",
        "80:90:100",
        "all",
        "auto",
        "~80",
        "127.0.0.1/80",
    ] {
        let input = TestInput {
            toml: "",
            args: &["run", "--expose-host-port", spec],
            env: &[],
            cwd_name: "invalid-exposed-host-port",
            cwd_entries: &[],
            fs_entries: &[],
        };

        let err = try_resolve_input(input).expect_err("expected invalid exposed host port");
        assert!(
            err.to_string()
                .contains("ERROR: invalid exposed host port specification"),
            "unexpected error for {spec:?}: {err}"
        );
    }
}

#[test]
fn rejects_exposed_host_ports_with_explicit_network() {
    for run_args in [
        r#"run_args = ["--network=host"]"#,
        r#"run_args = ["--net", "host"]"#,
    ] {
        let toml = format!(
            r#"
[general]
expose_host_ports = ["8080"]
{run_args}
"#
        );
        let input = TestInput {
            toml: &toml,
            args: &["run"],
            env: &[],
            cwd_name: "conflicting-exposed-host-port-network",
            cwd_entries: &[],
            fs_entries: &[],
        };

        let err = try_resolve_input(input).expect_err("expected explicit network conflict");
        assert!(err.to_string().contains(
            "ERROR: expose_host_ports cannot be combined with --network or --net in run_args"
        ));
    }
}
