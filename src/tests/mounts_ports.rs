use crate::tests::support::{assert_command, TestInput};

#[test]
fn merges_mounts_ports_and_cache() {
    let input = TestInput {
        toml: r#"
ports = ["127.0.0.1:3000:3000"]
caches = ["/var/cache/pacman/pkg:ro"]
mounts = ["~/data:/data:ro"]
"#,
        args: &["--port", "127.0.0.1:8080:8080", "--cache", "deps:rw", "--mount", "$HOME/.codex:/home/dungeon/.codex:rw"],
        env: &[],
        cwd_name: "mounts-root",
        cwd_entries: &[],
    };

    let expected = "podman run -it --userns=keep-id -w /home/dungeon/mounts-root --rm -p 127.0.0.1:3000:3000 -p 127.0.0.1:8080:8080 -v dungeon-cache:/home/dungeon/.cache -v dungeon-cache:/home/dungeon/.npm -v <HOME>/data:/data:ro -v <HOME>/.codex:/home/dungeon/.codex:rw -v dungeon-cache:/var/cache/pacman/pkg:ro -v dungeon-cache:deps:rw -v <CWD>:/home/dungeon/mounts-root localhost/dungeon bash";

    assert_command(input, expected);
}
