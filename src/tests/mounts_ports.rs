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
    };

    let expected = "podman run -it --userns=keep-id -w /workspace/pi-project --rm -v <HOME>/.pi/agent:/home/dungeon/.pi/agent:rw -v <CWD>:/workspace/pi-project localhost/dungeon zsh -ic pi";

    assert_command(input, expected);
}
