use std::env;

use crate::{
    config::{Engine, Settings},
    container::persist::{
        PersistMode, persisted_container_name, resolve_container_name, sanitize_container_base,
        validate_container_name,
    },
    tests::support::{TestInput, acquire_test_lock, run_input},
};

#[test]
fn persist_run_adds_container_name_and_omits_rm() {
    let input = TestInput {
        toml: "",
        args: &["run", "--persist"],
        env: &[],
        cwd_name: "persist-project",
        cwd_entries: &[],
    };

    let output = run_input(input);
    assert!(output.command.contains(" --name dungeon-"));
    assert!(!output.command.contains(" --rm "));
}

#[test]
fn persisted_name_changes_when_paths_change() {
    let _guard = cwd_lock();
    let temp_dir = tempfile::tempdir().expect("tempdir");
    let cwd = temp_dir.path().join("workspace");
    let dep = cwd.join("dep");

    std::fs::create_dir_all(&dep).expect("create dep");

    let original = env::current_dir().expect("current dir");
    env::set_current_dir(&cwd).expect("set cwd");

    let no_paths = persisted_container_name(&[]).expect("name without paths");
    let with_path = persisted_container_name(&["dep".to_string()]).expect("name with path");

    env::set_current_dir(original).expect("restore cwd");

    assert_ne!(no_paths, with_path);
}

#[test]
fn persisted_name_errors_on_nonexistent_paths() {
    let _guard = cwd_lock();
    let temp_dir = tempfile::tempdir().expect("tempdir");
    let cwd = temp_dir.path().join("workspace");
    std::fs::create_dir_all(&cwd).expect("create cwd");

    let original = env::current_dir().expect("current dir");
    env::set_current_dir(&cwd).expect("set cwd");

    let result = persisted_container_name(&["missing-path".to_string()]);

    env::set_current_dir(original).expect("restore cwd");

    assert!(result.is_err());
}

#[test]
fn resolve_container_name_ignores_paths_for_reuse_and_discard() {
    let _guard = cwd_lock();
    let temp_dir = tempfile::tempdir().expect("tempdir");
    let cwd = temp_dir.path().join("workspace");
    let dep = cwd.join("dep");

    std::fs::create_dir_all(&dep).expect("create dep");

    let original = env::current_dir().expect("current dir");
    env::set_current_dir(&cwd).expect("set cwd");

    let reuse_no_paths = resolve_container_name(PersistMode::Reuse, &[]).expect("reuse no paths");
    let reuse_with_paths =
        resolve_container_name(PersistMode::Reuse, &["dep".to_string()]).expect("reuse paths");
    let discard_no_paths =
        resolve_container_name(PersistMode::Discard, &[]).expect("discard no paths");

    env::set_current_dir(original).expect("restore cwd");

    assert_eq!(reuse_no_paths, reuse_with_paths);
    assert_eq!(reuse_no_paths, discard_no_paths);
}

#[test]
fn sanitize_container_base_replaces_invalid_and_truncates() {
    assert_eq!(sanitize_container_base(" hello/world "), "hello-world");
    assert_eq!(sanitize_container_base("-----"), "project");

    let long = "abcdefghijklmnopqrstuvwxyz0123456789";
    let sanitized = sanitize_container_base(long);
    assert_eq!(sanitized, "abcdefghijklmnopqrstuvwxyz012345");
}

#[test]
fn validate_container_name_rejects_empty_and_too_long_names() {
    let empty = validate_container_name("   ");
    assert!(empty.is_err());

    let too_long = validate_container_name(&"a".repeat(64));
    assert!(too_long.is_err());

    let valid = validate_container_name("dungeon-project-123");
    assert!(valid.is_ok());
}

#[test]
fn engine_binary_mapping_is_stable() {
    assert_eq!(Engine::Podman.binary(), "podman");
}

#[test]
fn persist_lookup_uses_podman_args_before_subcommand() {
    let settings = Settings {
        podman_args: Some(vec!["-c".to_string(), "agent-vm".to_string()]),
        ..Settings::default()
    };

    let spec = crate::container::engine::build_podman_command(
        &settings,
        vec![
            "container".to_string(),
            "exists".to_string(),
            "demo".to_string(),
        ],
    );

    assert_eq!(
        format!("{} {}", spec.program, spec.args.join(" ")),
        "podman -c agent-vm container exists demo"
    );
}

fn cwd_lock() -> std::sync::MutexGuard<'static, ()> {
    acquire_test_lock()
}
