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
