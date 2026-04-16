# dungeon

> ⚠️ Warning: this project is currently vibe-coded and still alpha, so breaking changes are expected. I plan to review the code once the program behavior and interface are settled, to move it toward vibe-engineered.

`dungeon` is a Podman wrapper to create sandboxed development containers with minimal configuration.

## How it works

This is the Podman command to create a temporary container, mount the current directory, and make Codex config and auth available:

```shell
podman run -it --rm --user root --userns=keep-id \
  --cap-add NET_ADMIN --cap-add NET_RAW --cap-add SYS_ADMIN \
  --cap-add SYS_CHROOT --cap-add SETUID --cap-add SETGID --cap-add SYS_PTRACE \
  --security-opt seccomp=unconfined \
  -w /home/dungeon/myrepo \
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
- [podman](https://podman.io/) (recommended in [rootless](https://github.com/containers/podman/blob/main/README.md#rootless) mode)
- [rust](https://rust-lang.org/)

Build the provided image:

```shell
dungeon image build
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
dungeon image build
```

## Images

Container files live in `images/`:
- `images/Containerfile`

The image provides the base setup to work with dungeon and use AI agents inside. It is meant to be customized to include the tools you usually need for your projects.

Notable defaults:
- `dungeon` always starts the container through a root bootstrap that installs the network policy, then drops to the unprivileged `dungeon` user.
- passwordless sudo is limited to `sudo dungeon-install ...`, a small wrapper around `pacman` with a denylist for security-sensitive packages.
- `bubblewrap` is installed and configured so Codex can use its sandbox inside the container.

Build it with Podman:

```shell
dungeon image build
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

For `dungeon run`, single settings like `command`, `image`, and the `network` booleans override lower-level configuration. List settings like ports, mounts, groups, and network allowlists are merged with lower-level configuration.

Configuration file, env vars, and groups apply to `dungeon run` only.

`dungeon` ships with a few default groups. Redefining these groups overrides the default ones. Groups are applied after the `[general]` configuration, with explicit CLI group flags taking precedence over `always_on_groups`.

### CLI flags

Run-session flags live under `dungeon run`:

- `--debug` to print the generated command instead of running it.
- `--persist`, `--persisted`, `--discard` to manage container persistence.
- `--command`, `--image`, `--port`, `--cache`, `--mount`, `--env`, `--env-file`, `--engine-arg` to customize container.
- `--skip-cwd` to skip mounting the current directory.
- `--ipv6`, `--no-ipv6`, `--allow-dns`, `--deny-dns`, `--allow-domain`, `--allow-host` to customize the outbound network policy.
- group flags (for example `--codex`)

Image and cache management:

- `dungeon image build [--tag <tag>] [--no-cache] [--context <path>]`
- `dungeon cache reset`

### Configuration file

Defaults live in `src/config/defaults.toml` (embedded at build time). User config overrides them at `$XDG_CONFIG_HOME/dungeon/config.toml` (or `~/.config/dungeon/config.toml`).

Example:
```toml
[general]
command = "codex"
image = "localhost/dungeon"
ports = ["127.0.0.1:8888:8888"]
caches = [".cache/pip:rw"]
mounts = ["~/projects:/home/dungeon/projects:rw"]
envs = ["OPENAI_API_KEY", "SECRET=abc123"]
env_files = [".env", "secrets.env"]
engine_args = ["--cap-add=SYS_PTRACE"]
always_on_groups = ["codex"]
ipv6 = false
allow_dns = false
allowed_tcp_domains = ["crates.io", "index.crates.io"]
allowed_tcp_hosts = ["10.0.0.0/8"]

[codex]
mounts = ["~/.codex:/home/dungeon/.codex:rw"]

allowed_tcp_domains = ["api.openai.com"]

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
- `mounts` entries are passed directly to Podman as `-v` arguments; dungeon only checks to prevent a home-directory mount.
- `--skip-cwd` prevents the implicit current-directory mount when no paths are provided.
- `caches` entries are passed directly as `dungeon-cache:<spec>` volume mounts.
- `envs` entries are passed directly to Podman (`NAME` or `NAME=VALUE`).
- `env_files` entries are passed to Podman via `--env-file`.
- `mounts`, `caches`, `envs`, `env_files`, `ports`, `engine_args`, and network allowlists extend the base settings when enabled.
- `command` and `image` use the last enabled group when multiple are set.
- `ipv6` and `allow_dns` use the highest-precedence value.

### Network policy

`dungeon` always starts with its firewall/bootstrap path enabled.

- If `allowed_tcp_domains` and `allowed_tcp_hosts` are both empty after merging all config layers, egress is unrestricted for enabled families.
- If either list is non-empty, egress is restricted to the merged allowlist.
- `ipv6 = false` disables IPv6 entirely.
- `ipv6 = true` enables IPv6 and applies the same filtering model as IPv4.
- `allow_dns = false` blocks container DNS queries after bootstrap has finished resolving any configured domains.

### Environment variables

Environment overrides use:
- `DUNGEON_COMMAND`
- `DUNGEON_IMAGE`
- `DUNGEON_PORTS` (comma-separated)
- `DUNGEON_CACHES` (comma-separated)
- `DUNGEON_MOUNTS` (comma-separated)
- `DUNGEON_ENVS` (comma-separated)
- `DUNGEON_ENV_FILES` (comma-separated)
- `DUNGEON_ENGINE_ARGS` (comma-separated)
- `DUNGEON_IPV6`
- `DUNGEON_ALLOW_DNS`
- `DUNGEON_ALLOWED_TCP_DOMAINS` (comma-separated)
- `DUNGEON_ALLOWED_TCP_HOSTS` (comma-separated)
- `DUNGEON_ALWAYS_ON_GROUPS` (comma-separated)

## Runtime behavior

- `dungeon run` always starts the container as root, installs the firewall policy, and then drops to the `dungeon` user.
- The Podman command keeps `--userns=keep-id`, so bind-mounted files still line up with the host user.
- The image entrypoint is `dungeon-bootstrap`, which applies the runtime network policy.
- Codex can rely on `bubblewrap`; there is no `CODEX_UNSAFE_ALLOW_NO_SANDBOX` fallback configured.

## Installing packages

Use `sudo dungeon-install ...` inside the container when the agent needs extra Arch packages.

- It is a small wrapper around `pacman -S --needed --noconfirm`.
- It rejects flags, local package files, URLs, and a denylist of security-sensitive packages.
- Broad root access is not available inside the container.

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
