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
  -w /workspace/myrepo \
  -v "$HOME:/home/dungeon/.codex" \
  -v "$PWD:/workspace/myrepo" \
  localhost/dungeon \
  zsh
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
echo 'export PATH="$HOME/.cargo/bin:$PATH"' >> ~/.zshrc
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

# run a session with pi available
dungeon run --pi --command pi

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

For `dungeon run`, single settings like `command`, `image`, and the `network` booleans override lower-level configuration. List settings like ports, dynamic ports, mounts, and network allowlists are merged with lower-level configuration.

Configuration file, environment variables, and included groups apply to `dungeon run`; image and cache commands also use applicable global settings such as `podman_args`.

`dungeon` ships with three default groups: `codex`, `opencode`, and `pi`. Redefining these groups overrides the default ones. Groups are applied after the `[general]` configuration. Explicit CLI group flags are selected after `[general].include_groups`, subject to dependency ordering.

### CLI flags

Run-session flags live under `dungeon run`:

- `--debug` to print the generated command instead of running it.
- `--persist`, `--persisted`, `--discard` to manage container persistence.
- `--command`, `--image`, `--port`, `--dynamic-port`, `--cache`, `--mount`, `--env`, `--env-file`, `--podman-arg`, `--run-arg`, `--mount-git-metadata`, `--no-mount-git-metadata` to customize container.
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
mount_git_metadata = false
ports = ["127.0.0.1:8888:8888"]
dynamic_ports = ["difit"]
caches = [".cache/pip:rw"]
mounts = ["~/projects:/home/dungeon/projects:rw"]
envs = ["OPENAI_API_KEY", "SECRET=abc123"]
env_files = [".env", "secrets.env"]
podman_args = ["-c", "agent-vm"]
run_args = ["--cap-add=SYS_PTRACE"]
include_groups = ["ai"]
ipv6 = false
allow_dns = true
allowed_tcp_domains = ["crates.io", "index.crates.io"]
allowed_tcp_hosts = ["10.0.0.0/8"]

[ai]
include_groups = ["codex", "difit"]

[codex]
mounts = ["~/.codex:/home/dungeon/.codex:rw"]

[difit]
dynamic_ports = ["difit"]

allowed_tcp_domains = ["api.openai.com"]

[obsidian]
mounts = ["~/my_vault:/home/dungeon/obsidian:ro"]

[pi]
mounts = ["~/.pi/agent:/home/dungeon/.pi/agent:rw"]

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
- `[general].include_groups` lists root groups to enable. `DUNGEON_INCLUDE_GROUPS` adds more root groups as a comma-separated list.
- A group's `include_groups` lists its dependencies. Dependencies are applied before the including group, and sibling dependencies keep declaration order.
- Each reachable group is applied once. Unknown included groups and inclusion cycles are configuration errors.
- `mounts` entries are passed directly to Podman as `-v` arguments; dungeon only checks to prevent a home-directory mount.
- `mount_git_metadata = true` makes dungeon inspect mounted directories for `.git` files that point outside the workspace and bind-mount the referenced Git metadata path so Git worktrees work inside the container.
- `podman_args` entries are inserted before the Podman subcommand, for example `podman -c agent-vm run ...`.
- `--skip-cwd` prevents the implicit current-directory mount when no paths are provided.
- `caches` entries are passed directly as `dungeon-cache:<spec>` volume mounts.
- `envs` entries are passed directly to Podman (`NAME` or `NAME=VALUE`).
- `env_files` entries are passed to Podman via `--env-file`.
- `dynamic_ports`, `DUNGEON_DYNAMIC_PORTS`, and repeatable `--dynamic-port <name>` each add a dynamic port. Names must be lower-case ASCII identifiers (`[a-z][a-z0-9_]*`); `difit` adds `-p 127.0.0.1:X:X` and `DUNGEON_PORT_FOR_DIFIT=X`.
- Dynamic-port listeners are reserved until Dungeon starts Podman. They are not retained by `--debug`; persisted containers keep their selected mapping and must be recreated if it later conflicts on restart.
- The published host port is loopback-only. Services using a dynamic port must listen on `0.0.0.0` inside the container so Podman can forward traffic to them.
- `mounts`, `caches`, `envs`, `env_files`, `ports`, `dynamic_ports`, `podman_args`, `run_args`, and network allowlists extend the base settings when enabled.
- `command` and `image` use the last enabled group when multiple are set.
- `mount_git_metadata`, `ipv6`, and `allow_dns` use the highest-precedence value.

### Network policy

`dungeon` always starts with its firewall/bootstrap path enabled.

- If `allowed_tcp_domains` and `allowed_tcp_hosts` are both empty after merging all config layers, egress is unrestricted for enabled families.
- If either list is non-empty, egress is restricted to the merged allowlist.
- `ipv6 = false` disables IPv6 entirely.
- `ipv6 = true` enables IPv6 and applies the same filtering model as IPv4.
- `allow_dns = true` allows container DNS queries by default.
- `allow_dns = false` blocks container DNS queries after bootstrap has finished resolving any configured domains.

### Environment variables

Environment overrides use:
- `DUNGEON_COMMAND`
- `DUNGEON_IMAGE`
- `DUNGEON_PORTS` (comma-separated)
- `DUNGEON_DYNAMIC_PORTS` (comma-separated)
- `DUNGEON_CACHES` (comma-separated)
- `DUNGEON_MOUNTS` (comma-separated)
- `DUNGEON_ENVS` (comma-separated)
- `DUNGEON_ENV_FILES` (comma-separated)
- `DUNGEON_PODMAN_ARGS` (comma-separated)
- `DUNGEON_RUN_ARGS` (comma-separated)
- `DUNGEON_MOUNT_GIT_METADATA`
- `DUNGEON_IPV6`
- `DUNGEON_ALLOW_DNS`
- `DUNGEON_ALLOWED_TCP_DOMAINS` (comma-separated)
- `DUNGEON_ALLOWED_TCP_HOSTS` (comma-separated)
- `DUNGEON_INCLUDE_GROUPS` (comma-separated)

## Runtime behavior

- `dungeon run` always starts the container as root, installs the firewall policy, drops capabilities, and then switches to the `dungeon` user.
- The runtime intentionally preserves the image's narrow `sudo dungeon-install ...` path; broader root access still is not granted.
- The Podman command keeps `--userns=keep-id`, so bind-mounted files still line up with the host user.
- The image entrypoint is `dungeon-bootstrap`, which applies the runtime network policy.
- `mount_git_metadata = true` is intended for Git worktrees and other checkouts with `.git` files that point outside the mounted workspace. It currently supports absolute `gitdir:` paths only.
- Codex can rely on `bubblewrap`; there is no `CODEX_UNSAFE_ALLOW_NO_SANDBOX` fallback configured.
- The built-in `pi` group mounts `~/.pi/agent`, which covers Pi auth, settings, sessions, and installed Pi packages.

## Installing packages

Use `sudo dungeon-install ...` inside the container when the agent needs extra Arch packages.

- It is a small wrapper around `pacman -S --needed --noconfirm`.
- It rejects flags, local package files, URLs, and a denylist of security-sensitive packages.
- It depends on the image's passwordless sudoers rule for `/usr/local/bin/dungeon-install`.
- Broad root access is not available inside the container.

### Default configuration

The default config is embedded at build time from [`src/config/defaults.toml`](./src/config/defaults.toml) and is also the reference for all available groups and settings.

## Persistence

Use `dungeon run --persist` to tell the selected engine to keep a container instead of deleting it after the shell session closes or the run command terminates.

Persisted containers are tied to the current folder: they are named `dungeon-<folder_name>-<path_hash>`. When you run `dungeon run --persisted` (no other run arguments are allowed), dungeon restarts the container and opens a shell session for the container matching the current directory, if it exists.

This enables project-level persisted containers if you prefer them over temporary containers.

## Cache

A named volume `dungeon-cache` is used for caches. This lets you mount specific folders to cache between temporary sessions. This is typically used to speed up installing dependencies.

Reset it with:

```shell
dungeon cache reset
```

## License

See [LICENSE](LICENSE)

Test PR
