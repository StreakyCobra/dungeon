use crate::{
    cli,
    config,
    error::AppError,
    tests::support::{run_input, TestInput},
};

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
fn errors_on_invalid_env_spec() {
    let input = TestInput {
        toml: "",
        args: &["--env", "=oops"],
        env: &[],
        cwd_name: "bad-env",
        cwd_entries: &[],
    };

    let result = std::panic::catch_unwind(|| run_input(input));
    assert!(result.is_err());
}

#[test]
fn errors_on_invalid_port_spec() {
    let input = TestInput {
        toml: "",
        args: &["--port", "123"],
        env: &[],
        cwd_name: "bad-port",
        cwd_entries: &[],
    };

    let result = std::panic::catch_unwind(|| run_input(input));
    assert!(result.is_err());
}

#[test]
fn errors_on_empty_image_flag() {
    let defaults = config::Config::default();
    let env_cfg = config::Config::default();
    let file_cfg = config::Config::default();

    let argv = vec!["--image".to_string(), " ".to_string()];
    let result = cli::parse_args_with_sources(argv, defaults, file_cfg, env_cfg);
    match result {
        Err(AppError::Message(msg)) => assert!(msg.contains("--image")),
        _ => panic!("expected image validation error"),
    }
}
