# dungeon

> ⚠️ Warning: this project is currently vibe-coded and still alpha, so breaking changes are expected. I plan to review the code once the program behavior and interface are settled, to move it toward vibe-engineered.

`dungeon` is a Podman/Docker wrapper to create sandboxed development containers with minimal configuration.

## How it works

This is the Podman command to create a temporary container, mount the current directory, and make Codex config and auth available:

```shell
podman run -it --rm --userns=keep-id -w /home/dungeon/myrepo \
  -v "$HOME:/home/dungeon/.codex" \
  -v "$PWD:/home/dungeon/myrepo" \
  localhost/dungeon \
  bash
```

With dungeon it gets much simpler:

```shell
dungeon run --codex
```

It gets even easier when you want a composition of tools/configurations:

```shell
dungeon run --codex --obsidian
```

## Getting started

Ensure you have the required tools:
- [podman](https://podman.io/) (recommended in [rootless](https://github.com/containers/podman/blob/main/README.md#rootless) mode) or [docker](https://www.docker.com/)
- [rust](https://rust-lang.org/)

Build one of the provided images, based on your personal favorite:

```shell
dungeon image build archlinux
# OR
dungeon image build ubuntu
```

If you only want to test `dungeon`, you can build it and run it from there:

```shell
cargo build
export PATH="$PWD/target/debug:$PATH"
```

Then you can move to any project and run `dungeon`.

If you want to install it:

```shell
cargo install --path .
```

This will install it in `~/.cargo/bin`. You can add this to your path with the following:

```shell
echo 'export PATH="$HOME/.cargo/bin:$PATH"' >> ~/.bashrc
```

## Usage

- `dungeon run` handles container session.
- `dungeon image` works with dungeon images.
- `dungeon cache` manages `dungeon-cache` volume.

Common commands:

```shell
# run a session
dungeon run

# run a session with codex available
dungeon run --codex --command codex

# build image
dungeon image build archlinux
```

## Images

Container files live in `images/`:
- `images/Containerfile.archlinux`
- `images/Containerfile.ubuntu`

These images provide the base setup to work with dungeon and use AI agents inside. They are meant to be customized to include the tools you usually need for your projects. Note that passwordless sudo is allowed within the container.

Build the one you like with Podman or Docker:

```shell
dungeon image build archlinux
# OR
dungeon image build ubuntu
```
You can build several images with different tags using `--tag`, and use the [Configuration](#configuration) below to switch images.

There is also the option to [persist](#persistence) containers if you don't want to extend the base image but keep a container around for some time.

## Configuration

There are several ways to configure dungeon, in order of precedence:
- [CLI flags](#cli-flags)
- [Environment variables](#environment-variables)
- [Groups](#groups)
- [Configuration file](#configuration-file)
- [Default configuration](#default-configuration)

For `dungeon run`, single settings like `command`, `image`, and `engine` override lower-level configuration. List settings like ports, mounts, and groups are merged with lower-level configuration.

Configuration file, env vars, and groups apply to `dungeon run` only.

`dungeon` ships with a few default groups. Redefining these groups overrides the default ones. Groups are applied after the `[general]` configuration, with explicit CLI group flags taking precedence over `always_on_groups`.

### CLI flags

Run-session flags live under `dungeon run`:

- `--debug` to print the generated command instead of running it.
- `--persist`, `--persisted`, `--discard` to manage container persistence.
- `--engine` to select `podman` or `docker` engine.
- `--command`, `--image`, `--port`, `--cache`, `--mount`, `--env`, `--env-file`, `--engine-arg` to customize container.
- `--skip-cwd` to skip mounting the current directory.
- group flags (for example `--codex`)

Image and cache management:

- `dungeon image build <archlinux|ubuntu> [--engine <podman|docker>] [--tag <tag>] [--no-cache] [--context <path>]`
- `dungeon cache reset [--engine <podman|docker>]`

### Configuration file

Defaults live in `src/config/defaults.toml` (embedded at build time). User config overrides them at `$XDG_CONFIG_HOME/dungeon/config.toml` (or `~/.config/dungeon/config.toml`).

Example:
```toml
[general]
command = "codex"
engine = "podman"
image = "localhost/dungeon"
ports = ["127.0.0.1:8888:8888"]
caches = [".cache/pip:rw"]
mounts = ["~/projects:/home/dungeon/projects:rw"]
envs = ["OPENAI_API_KEY", "SECRET=abc123"]
env_files = [".env", "secrets.env"]
engine_args = ["--cap-add=SYS_PTRACE"]
forbidden_markers = [".no-dungeon"]
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
- `[general]` defines global defaults and is reserved (it cannot be used as a group name).
- Each other top-level table (for example `[codex]`) defines a group.
- Each group name becomes a CLI flag (example: `--codex`).
- An empty group table removes a default group of the same name.

- `always_on_groups` lists groups that are always enabled, in order of precedence (later entries take precedence).
- `mounts` entries are passed directly to the selected engine as `-v` arguments; dungeon only checks to prevent a home-directory mount.
- `--skip-cwd` prevents the implicit current-directory mount when no paths are provided.
- dungeon refuses to run when `.dungeon-forbidden` exists in the current directory or any parent directory.
- `forbidden_markers` adds extra marker filenames (also checked from current directory up to root).
- `caches` entries are passed directly as `dungeon-cache:<spec>` volume mounts.
- `envs` entries are passed directly to the selected engine (`NAME` or `NAME=VALUE`).
- `env_files` entries are passed to the selected engine via `--env-file`.
- `mounts`, `caches`, `envs`, `env_files`, `ports`, and `engine_args` extend the base settings when enabled.
- `command` and `image` use the last enabled group when multiple are set.
- `engine` also uses the last enabled group when multiple are set.

### Environment variables

Environment overrides use:
- `DUNGEON_COMMAND`
- `DUNGEON_ENGINE`
- `DUNGEON_IMAGE`
- `DUNGEON_PORTS` (comma-separated)
- `DUNGEON_CACHES` (comma-separated)
- `DUNGEON_MOUNTS` (comma-separated)
- `DUNGEON_ENVS` (comma-separated)
- `DUNGEON_ENV_FILES` (comma-separated)
- `DUNGEON_ENGINE_ARGS` (comma-separated)
- `DUNGEON_FORBIDDEN_MARKERS` (comma-separated)
- `DUNGEON_ALWAYS_ON_GROUPS` (comma-separated)

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
