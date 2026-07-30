use std::net::TcpListener;

use crate::{
    config::Settings,
    container::engine::{build_container_command, reserve_runtime_ports},
    tests::support::{TestInput, resolve_input, try_run_input},
};

const TEST_KEY: &str = "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIFakePublicKey alice@example.com";

#[test]
fn resolves_ssh_settings_from_config_groups_and_environment() {
    let output = resolve_input(TestInput {
        toml: r#"
[general]
enable_ssh = false
ssh_keys = ["ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIFakePublicKey general@example.com"]

[ssh]
enable_ssh = true
ssh_keys = ["ssh-rsa AAAAB3NzaC1yc2EAAAADAQABAAABAQFakePublicKey group@example.com"]
"#,
        args: &["run", "--ssh"],
        env: &[("DUNGEON_SSH_KEYS", TEST_KEY)],
        cwd_name: "ssh-config",
        cwd_entries: &[],
        fs_entries: &[],
    });

    assert_eq!(output.resolved.settings.enable_ssh, Some(true));
    assert_eq!(
        output.resolved.settings.ssh_keys,
        Some(vec![
            "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIFakePublicKey general@example.com".to_string(),
            "ssh-rsa AAAAB3NzaC1yc2EAAAADAQABAAABAQFakePublicKey group@example.com".to_string(),
            TEST_KEY.to_string(),
        ])
    );
}

#[test]
fn rejects_enabled_ssh_without_valid_public_keys() {
    for toml in [
        "[general]\nenable_ssh = true",
        "[general]\nenable_ssh = true\nssh_keys = [\"not a key\"]",
    ] {
        let err = try_run_input(TestInput {
            toml,
            args: &["run"],
            env: &[],
            cwd_name: "invalid-ssh",
            cwd_entries: &[],
            fs_entries: &[],
        })
        .expect_err("invalid SSH configuration");
        assert!(err.to_string().contains("ssh_keys"));
    }
}

#[test]
fn reserves_loopback_ssh_port_and_overrides_generated_environment() {
    let mut settings = Settings {
        enable_ssh: Some(true),
        ssh_keys: Some(vec![TEST_KEY.to_string()]),
        env_vars: Some(vec![
            "DUNGEON_SSH_ENABLED=false".to_string(),
            "DUNGEON_SSH_PORT=22".to_string(),
            "DUNGEON_SSH_KEYS=other-key".to_string(),
        ]),
        ..Settings::default()
    };

    let reservations = reserve_runtime_ports(&mut settings).expect("reserve SSH port");
    let port_spec = settings
        .ports
        .as_deref()
        .and_then(|ports| ports.first())
        .expect("SSH port mapping");
    let port = port_spec
        .strip_prefix("127.0.0.1:")
        .and_then(|value| value.split_once(':'))
        .expect("loopback port mapping");
    assert_eq!(port.0, port.1);
    assert!(TcpListener::bind(format!("127.0.0.1:{}", port.0)).is_err());

    let envs = settings.env_vars.as_deref().expect("SSH environment");
    assert!(envs.contains(&"DUNGEON_SSH_ENABLED=true".to_string()));
    assert!(envs.contains(&format!("DUNGEON_SSH_PORT={}", port.0)));
    assert!(envs.contains(&format!("DUNGEON_SSH_KEYS={TEST_KEY}")));

    let spec = build_container_command(&settings, &[], false).expect("container command");
    assert!(
        spec.args
            .windows(2)
            .any(|pair| { pair == ["-p", port_spec] })
    );

    drop(reservations);
    assert!(TcpListener::bind(format!("127.0.0.1:{}", port.0)).is_ok());
}
