# dungeon

`dungeon` is a Podman/Docker wrapper to create sandboxed development containers with minimal configuration.

## How it works

This is the Podman command to create a temporary container, mount the current directory, bring some cache folders, and make Codex config and auth available:

```shell
podman run -it --rm --userns=keep-id -w /home/dungeon/myrepo \
  -v dungeon-cache:/home/dungeon/.cache \
  -v dungeon-cache:/home/dungeon/.npm \
  -v "$HOME:/home/dungeon/.codex" \
  -v "$PWD:/home/dungeon/myrepo" \
  localhost/dungeon \
  bash
```

With dungeon it gets much simpler:

```shell
dungeon run --codex
```

It gets even easier when a composition of tools/configurations is desired:

```shell
dungeon run --codex --obsidian
```

## Getting started

Ensure you have all needed requirements:
- [podman](https://podman.io/) (recommended in [rootless](https://github.com/containers/podman/blob/main/README.md#rootless) mode) or [docker](https://www.docker.com/)
- [rust](https://rust-lang.org/)

Build one of the provided images:

```shell
dungeon image build archlinux
# OR
dungeon image build ubuntu

# use docker and refresh layers
dungeon image build ubuntu --engine docker --no-cache

# manual equivalent
podman build -f images/Containerfile.archlinux -t localhost/dungeon .
# OR
podman build -f images/Containerfile.ubuntu -t localhost/dungeon .

# OR with docker
docker build -f images/Containerfile.archlinux -t localhost/dungeon .
# OR
docker build -f images/Containerfile.ubuntu -t localhost/dungeon .
```

Build the CLI and install it:

```shell
cargo install --path .
```

If you want to run dungeon without having to specify the path, add `~/.cargo/bin` to your PATH:

```shell
export PATH="$HOME/.cargo/bin:$PATH"
echo 'export PATH="$HOME/.cargo/bin:$PATH"' >> ~/.bashrc
```

## Usage

- `dungeon run` starts a container session.
- `dungeon image build` builds one of the provided images.
- `dungeon cache reset` clears the `dungeon-cache` volume.

Common commands:

```shell
# run a session
dungeon run --codex

# run with explicit engine
dungeon run --engine docker

# build image
dungeon image build archlinux

# force image refresh
dungeon image build archlinux --no-cache

# clear cache volume
dungeon cache reset
```

## Images

Container files live in `images/`:
- `images/Containerfile.archlinux`
- `images/Containerfile.ubuntu`

These images provide the base setup to work with dungeon and use AI agents inside. They are meant to be customized to include the tools you usually need for your projects. Note that passwordless sudo is allowed within the container.

Build the one you like with Podman or Docker:

```shell
podman build -f images/Containerfile.archlinux -t localhost/dungeon .
# OR
podman build -f images/Containerfile.ubuntu -t localhost/dungeon .

# OR with docker
docker build -f images/Containerfile.archlinux -t localhost/dungeon .
# OR
docker build -f images/Containerfile.ubuntu -t localhost/dungeon .
```

You can build several images by giving them different tags with `-t`, and use the [Configuration](#configuration) below to switch images.

There is also the option to [persist](#persistence) containers if you don't want to extend the base image but keep a container around for some time.

## Configuration

There are several ways to configure dungeon, in order of precedence:
- [CLI flags](#cli-flags)
- [Environment variables](#environment-variables)
- [Groups](#groups)
- [Configuration file](#configuration-file)
- [Default configuration](#default-configuration)

Single arguments like `run`, `image`, and `engine` override lower-level configuration. List arguments like ports, mounts, and groups are merged with lower-level configuration.

Configuration file, env vars, and groups apply to `dungeon run` only.

Groups defined in config replace defaults, and an empty table removes a default group. Groups are applied after the top-level config file, with explicit CLI group flags taking precedence over `always_on_groups`.

### CLI flags

Run-session flags live under `dungeon run`:

- `--debug`
- `--persist`, `--persisted`, `--discard`
- `--engine`, `--run`, `--image`
- `--port`, `--cache`, `--mount`, `--env`, `--env-file`, `--engine-arg`
- `--skip-cwd`
- group flags (for example `--codex`)

Image and cache management:

- `dungeon image build <archlinux|ubuntu> [--engine <podman|docker>] [--tag <tag>] [--no-cache] [--context <path>]`
- `dungeon cache reset [--engine <podman|docker>]`

### Configuration file

Defaults live in `src/config/defaults.toml` (embedded at build time). User config overrides them at `$XDG_CONFIG_HOME/dungeon/config.toml` (or `~/.config/dungeon/config.toml`).

Example:
```toml
run = "codex"
engine = "podman"
image = "localhost/dungeon"
ports = ["127.0.0.1:8888:8888"]
caches = [".cache/pip:rw"]
mounts = ["~/projects:/home/dungeon/projects:rw"]
envs = ["OPENAI_API_KEY", "SECRET=abc123"]
env_files = [".env", "secrets.env"]
engine_args = ["--cap-add=SYS_PTRACE"]
always_on_groups = ["codex"]

[codex]
mounts = ["~/.codex:/home/dungeon/.codex:rw"]

[obsidian]
mounts = ["~/my_vault:/home/dungeon/obsidian:ro"]

[python]
caches = ["/var/cache/pacman/pkg"]
envs = ["MYSECRETDJANGOKEY"]
ports = ["127.0.0.1:8000:8000"]
```

Group behavior:
- Each top-level table (for example `[codex]`) defines a group.
- Each group name becomes a CLI flag (example: `--codex`).
- An empty group table removes a default group of the same name.

- `always_on_groups` lists groups always enabled, in order of precedence (later entries take precedence).
- `mounts` entries are passed directly to the selected engine as `-v` arguments; dungeon only checks for a home-directory mount.
- `--skip-cwd` prevents the implicit current-directory mount when no paths are provided.
- `caches` entries are passed directly as `dungeon-cache:<spec>` volume mounts.
- `envs` entries are passed directly to the selected engine (`NAME` or `NAME=VALUE`).
- `env_files` entries are passed to the selected engine via `--env-file`.
- `mounts`, `caches`, `envs`, `env_files`, `ports`, and `engine_args` extend the base settings when enabled.
- `run` and `image` use the last enabled group when multiple are set.
- `engine` also uses the last enabled group when multiple are set.

### Environment variables

Environment overrides use:
- `DUNGEON_RUN`
- `DUNGEON_ENGINE`
- `DUNGEON_IMAGE`
- `DUNGEON_PORTS` (comma-separated)
- `DUNGEON_CACHES` (comma-separated)
- `DUNGEON_MOUNTS` (comma-separated)
- `DUNGEON_ENVS` (comma-separated)
- `DUNGEON_ENV_FILES` (comma-separated)
- `DUNGEON_ENGINE_ARGS` (comma-separated)
- `DUNGEON_DEFAULT_GROUPS` (comma-separated)

## Engine behavior

- `engine = "podman"` uses `--userns=keep-id`, which keeps file ownership aligned with the host user in bind mounts.
- `engine = "docker"` uses `--user <uid>:<gid>` so files created in bind mounts belong to the host user.
- The provided images remain Podman-compatible and still default to the `dungeon` user.
- The default engine is `podman`.

### Default configuration

The default config is embedded at build time from [`src/config/defaults.toml`](./src/config/defaults.toml) and is also the reference for all available groups and settings.

## Persistence

Use `dungeon run --persist` to tell the selected engine to keep a container instead of deleting it after the bash session closes or the run command terminates.

Persisted containers are tied to the current folder: they are named `dungeon-<folder_name>-<path_hash>`. When you run `dungeon run --persisted` (no other run arguments are allowed), dungeon restarts the container and opens a bash session for the container matching the current directory, if it exists.

This enables project-level persisted containers if you prefer them over temporary containers.

## Cache

A named volume `dungeon-cache` is used for caches. This lets you mount specific folders to cache between temporary sessions. This is typically used to speed up installing dependencies.

Reset it with:

```shell
dungeon cache reset
```

## License

See [LICENSE](LICENSE)
