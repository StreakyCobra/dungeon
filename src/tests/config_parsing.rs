use crate::tests::support::{TestInput, try_run_input};

#[test]
fn errors_on_unknown_general_key() {
    let input = TestInput {
        toml: r#"
[general]
unknown = "value"
"#,
        args: &["run"],
        env: &[],
        cwd_name: "unknown-general-key",
        cwd_entries: &[],
        fs_entries: &[],
    };

    assert_input_error_contains(input, "[general] has unknown key \"unknown\"");
}

#[test]
fn rejects_replaced_always_on_groups_key() {
    let input = TestInput {
        toml: r#"
[general]
always_on_groups = ["codex"]
"#,
        args: &["run"],
        env: &[],
        cwd_name: "replaced-general-key",
        cwd_entries: &[],
        fs_entries: &[],
    };

    assert_input_error_contains(input, "[general] has unknown key \"always_on_groups\"");
}

#[test]
fn errors_on_non_string_general_engine() {
    let input = TestInput {
        toml: r#"
[general]
engine = 1
"#,
        args: &["run"],
        env: &[],
        cwd_name: "non-string-engine",
        cwd_entries: &[],
        fs_entries: &[],
    };

    assert_input_error_contains(input, "general.engine must be a string");
}

#[test]
fn errors_on_non_list_ports() {
    let input = TestInput {
        toml: r#"
[general]
ports = "127.0.0.1:3000:3000"
"#,
        args: &["run"],
        env: &[],
        cwd_name: "non-list-ports",
        cwd_entries: &[],
        fs_entries: &[],
    };

    assert_input_error_contains(input, "general.ports must be a list of strings");
}

#[test]
fn errors_on_non_string_port_entries() {
    let input = TestInput {
        toml: r#"
[general]
ports = [1234]
"#,
        args: &["run"],
        env: &[],
        cwd_name: "non-string-port-entry",
        cwd_entries: &[],
        fs_entries: &[],
    };

    assert_input_error_contains(input, "general.ports must be a list of strings");
}

#[test]
fn errors_on_non_string_exposed_host_port_entries() {
    let input = TestInput {
        toml: r#"
[general]
expose_host_ports = [8080]
"#,
        args: &["run"],
        env: &[],
        cwd_name: "non-string-exposed-host-port",
        cwd_entries: &[],
        fs_entries: &[],
    };

    assert_input_error_contains(input, "general.expose_host_ports must be a list of strings");
}

#[test]
fn errors_on_unknown_group_key() {
    let input = TestInput {
        toml: r#"
[workspace]
unknown = "value"
"#,
        args: &["run"],
        env: &[],
        cwd_name: "unknown-group-key",
        cwd_entries: &[],
        fs_entries: &[],
    };

    assert_input_error_contains(input, "group \"workspace\" has unknown key \"unknown\"");
}

#[test]
fn errors_on_invalid_engine_from_env() {
    let input = TestInput {
        toml: "",
        args: &["run"],
        env: &[("DUNGEON_ENGINE", "invalid")],
        cwd_name: "invalid-env-engine",
        cwd_entries: &[],
        fs_entries: &[],
    };

    assert_input_error_contains(input, "engine must be one of: podman");
}

fn assert_input_error_contains(input: TestInput<'_>, expected_substring: &str) {
    let err = match try_run_input(input) {
        Ok(_) => panic!("expected input to fail"),
        Err(err) => err,
    };
    let message = err.to_string();
    assert!(
        message.contains(expected_substring),
        "expected error containing '{expected_substring}', got '{message}'"
    );
}
