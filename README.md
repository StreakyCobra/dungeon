# dungeon

`dungeon` is a sandboxed development container system. It is build as a developer friendly wrapper over podman.

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

Run directly a command inside the container:

```shell
dungeon --codex --run codex
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
dungeon --codex --opencode
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

Note: the Containerfiles use `RUN --mount=type=cache` for package caches. Podman supports this.

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
- `--help` shows the help message.
- `--version` prints the version.
- `--reset-cache` deletes the `dungeon-cache` volume before running.
- `--persist` creates or reuses a persisted container.
- `--persisted` reuses the persisted container (no extra config/group/path args).
- `--discard` removes the persisted container.

Configuration:
- `--run` runs a command inside the container.
- `--image` selects the container image.
- `--port` publishes a container port (repeatable).
- `--cache` mounts cache volume targets (repeatable).
- `--mount` bind-mounts a host path (repeatable).
- `--env` adds container environment variables (repeatable).
- `--podman-arg` appends a `podman run` argument (repeatable).

Groups defined in config become flags (example: `--codex`, `--obsidian`).

Persistence:
- `--persist` creates a named container and keeps it for reuse.
- `--persisted` starts the named container if stopped and opens a shell.
- `--discard` removes the named container.
- Names are based on the current folder and a hash of the path.

## Config file

Defaults live in `internal/config/defaults.toml` and are embedded at build time.
User config overrides them at `~/.config/dungeon/config.toml` (or `$XDG_CONFIG_HOME/dungeon/config.toml`).
Precedence is defaults < group config < config file top level < environment < CLI flags.
Only provided values override earlier sources; list settings are merged by appending.
Groups defined in config replace defaults, and an empty table removes a default group.
Environment overrides use:
- `DUNGEON_RUN`, `DUNGEON_IMAGE`
- `DUNGEON_PORTS` (comma-separated)
- `DUNGEON_CACHES` (comma-separated)
- `DUNGEON_MOUNTS` (comma-separated)
- `DUNGEON_ENVS` (comma-separated)
- `DUNGEON_PODMAN_ARGS` (comma-separated)
- `DUNGEON_DEFAULT_GROUPS` (comma-separated)

Example:
```
run = "codex"
image = "localhost/dungeon"
ports = ["127.0.0.1:8888:8888"]
caches = [".cache/pip:rw"]
mounts = ["~/projects:/home/dungeon/projects:rw"]
envs = ["OPENAI_API_KEY"]
podman_args = ["--cap-add=SYS_PTRACE"]
default_groups = ["codex"]

[codex]
mounts = ["~/.codex:/home/dungeon/.codex:rw"]

[obsidian]
mounts = ["~/my_vault:/home/dungeon/obsidian:ro"]

[python]
caches = ["/var/cache/pacman/pkg"]
envs = ["OPENAPI_KEY"]
ports = ["127.0.0.1:8000:8000"]
```

Group behavior:
- Each top-level table (for example `[codex]`) defines a group.
- Each group name becomes a CLI flag (example: `--codex`).
- `default_groups` lists groups enabled by default, in order.
- An empty group table removes a default group of the same name.
- `mounts` entries use `source:target[:ro|rw]`.
- `caches` entries use `target[:ro|rw]` from the `dungeon-cache` volume.
- `envs` entries support `NAME=VALUE` for static values or `NAME` to pass through the host value.
- `mounts`, `caches`, `envs`, `ports`, and `podman_args` extend the base settings when enabled.
- `run` and `image` use the last enabled group when multiple are set.
- Group settings apply before top-level config, env vars, and CLI overrides.
- `source` may be absolute, `~/...`, or relative to `$HOME`; `target` may be absolute or relative to `/home/dungeon`.

Run behavior:
- `image` overrides the container image (default `localhost/dungeon`).
- `ports` adds `-p` rules (repeatable).
- `caches` adds `dungeon-cache` volume mounts.
- `mounts` adds bind mounts from the host.
- `envs` adds `--env` entries.
- `podman_args` appends extra `podman run` args.

## Notes
- The container user is `dungeon` with passwordless sudo.
- A named volume `dungeon-cache` is used for caches.

## License
MIT. See `LICENSE`.
