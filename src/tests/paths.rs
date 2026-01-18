use crate::tests::support::{assert_command, TestInput};

#[test]
fn mounts_cli_paths_with_custom_names() {
    let input = TestInput {
        toml: "",
        args: &["file1", "folder1"],
        env: &[],
        cwd_name: "paths-project",
        cwd_entries: &["file1", "folder1/"],
    };

    let expected = "podman run -it --userns=keep-id -w /home/dungeon/project --rm -v dungeon-cache:/home/dungeon/.cache -v dungeon-cache:/home/dungeon/.npm -v <CWD>/file1:/home/dungeon/project/file1 -v <CWD>/folder1:/home/dungeon/project/folder1 localhost/dungeon bash";

    assert_command(input, expected);
}

#[test]
fn skips_cwd_mount_when_flagged() {
    let input = TestInput {
        toml: "",
        args: &["--skip-cwd"],
        env: &[],
        cwd_name: "paths-project",
        cwd_entries: &[],
    };

    let expected = "podman run -it --userns=keep-id -w /home/dungeon --rm -v dungeon-cache:/home/dungeon/.cache -v dungeon-cache:/home/dungeon/.npm localhost/dungeon bash";

    assert_command(input, expected);
}
