use crate::tests::support::{assert_command, TestInput};

#[test]
fn basic_run_uses_cwd_mount() {
    let input = TestInput {
        toml: "",
        args: &[],
        env: &[],
    };

    let expected = "podman run -it --userns=keep-id -w /home/dungeon/project --rm -v dungeon-cache:/home/dungeon/.cache -v dungeon-cache:/home/dungeon/.npm -v <CWD>:/home/dungeon/project localhost/dungeon bash";

    assert_command(input, expected);
}
