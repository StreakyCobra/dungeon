use crate::tests::support::{TestInput, assert_command};

#[test]
fn merges_mounts_ports_and_cache() {
    let input = TestInput {
        toml: r#"
[general]
ports = ["127.0.0.1:3000:3000"]
caches = ["/var/cache/pacman/pkg:ro"]
mounts = ["~/data:/data:ro"]
"#,
        args: &[
            "run",
            "--port",
            "127.0.0.1:8080:8080",
            "--cache",
            "deps:rw",
            "--mount",
            "$HOME/.codex:/home/dungeon/.codex:rw",
        ],
        env: &[],
        cwd_name: "mounts-root",
        cwd_entries: &[],
        fs_entries: &[],
    };

    let expected = "podman run -it --userns=keep-id -w /workspace/mounts-root --rm -p 127.0.0.1:3000:3000 -p 127.0.0.1:8080:8080 -v <HOME>/data:/data:ro -v <HOME>/.codex:/home/dungeon/.codex:rw -v dungeon-cache:/var/cache/pacman/pkg:ro -v dungeon-cache:deps:rw -v <CWD>:/workspace/mounts-root localhost/dungeon zsh";

    assert_command(input, expected);
}

#[test]
fn pi_group_mounts_agent_dir() {
    let input = TestInput {
        toml: "",
        args: &["run", "--pi", "--command", "pi"],
        env: &[],
        cwd_name: "pi-project",
        cwd_entries: &[],
        fs_entries: &[],
    };

    let expected = "podman run -it --userns=keep-id -w /workspace/pi-project --rm -v <HOME>/.pi/agent:/home/dungeon/.pi/agent:rw -v <CWD>:/workspace/pi-project localhost/dungeon zsh -ic pi";

    assert_command(input, expected);
}

#[test]
fn renders_cache_subpath_mounts_from_config_environment_and_cli() {
    let input = TestInput {
        toml: r#"
[general]
caches = ["pacman:/var/cache/pacman/pkg:rw"]
"#,
        args: &[
            "run",
            "--cache",
            "npm:/home/dungeon/.cache/npm",
            "--cache",
            "uv:/home/dungeon/.cache/uv:ro",
        ],
        env: &[("DUNGEON_CACHES", "env:/home/dungeon/.cache/dungeon")],
        cwd_name: "cache-subpath-project",
        cwd_entries: &[],
        fs_entries: &[],
    };

    let expected = "podman run -it --userns=keep-id -w /workspace/cache-subpath-project --rm --mount type=volume,src=dungeon-cache,dst=/var/cache/pacman/pkg,subpath=pacman,rw --mount type=volume,src=dungeon-cache,dst=/home/dungeon/.cache/dungeon,subpath=env --mount type=volume,src=dungeon-cache,dst=/home/dungeon/.cache/npm,subpath=npm --mount type=volume,src=dungeon-cache,dst=/home/dungeon/.cache/uv,subpath=uv,ro -v <CWD>:/workspace/cache-subpath-project localhost/dungeon zsh";

    assert_command(input, expected);
}
