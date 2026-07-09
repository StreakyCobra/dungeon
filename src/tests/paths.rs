use crate::tests::support::{TestInput, assert_command};

#[test]
fn mounts_cli_paths_with_custom_names() {
    let input = TestInput {
        toml: "",
        args: &["run", "file1", "folder1"],
        env: &[],
        cwd_name: "paths-project",
        cwd_entries: &["file1", "folder1/"],
        fs_entries: &[],
    };

    let expected = "podman run -it --userns=keep-id -w /workspace/project --rm -v <CWD>/file1:/workspace/project/file1 -v <CWD>/folder1:/workspace/project/folder1 localhost/dungeon zsh";

    assert_command(input, expected);
}

#[test]
fn skips_cwd_mount_when_flagged() {
    let input = TestInput {
        toml: "",
        args: &["run", "--skip-cwd"],
        env: &[],
        cwd_name: "paths-project",
        cwd_entries: &[],
        fs_entries: &[],
    };

    let expected = "podman run -it --userns=keep-id -w /workspace --rm localhost/dungeon zsh";

    assert_command(input, expected);
}

#[test]
fn mounts_nonexistent_explicit_paths_without_validation() {
    let input = TestInput {
        toml: "",
        args: &["run", "missing-file.txt"],
        env: &[],
        cwd_name: "paths-project",
        cwd_entries: &[],
        fs_entries: &[],
    };

    let expected = "podman run -it --userns=keep-id -w /workspace/project --rm -v <CWD>/missing-file.txt:/workspace/project/missing-file.txt localhost/dungeon zsh";

    assert_command(input, expected);
}
