use std::net::TcpListener;

use crate::{
    config::Settings,
    container::engine::reserve_dynamic_ports,
    tests::support::{TestInput, try_run_input},
};

#[test]
fn reserves_deduped_dynamic_ports_and_generated_envs_override_configured_values() {
    let mut settings = Settings {
        dynamic_ports: Some(vec![
            "difit".to_string(),
            "difit".to_string(),
            "api_2".to_string(),
        ]),
        ports: Some(vec!["127.0.0.1:3000:3000".to_string()]),
        env_vars: Some(vec!["DUNGEON_PORT_FOR_DIFIT=9999".to_string()]),
        ..Settings::default()
    };

    let reservations = reserve_dynamic_ports(&mut settings).expect("reserve dynamic ports");
    let ports = settings.ports.expect("ports");
    let envs = settings.env_vars.expect("envs");

    assert_eq!(ports.len(), 3);
    assert_eq!(envs.len(), 2);
    assert!(envs[0].starts_with("DUNGEON_PORT_FOR_DIFIT="));
    assert!(envs[1].starts_with("DUNGEON_PORT_FOR_API_2="));
    for (port_spec, env) in ports[1..].iter().zip(&envs) {
        let port = port_spec
            .strip_prefix("127.0.0.1:")
            .and_then(|value| value.split_once(':'))
            .expect("dynamic port spec");
        assert_eq!(port.0, port.1);
        assert!(port.0.parse::<u16>().is_ok());
        assert!(env.ends_with(port.0));
    }
    let first_dynamic_port = ports[1]["127.0.0.1:".len()..]
        .split_once(':')
        .expect("port")
        .0;
    let address = format!("127.0.0.1:{first_dynamic_port}");
    assert!(TcpListener::bind(&address).is_err());

    drop(reservations);
    assert!(TcpListener::bind(address).is_ok());
}

#[test]
fn dynamic_ports_merge_from_general_and_groups() {
    let input = TestInput {
        toml: r#"
[general]
dynamic_ports = ["difit"]

[review]
dynamic_ports = ["difit", "api"]
"#,
        args: &["run", "--review"],
        env: &[],
        cwd_name: "dynamic-ports",
        cwd_entries: &[],
        fs_entries: &[],
    };

    let mut settings = crate::tests::support::resolve_input(input)
        .resolved
        .settings;
    let _reservations = reserve_dynamic_ports(&mut settings).expect("reserve dynamic ports");
    assert_eq!(settings.ports.as_ref().expect("ports").len(), 2);
    assert_eq!(settings.env_vars.as_ref().expect("envs").len(), 2);
}

#[test]
fn dynamic_ports_merge_from_config_groups_environment_and_cli() {
    let input = TestInput {
        toml: r#"
[general]
dynamic_ports = ["general"]

[review]
dynamic_ports = ["group", "general"]
"#,
        args: &[
            "run",
            "--review",
            "--dynamic-port",
            "cli",
            "--dynamic-port",
            "general",
        ],
        env: &[("DUNGEON_DYNAMIC_PORTS", "env,general")],
        cwd_name: "dynamic-port-precedence",
        cwd_entries: &[],
        fs_entries: &[],
    };

    let settings = crate::tests::support::resolve_input(input)
        .resolved
        .settings;
    assert_eq!(
        settings.dynamic_ports,
        Some(vec![
            "general".to_string(),
            "group".to_string(),
            "general".to_string(),
            "env".to_string(),
            "general".to_string(),
            "cli".to_string(),
            "general".to_string(),
        ])
    );
}

#[test]
fn rejects_invalid_dynamic_port_names() {
    let input = TestInput {
        toml: r#"
[general]
dynamic_ports = ["Difit"]
"#,
        args: &["run"],
        env: &[],
        cwd_name: "invalid-dynamic-port",
        cwd_entries: &[],
        fs_entries: &[],
    };

    let err = try_run_input(input).expect_err("invalid dynamic port name");
    assert!(err.to_string().contains("lower-case ASCII identifiers"));
}

#[test]
fn rejects_invalid_dynamic_port_names_from_environment_and_cli() {
    for (args, env) in [
        (
            vec!["run", "--dynamic-port", "Difit"],
            Vec::<(&str, &str)>::new(),
        ),
        (vec!["run"], vec![("DUNGEON_DYNAMIC_PORTS", "Difit")]),
    ] {
        let input = TestInput {
            toml: "",
            args: &args,
            env: &env,
            cwd_name: "invalid-dynamic-port-input",
            cwd_entries: &[],
            fs_entries: &[],
        };

        let err = try_run_input(input).expect_err("invalid dynamic port name");
        assert!(err.to_string().contains("lower-case ASCII identifiers"));
    }
}
