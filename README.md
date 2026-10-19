# dungeon

> ⚠️ Warning: this project is currently vibe-coded and still alpha, so breaking changes are expected. I plan to review the code once the program behavior and interface are settled, to move it toward vibe-engineered.

`dungeon` is a Podman wrapper to create sandboxed development containers with minimal configuration.

## How it works

This is the Podman command to create a temporary container, mount the current directory, and make Codex config and auth available:

```shell
podman run -it --rm --userns=keep-id --user root \
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

# create configured cache subdirectories
dungeon cache init
```

## Images

Container files live in `images/`:
- `images/Containerfile`

The image provides the base setup to work with dungeon and use AI agents inside. It is meant to be customized to include the tools you usually need for your projects.

Notable defaults:
- Containers run as the unprivileged `dungeon` user with `HOME=/home/dungeon`, so the image's Zsh and Zim configuration loads normally. Dungeon starts a minimal entrypoint as namespaced root and immediately drops privileges before running the requested command.
- passwordless sudo is limited to `sudo dungeon-install ...`, a small wrapper around `pacman` with a denylist for security-sensitive packages.
- `bubblewrap` is installed and configured so Codex can use its sandbox inside the container.

Build it with Podman:

```shell
dungeon image build
```
You can build several images with different tags using `--tag`, and use the [Configuration](#configuration) below to switch images.

## Configuration

There are several ways to configure dungeon, in order of precedence:
- [CLI flags](#cli-flags)
- [Environment variables](#environment-variables)
- [Groups](#groups)
- [Configuration file](#configuration-file)
- [Default configuration](#default-configuration)

For `dungeon run`, single settings like `command`, `image`, and `enable_ssh` override lower-level configuration. List settings like ports, dynamic ports, exposed host ports, mounts, caches, and SSH keys are merged with lower-level configuration.

Configuration file, environment variables, and included groups apply to `dungeon run`; image and cache commands also use applicable global settings such as `podman_args`.

`dungeon` ships with three default groups: `codex`, `opencode`, and `pi`. Redefining these groups overrides the default ones. Groups are applied after the `[general]` configuration. Explicit CLI group flags are selected after `[general].include_groups`, subject to dependency ordering.

### CLI flags

Run-session flags live under `dungeon run`:

- `--debug` to print the generated command instead of running it.
- `--command`, `--image`, `--port`, `--dynamic-port`, `--cache`, `--mount`, `--env`, `--env-file`, `--podman-arg`, `--run-arg`, `--mount-git-metadata`, `--no-mount-git-metadata` to customize container.
- repeatable `--secret <name>` or `--secret <env>=<secret>` to expose a Podman secret as a container environment variable.
- `--skip-cwd` to skip mounting the current directory.
- repeatable `--expose-host-port <spec>` to expose host-loopback TCP services or ranges inside the container.
- group flags (for example `--codex`)

Image and cache management:

- `dungeon image build [--tag <tag>] [--no-cache] [--context <path>]`
- `dungeon cache reset`
- `dungeon cache init [--image <image>] [--cache <spec>] [--podman-arg <arg>]`

### Configuration file

Defaults live in `src/config/defaults.toml` (embedded at build time). User config overrides them at `$XDG_CONFIG_HOME/dungeon/config.toml` (or `~/.config/dungeon/config.toml`).

Example:
```toml
[general]
command = "codex"
image = "localhost/dungeon"
mount_git_metadata = false
enable_ssh = true
ssh_keys = ["ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAA... you@example.com"]
ports = ["127.0.0.1:8888:8888"]
dynamic_ports = ["difit"]
expose_host_ports = ["8080", "18080:8080", "8000-8010"]
caches = [
  "pacman:/var/cache/pacman/pkg",
  "npm:/home/dungeon/.cache/npm",
  "uv:/home/dungeon/.cache/uv",
  "dungeon:/home/dungeon/.cache/dungeon",
]
mounts = ["~/projects:/home/dungeon/projects:rw"]
envs = ["OPENAI_API_KEY", "SECRET=abc123"]
env_files = [".env", "secrets.env"]
secrets = ["GH_TOKEN=MY_OTHER_SECRET", "SOME_OTHER"]
podman_args = ["--log-level=warn"]
run_args = ["--cap-add=SYS_PTRACE"]
include_groups = ["ai"]

[ai]
include_groups = ["codex", "difit"]

[codex]
mounts = ["~/.codex:/home/dungeon/.codex:rw"]

[difit]
dynamic_ports = ["difit"]

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
- `enable_ssh = true` starts a key-only SSH daemon before the main command. It requires at least one literal, single-line OpenSSH public key in `ssh_keys`.
- `ssh_keys` entries are appended across configuration layers and groups. They are written to `/home/dungeon/.ssh/authorized_keys` for the temporary container only.
- `podman_args` entries are inserted before the Podman subcommand, for example `podman -c agent-vm run ...`.
- `--skip-cwd` prevents the implicit current-directory mount when no paths are provided.
- `caches` entries use a list syntax and are appended across configuration layers and groups.
- A cache entry in the form `SUBPATH:ABSOLUTE_TARGET[:ro|rw]` mounts a subdirectory of `dungeon-cache`, for example `npm:/home/dungeon/.cache/npm` generates `--mount type=volume,src=dungeon-cache,dst=/home/dungeon/.cache/npm,subpath=npm`. Subpaths must be relative and cannot contain `.` or `..` components.
- Legacy cache entries such as `/var/cache/pacman/pkg` and `/var/cache/pacman/pkg:ro` continue to generate `-v dungeon-cache:<entry>` mounts for the whole volume.
- `dungeon cache init` creates and assigns the configured subdirectories to UID/GID 1000. It includes subpaths from general configuration, environment variables, the included configuration groups, and repeatable `--cache` flags. It also scans groups not currently enabled.
- Run `dungeon cache init` after `dungeon cache reset` and before a session that uses a new subpath. Podman requires a volume subpath to exist before mounting it.
- `envs` entries are passed directly to Podman (`NAME` or `NAME=VALUE`).
- `env_files` entries are passed to Podman via `--env-file`.
- `secrets` entries accept `NAME` or `ENV=SECRET` and are passed to Podman as environment-type secrets. For example, `GH_TOKEN` becomes `--secret=GH_TOKEN,type=env,target=GH_TOKEN`, and `GH_TOKEN=MY_OTHER_SECRET` becomes `--secret=MY_OTHER_SECRET,type=env,target=GH_TOKEN`. Duplicate entries across configuration layers are emitted only once.
- `dynamic_ports`, `DUNGEON_DYNAMIC_PORTS`, and repeatable `--dynamic-port <name>` each add a dynamic port. Names must be lower-case ASCII identifiers (`[a-z][a-z0-9_]*`); `difit` adds `-p 127.0.0.1:X:X` and `DUNGEON_PORT_FOR_DIFIT=X`.
- Dynamic-port listeners are reserved until Dungeon starts Podman. They are not retained by `--debug`.
- The published host port is loopback-only. Services using a dynamic port must listen on `0.0.0.0` inside the container so Podman can forward traffic to them.
- SSH uses the same reservation strategy: Dungeon allocates a random host loopback port, maps it to the same container port, and passes it as `DUNGEON_SSH_PORT`. Do not configure SSH with `ports` yourself.
- `expose_host_ports`, `DUNGEON_EXPOSE_HOST_PORTS`, and repeatable `--expose-host-port <spec>` expose host-loopback TCP services or ranges inside the container through pasta reverse forwarding.
- An exposed host port specification can be a port (`8080`), a translated port (`18080:8080` maps container port 18080 to host port 8080), a range (`8000-8010`), or equal-sized translated ranges (`10000-10010:20000-20010`). Ports must be between 1 and 65535.
- Exposed host ports generate repeated pasta arguments directly, for example `--network=pasta:-T,8080,-T,18080:8080`. Each mapping must be comma-free because Podman uses commas to split pasta arguments. Explicit `--network` or `--net` values in `run_args` conflict with this setting. Advanced pasta expressions such as addresses, `all`, `auto`, exclusions, and comma-containing specifications are not supported.
- Use `127.0.0.1` inside the container for an IPv4-only host service. `localhost` can resolve to `::1`, which forwards to host IPv6 loopback instead.
- Exposing a host port deliberately grants container processes access to that host-loopback service. It is separate from HTTP reverse proxying and from `ports`, which publish container services to the host.
- `mounts`, `caches`, `envs`, `env_files`, `ports`, `dynamic_ports`, `expose_host_ports`, `podman_args`, `run_args`, and `ssh_keys` extend the base settings when enabled.
- `command`, `image`, `mount_git_metadata`, and `enable_ssh` use the highest-precedence value.
- Environment values support the `${project}` placeholder, which is replaced with the basename of Dungeon's invocation directory. Expansion applies only to the value in `NAME=VALUE` entries; shell expressions such as `$(basename $PWD)` remain literal.

### libkrun

The provided image can run as a libkrun microVM. Configure an opt-in group with resource limits appropriate for the host:

```toml
[krun]
run_args = [
  "--runtime=krun",
  "--annotation=krun.ram_mib=4096",
  "--memory=5g",
]
```

Start it with `dungeon run --krun`. `krun.ram_mib` sets guest RAM; `--memory` also leaves room for the runtime outside the guest.

The image does not request a virtual network interface, so libkrun uses Transparent Socket Impersonation (TSI). Guest TCP and UDP operations are proxied by the VMM from Podman's network namespace. This avoids a nested passt process and allows `expose_host_ports` to work under krun without an additional annotation.

TSI keeps the KVM boundary and Podman's user, mount, and network namespaces, but Dungeon does not filter egress inside the guest. The workload can make any TCP, UDP, DNS, IPv4, or IPv6 connection allowed by the outer Podman network. TSI can also proxy absolute-path Unix sockets visible and accessible in the VMM's mount namespace, so do not mount sensitive engine, SSH-agent, or desktop sockets unless that access is intentional.

The Linux system running Podman locally must provide KVM access, a krun-enabled crun runtime, and libkrunfw. Exposed host ports additionally require pasta. Podman's remote client, including Podman machines selected with `-c` or `--connection`, does not support `--runtime`; run this configuration against a local Podman service instead.

### Environment variables

Environment overrides use:
- `DUNGEON_COMMAND`
- `DUNGEON_IMAGE`
- `DUNGEON_PORTS` (comma-separated)
- `DUNGEON_DYNAMIC_PORTS` (comma-separated)
- `DUNGEON_EXPOSE_HOST_PORTS` (comma-separated)
- `DUNGEON_CACHES` (comma-separated cache specs using the same syntax as `caches`)
- `DUNGEON_MOUNTS` (comma-separated)
- `DUNGEON_ENVS` (comma-separated)
- `DUNGEON_ENV_FILES` (comma-separated)
- `DUNGEON_SECRETS` (comma-separated)
- `DUNGEON_PODMAN_ARGS` (comma-separated)
- `DUNGEON_RUN_ARGS` (comma-separated)
- `DUNGEON_MOUNT_GIT_METADATA`
- `DUNGEON_ENABLE_SSH`
- `DUNGEON_SSH_KEYS` (newline-separated literal public-key lines)
- `DUNGEON_INCLUDE_GROUPS` (comma-separated)

### SSH access

Enable SSH in a configuration group when a runtime such as `krun` cannot support `podman exec`:

```toml
[ssh]
enable_ssh = true
ssh_keys = ["ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAA... you@example.com"]
```

Start with `dungeon run --ssh`. Once `sshd` is ready, the bootstrap prints the exact command to connect as `dungeon` and sends the same command to the terminal clipboard using OSC 52. Clipboard support depends on the host terminal emulator and its OSC 52 policy; the printed command remains the fallback.

SSH listens only through a randomly selected `127.0.0.1` host port. Password, keyboard-interactive, root, agent forwarding, TCP forwarding, X11 forwarding, user-provided SSH environment, and tunneling are disabled. SSH host keys and `authorized_keys` exist only for the temporary container lifetime. The configured public keys appear in the generated Podman environment and are not private credentials.

Interactive SSH logins automatically create or attach to a Zellij session. Zellij does not start for Dungeon's primary shell, non-interactive SSH commands, or nested Zellij sessions.

## Runtime behavior

- `dungeon run` starts the minimal image entrypoint as namespaced root so it can retain the capability bounding set required by setuid tools, then switches to the unprivileged `dungeon` user.
- Dungeon does not add capabilities or install an in-container firewall.
- The runtime intentionally preserves the image's narrow `sudo dungeon-install ...` path; broader root access still is not granted.
- The Podman command uses `--userns=keep-id --user root`; `dungeon-bootstrap` is the only root process. When SSH is enabled, it creates the temporary credentials, starts `sshd`, and then hands control to the image's `dungeon` user.
- The bootstrap starts no service unless `DUNGEON_SSH_ENABLED=true`. It does not add a firewall.
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

## Cache

A named volume `dungeon-cache` is used for caches. This lets you mount specific folders to cache between temporary sessions. This is typically used to speed up installing dependencies. Use `dungeon cache init` to create configured subdirectories before using `subpath` cache entries.

Reset it with:

```shell
dungeon cache reset
```

Reset removes the entire volume, including all initialized subdirectories. Re-run `dungeon cache init` before using subpath cache entries.

## License

See [LICENSE](LICENSE)
