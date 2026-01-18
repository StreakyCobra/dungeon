# dungeon

`dungeon` is a rootless Podman wrapper to create sandboxed development containers with minimal configuration.

## How it works

This is the Podman command to create a temporary container, mount the current directory, bring some cache folders, and make Codex config and auth available:

```shell
podman run -it --rm --userns=keep-id -w /home/dungeon/myrepo \
  -v dungeon-cache:/home/dungeon/.cache \
  -v dungeon-cache:/home/dungeon/.npm \
  -v "$HOME:/home/dungeon/.codex \
  -v "$PWD:/home/dungeon/myrepo" \
  localhost/dungeon \
  bash
```

With dungeon it gets much simpler:

```shell
dungeon --codex
```

It gets even easier when a composition of tools/configurations is desired:

```shell
dungeon --codex --obsidian
```

## Getting started

Ensure you have all needed requirements:
- [podman](https://podman.io/) (in [rootless](https://github.com/containers/podman/blob/main/README.md#rootless) mode)
- [rust](https://rust-lang.org/)

Build one of the provided images:

```shell
podman build -f images/Containerfile.archlinux -t localhost/dungeon .
# OR
podman build -f images/Containerfile.ubuntu -t localhost/dungeon .
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

Dungeon is ready to use; check usage with:

```shell
dungeon --help
```

## Usage

```text
Usage: dungeon [OPTIONS] [paths]...

Arguments:
  [paths]...  Paths to mount inside the container (default: current directory)

Options:
      --help         Show help information
      --reset-cache  Clear the dungeon-cache volume before running
      --version      Show version information

Persistence:
      --persist    Persist the container
      --persisted  Connect to the persisted container
      --discard    Remove the persisted container

Configurations:
      --run <run>                Run a command inside the container
      --image <image>            Select the container image
      --port <port>              Publish a container port (repeatable)
      --cache <cache>            Mount a cache volume target (repeatable)
      --mount <mount>            Bind-mount a host path (repeatable)
      --env <env>                Add a container environment variable (repeatable)
      --env-file <env-file>      Add a Podman env-file (repeatable)
      --podman-arg <podman-arg>  Append an extra podman run argument (repeatable)

Groups:
      --claude    Enable the claude group
      --codex     Enable the codex group
      --gemini    Enable the gemini group
      --opencode  Enable the opencode group
```

## Images

Container files live in `images/`:
- `images/Containerfile.archlinux`
- `images/Containerfile.ubuntu`

These images provide the base setup to work with dungeon and use AI agents inside. They are meant to be customized to include the tools you usually need for your projects. Note that passwordless sudo is allowed within the container.

Build the one you like with Podman:

```shell
podman build -f images/Containerfile.archlinux -t localhost/dungeon .
# OR
podman build -f images/Containerfile.ubuntu -t localhost/dungeon .
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

Single arguments like the command to run override lower-level configuration. List arguments like ports, mounts, and groups are merged with lower-level configuration.

Groups defined in config replace defaults, and an empty table removes a default group. Groups are applied after the top-level config file, with explicit CLI group flags taking precedence over `always_on_groups`.

### CLI flags

See `dungeon --help` in [Usage](#usage) above to see the available CLI configuration flags.

### Configuration file

Defaults live in `src/config/defaults.toml` (embedded at build time). User config overrides them at `$XDG_CONFIG_HOME/dungeon/config.toml` (or `~/.config/dungeon/config.toml`).

Example:
```toml
run = "codex"
image = "localhost/dungeon"
ports = ["127.0.0.1:8888:8888"]
caches = [".cache/pip:rw"]
mounts = ["~/projects:/home/dungeon/projects:rw"]
envs = ["OPENAI_API_KEY", "SECRET=abc123"]
env_files = [".env", "secrets.env"]
podman_args = ["--cap-add=SYS_PTRACE"]
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
- `mounts` entries are passed directly to Podman as `-v` arguments; dungeon only checks for a home-directory mount.
- `caches` entries are passed directly as `dungeon-cache:<spec>` volume mounts.
- `envs` entries are passed directly to Podman (`NAME` or `NAME=VALUE`).
- `env_files` entries are passed to Podman via `--env-file`.
- `mounts`, `caches`, `envs`, `env_files`, `ports`, and `podman_args` extend the base settings when enabled.
- `run` and `image` use the last enabled group when multiple are set.

### Environment variables

Environment overrides use:
- `DUNGEON_RUN`
- `DUNGEON_IMAGE`
- `DUNGEON_PORTS` (comma-separated)
- `DUNGEON_CACHES` (comma-separated)
- `DUNGEON_MOUNTS` (comma-separated)
- `DUNGEON_ENVS` (comma-separated)
- `DUNGEON_ENV_FILES` (comma-separated)
- `DUNGEON_PODMAN_ARGS` (comma-separated)
- `DUNGEON_DEFAULT_GROUPS` (comma-separated)

### Default configuration

The default config is embedded at build time from [`src/config/defaults.toml`](./src/config/defaults.toml) and is also the reference for all available groups and settings.

## Persistence

Use the CLI `--persist` flag to tell Podman to keep a container instead of deleting it after the bash session closes or the run command terminates.

Persisted containers are tied to the current folder: they are named `dungeon-<folder_name>-<path_hash>`. When you run `dungeon --persisted` (no other arguments are allowed), dungeon restarts the container and opens a bash session for the container matching the current directory, if it exists.

This enables project-level persisted containers if you prefer them over temporary containers.

## Cache

A named volume `dungeon-cache` is used for caches. This lets you mount specific folders to cache between temporary sessions. This is typically used to speed up installing dependencies.

## License

See [LICENSE](LICENSE)
