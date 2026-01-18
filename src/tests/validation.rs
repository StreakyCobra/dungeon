use crate::tests::support::{run_input, TestInput};

#[test]
fn errors_on_unknown_config_keys() {
    let input = TestInput {
        toml: "unknown = 'value'",
        args: &[],
        env: &[],
        cwd_name: "unknown-config",
        cwd_entries: &[],
    };

    let result = std::panic::catch_unwind(|| run_input(input));
    assert!(result.is_err());
}

#[test]
fn errors_when_skip_cwd_with_paths() {
    let input = TestInput {
        toml: "",
        args: &["--skip-cwd", "folder1"],
        env: &[],
        cwd_name: "skip-cwd-paths",
        cwd_entries: &["folder1/"],
    };

    let result = std::panic::catch_unwind(|| run_input(input));
    assert!(result.is_err());
}
