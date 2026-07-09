use crate::tests::support::{TestInput, assert_command, resolve_input, try_run_input};

#[test]
fn parses_mount_git_metadata_from_env() {
    let input = TestInput {
        toml: "",
        args: &["run", "--skip-cwd"],
        env: &[("DUNGEON_MOUNT_GIT_METADATA", "true")],
        cwd_name: "git-env-project",
        cwd_entries: &[],
        fs_entries: &[],
    };

    let output = resolve_input(input);
    assert_eq!(output.resolved.settings.mount_git_metadata, Some(true));
}

#[test]
fn disabled_flag_does_not_mount_external_git_metadata() {
    let input = TestInput {
        toml: r#"
[general]
mount_git_metadata = false
"#,
        args: &["run"],
        env: &[],
        cwd_name: "worktree-project",
        cwd_entries: &[],
        fs_entries: &worktree_fs_entries("worktree-project"),
    };

    let expected = "podman run -it --userns=keep-id -w /workspace/worktree-project --rm -v <CWD>:/workspace/worktree-project localhost/dungeon zsh";

    assert_command(input, expected);
}

#[test]
fn normal_git_directory_does_not_add_extra_mount() {
    let input = TestInput {
        toml: r#"
[general]
mount_git_metadata = true
"#,
        args: &["run"],
        env: &[],
        cwd_name: "repo-project",
        cwd_entries: &[".git/"],
        fs_entries: &[],
    };

    let expected = "podman run -it --userns=keep-id -w /workspace/repo-project --rm -v <CWD>:/workspace/repo-project localhost/dungeon zsh";

    assert_command(input, expected);
}

#[test]
fn mounts_common_git_dir_for_worktree() {
    let input = TestInput {
        toml: r#"
[general]
mount_git_metadata = true
"#,
        args: &["run"],
        env: &[],
        cwd_name: "worktree-project",
        cwd_entries: &[],
        fs_entries: &worktree_fs_entries("worktree-project"),
    };

    let expected = "podman run -it --userns=keep-id -w /workspace/worktree-project --rm -v <CWD>:/workspace/worktree-project -v <TMP>/repo/.git:<TMP>/repo/.git localhost/dungeon zsh";

    assert_command(input, expected);
}

#[test]
fn mounts_gitdir_when_commondir_is_missing() {
    let input = TestInput {
        toml: r#"
[general]
mount_git_metadata = true
"#,
        args: &["run"],
        env: &[],
        cwd_name: "detached-project",
        cwd_entries: &[],
        fs_entries: &[
            (
                "detached-project/.git",
                Some("gitdir: <TMP>/git-meta/worktrees/detached-project\n"),
            ),
            ("git-meta/worktrees/detached-project", None),
            (
                "git-meta/worktrees/detached-project/HEAD",
                Some("ref: refs/heads/main\n"),
            ),
        ],
    };

    let expected = "podman run -it --userns=keep-id -w /workspace/detached-project --rm -v <CWD>:/workspace/detached-project -v <TMP>/git-meta/worktrees/detached-project:<TMP>/git-meta/worktrees/detached-project localhost/dungeon zsh";

    assert_command(input, expected);
}

#[test]
fn rejects_relative_gitdir_paths() {
    let input = TestInput {
        toml: r#"
[general]
mount_git_metadata = true
"#,
        args: &["run"],
        env: &[],
        cwd_name: "relative-worktree",
        cwd_entries: &[],
        fs_entries: &[(
            "relative-worktree/.git",
            Some("gitdir: ../repo/.git/worktrees/relative-worktree\n"),
        )],
    };

    let err = try_run_input(input).expect_err("expected relative gitdir error");
    assert!(
        err.to_string()
            .contains("ERROR: relative gitdir paths are unsupported")
    );
}

#[test]
fn rejects_malformed_git_metadata_file() {
    let input = TestInput {
        toml: r#"
[general]
mount_git_metadata = true
"#,
        args: &["run"],
        env: &[],
        cwd_name: "bad-worktree",
        cwd_entries: &[],
        fs_entries: &[("bad-worktree/.git", Some("not a gitdir file\n"))],
    };

    let err = try_run_input(input).expect_err("expected malformed git metadata error");
    assert!(
        err.to_string()
            .contains("ERROR: malformed git metadata file")
    );
}

#[test]
fn explicit_directory_path_mount_adds_git_metadata_mount() {
    let input = TestInput {
        toml: r#"
[general]
mount_git_metadata = true
"#,
        args: &["run", "linked"],
        env: &[],
        cwd_name: "paths-root",
        cwd_entries: &["linked/"],
        fs_entries: &[
            (
                "paths-root/linked/.git",
                Some("gitdir: <TMP>/repo/.git/worktrees/linked\n"),
            ),
            ("repo/.git/worktrees/linked", None),
            ("repo/.git/worktrees/linked/commondir", Some("../..\n")),
            (
                "repo/.git/worktrees/linked/HEAD",
                Some("ref: refs/heads/main\n"),
            ),
            ("repo/.git", None),
        ],
    };

    let expected = "podman run -it --userns=keep-id -w /workspace/project --rm -v <CWD>/linked:/workspace/project/linked -v <TMP>/repo/.git:<TMP>/repo/.git localhost/dungeon zsh";

    assert_command(input, expected);
}

#[test]
fn explicit_file_path_does_not_trigger_git_metadata_mounts() {
    let input = TestInput {
        toml: r#"
[general]
mount_git_metadata = true
"#,
        args: &["run", "linked.txt"],
        env: &[],
        cwd_name: "file-root",
        cwd_entries: &["linked.txt"],
        fs_entries: &[],
    };

    let expected = "podman run -it --userns=keep-id -w /workspace/project --rm -v <CWD>/linked.txt:/workspace/project/linked.txt localhost/dungeon zsh";

    assert_command(input, expected);
}

fn worktree_fs_entries(cwd_name: &str) -> [(&str, Option<&str>); 5] {
    [
        (
            Box::leak(format!("{cwd_name}/.git").into_boxed_str()),
            Some(Box::leak(
                format!("gitdir: <TMP>/repo/.git/worktrees/{cwd_name}\n").into_boxed_str(),
            )),
        ),
        (
            Box::leak(format!("repo/.git/worktrees/{cwd_name}").into_boxed_str()),
            None,
        ),
        (
            Box::leak(format!("repo/.git/worktrees/{cwd_name}/commondir").into_boxed_str()),
            Some("../..\n"),
        ),
        (
            Box::leak(format!("repo/.git/worktrees/{cwd_name}/HEAD").into_boxed_str()),
            Some("ref: refs/heads/main\n"),
        ),
        ("repo/.git", None),
    ]
}
