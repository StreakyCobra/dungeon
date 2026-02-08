use crate::tests::support::{TestInput, assert_command, run_input};

#[test]
fn basic_run_uses_cwd_mount() {
    let input = TestInput {
        toml: "",
        args: &["run"],
        env: &[],
        cwd_name: "alpha",
        cwd_entries: &[],
    };

    let expected = "podman run -it --userns=keep-id -w /home/dungeon/alpha --rm -v <CWD>:/home/dungeon/alpha localhost/dungeon bash";

    assert_command(input, expected);
}

#[test]
fn basic_run_errors_from_home_dir() {
    let input = TestInput {
        toml: "",
        args: &["run"],
        env: &[],
        cwd_name: "home",
        cwd_entries: &[],
    };

    let result = std::panic::catch_unwind(|| run_input(input));
    assert!(result.is_err());
}

#[test]
fn skip_cwd_allows_home_dir_run() {
    let input = TestInput {
        toml: "",
        args: &["run", "--skip-cwd"],
        env: &[],
        cwd_name: "home",
        cwd_entries: &[],
    };

    let expected = "podman run -it --userns=keep-id -w /home/dungeon --rm localhost/dungeon bash";

    assert_command(input, expected);
}
