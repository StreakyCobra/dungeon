use crate::tests::support::{TestInput, resolve_input, try_resolve_input};

#[test]
fn merges_network_settings_across_config_layers() {
    let input = TestInput {
        toml: r#"
[general.network]
ipv6 = false
allow_dns = false
allowed_tcp_domains = ["crates.io"]

[codex.network]
allow_dns = true
allowed_tcp_domains = ["index.crates.io"]
allowed_tcp_hosts = ["10.0.0.0/8"]
"#,
        args: &["run", "--codex", "--allow-host", "2001:db8::/32"],
        env: &[("DUNGEON_NETWORK_ALLOWED_TCP_HOSTS", "192.168.1.10")],
        cwd_name: "network-merge-project",
        cwd_entries: &[],
    };

    let output = resolve_input(input);
    let network = output.resolved.settings.network;

    assert_eq!(network.ipv6, Some(false));
    assert_eq!(network.allow_dns, Some(true));
    assert_eq!(
        network.allowed_tcp_domains,
        Some(vec!["crates.io".to_string(), "index.crates.io".to_string()])
    );
    assert_eq!(
        network.allowed_tcp_hosts,
        Some(vec![
            "10.0.0.0/8".to_string(),
            "192.168.1.10".to_string(),
            "2001:db8::/32".to_string(),
        ])
    );
}

#[test]
fn parses_network_flags_from_cli() {
    let input = TestInput {
        toml: "",
        args: &[
            "run",
            "--network-ipv6",
            "--allow-dns",
            "--allow-domain",
            "crates.io",
            "--allow-host",
            "127.0.0.1",
        ],
        env: &[],
        cwd_name: "network-cli-project",
        cwd_entries: &[],
    };

    let output = resolve_input(input);
    let network = output.resolved.settings.network;

    assert_eq!(network.ipv6, Some(true));
    assert_eq!(network.allow_dns, Some(true));
    assert_eq!(
        network.allowed_tcp_domains,
        Some(vec!["crates.io".to_string()])
    );
    assert_eq!(
        network.allowed_tcp_hosts,
        Some(vec!["127.0.0.1".to_string()])
    );
}

#[test]
fn rejects_invalid_network_domain() {
    let input = TestInput {
        toml: r#"
[general.network]
allowed_tcp_domains = ["bad domain"]
"#,
        args: &["run"],
        env: &[],
        cwd_name: "invalid-network-domain",
        cwd_entries: &[],
    };

    let err = try_resolve_input(input).expect_err("expected invalid domain");
    assert!(
        err.to_string()
            .contains("ERROR: invalid network domain \"bad domain\"")
    );
}

#[test]
fn rejects_invalid_network_host() {
    let input = TestInput {
        toml: "",
        args: &["run", "--allow-host", "not-a-host"],
        env: &[],
        cwd_name: "invalid-network-host",
        cwd_entries: &[],
    };

    let err = try_resolve_input(input).expect_err("expected invalid host");
    assert!(
        err.to_string()
            .contains("ERROR: invalid network host \"not-a-host\"")
    );
}

#[test]
fn rejects_conflicting_network_flags() {
    let input = TestInput {
        toml: "",
        args: &["run", "--network-ipv6", "--network-no-ipv6"],
        env: &[],
        cwd_name: "conflicting-network-flags",
        cwd_entries: &[],
    };

    let err = try_resolve_input(input).expect_err("expected conflicting flag error");
    assert!(
        err.to_string()
            .contains("ERROR: --network-ipv6 and --network-no-ipv6 are mutually exclusive")
    );
}
