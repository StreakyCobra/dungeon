# dungeon

`dungeon` is a sandboxed development container system. It is build as a developer friendly wrapper over podman/docker.

It is quick to launch, it comes preconfigured for AI agents, and it is easy to configure and extend.

## Getting started

Build an image and the CLI:

```shell
make archlinux
```

Start a shell in the container with your current project mounted:

```shell
./build/dungeon
```

You can alias it in the current session:

```shell
alias dungeon=$PWD/build/dungeon
```

Configure the use of Codex within the sandbox:

```shell
dungeon --codex
```

Run a command inside the container:

```shell
dungeon --marimo --run "marimo edit"
```

## Why it is simpler

This is the podman command to create a temporary container, mount the current directory, mount some cache folders, and mount codex:

```shell
podman run -it --rm --userns=keep-id -w /home/dungeon/myrepo \
  -v dungeon-cache:/home/dungeon/.cache \
  -v dungeon-cache:/home/dungeon/.npm \
  -v "$HOME:/home/dungeon/.codex \
  -v "$PWD:/home/dungeon/myrepo" \
  localhost/dungeon \
  bash
```

With dungeon it is much simpler:

```shell
dungeon --codex
```

It gets even easier when a composition of tools is desired:

```shell
dungeon --codex --marimo
```

## Images

Container files live in `images/`:
- `images/Containerfile.archlinux`
- `images/Containerfile.ubuntu`

Build with Make:

```shell
make archlinux
make ubuntu
```

The latest built image will be the default image when non is manually specified.

Note: the Containerfiles use `RUN --mount=type=cache` for package caches. Podman supports this; Docker requires BuildKit (`DOCKER_BUILDKIT=1 docker build ...`).

## CLI

Build the binary:

```shell
make cli
# OR
go build -o build/dungeon ./cmd/dungeon
```

Install the CLI:

```shell
go install github.com/StreakyCobra/dungeon/cmd/dungeon@latest
```

Options:
- `--run` runs a command inside the container.
- `--reset-cache` deletes the `dungeon-cache` volume before running.
- `--port` publishes a container port (repeatable).
- `--network` sets the container network mode.
- `--name` assigns a container name and disables `--rm` unless explicitly set.
- `--rm` removes the container after exit (default true unless `--name` is set).
- Groups defined in config become flags (example: `--codex`, `--obsidian`).

## Config file

Defaults live in `cmd/dungeon/defaults.toml` and are embedded at build time.
User config overrides them at `~/.config/dungeon/config.toml` (or `$XDG_CONFIG_HOME/dungeon/config.toml`).
Precedence is defaults < config < environment < CLI flags.

Example:
```
run = "marimo edit"
image = "localhost/dungeon"
ports = ["127.0.0.1:8888:8888"]
network = "host"
name = "dungeon-dev"
rm = false
podman_args = ["--cap-add=SYS_PTRACE"]
default_groups = ["codex"]

[codex]
mounts = ["~/.codex:/home/dungeon/.codex:rw"]

[obsidian]
mounts = ["~/my_vault:/home/dungeon/obsidian:ro"]

[python]
cache = ["/var/cache/pacman/pkg", ".cache/pip:rw"]
env = ["OPENAPI_KEY"]
```

Group behavior:
- Each top-level table (for example `[codex]`) defines a group.
- Each group name becomes a CLI flag (example: `--codex`).
- `default_groups` lists groups enabled by default.
- `mounts` entries use `source:target[:ro|rw]`.
- `cache` entries use `target[:ro|rw]` from the `dungeon-cache` volume.
- `env` entries support `NAME=VALUE` for static values or `NAME` to pass through the host value.
- `source` may be absolute, `~/...`, or relative to `$HOME`; `target` may be absolute or relative to `/home/dungeon`.
- Built-in groups include `codex` and `marimo` (override them by redefining `[codex]` or `[marimo]`).

Run behavior:
- `image` overrides the container image (default `localhost/dungeon`).
- `ports` adds `-p` rules (repeatable).
- `network` sets the network mode.
- `name` sets the container name.
- `rm` toggles `--rm` (default true unless `name` is set).
- `podman_args` appends extra `podman run` args.

## Notes
- The container user is `dungeon` with passwordless sudo.
- A named volume `dungeon-cache` is used for caches.

## License
MIT. See `LICENSE`.
