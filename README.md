# Rootless Podman inside a krun microVM

## Goals

- Preserve the existing disposable, rootless Podman sandbox and its launcher flags,
  mounts, secrets, and caches.
- Run the agent as a non-root guest user, with rootless Podman for image builds,
  Compose services, DNS, volumes, bind mounts, and outbound networking.
- Publish selected guest ports on the physical host, including loopback-only binds.
- Grant selected host-local TCP services without exposing every host-local service.
- Avoid privileged containers, host container-engine sockets, and another nested VM.

## Implementation

- **Outer rootless Podman:** OCI lifecycle, user mappings, mounts, and explicit ports.
- **krun / libkrun / KVM:** a microVM with its own **libkrunfw** Linux kernel.
- **pasta:** connects the outer container's network namespace to the host network.
- **passt / virtio-net:** gives the guest a conventional NIC (`krun.use_passt=1`).
- **virtiofs:** exposes the image filesystem and explicitly mounted host paths.
- **Arch Linux, Podman, crun:** guest userspace and inner rootless containers.
- **Netavark / Aardvark / nftables:** inner bridge networks, service DNS, and NAT.
- **VFS / guest tmpfs:** image storage and Buildah temporary files; currently 2 GiB
  shared by images, writable layers, named volumes, and temporary files.
- **FUSE / fuse-overlayfs:** available in the base image, with guest FUSE device
  access for `agent`. Tool-specific storage configuration is opt-in (see below).
- **Compose:** installed by the agent when needed; no provider or version is forced.
- **Optional socat relay:** forwards an explicitly selected host service from an
  outer-namespace listener to a `pasta -T` endpoint. Currently managed manually.

## Security boundaries and limitations

- **Kernel boundary:** inner containers share the guest kernel, not the physical
  host kernel. Guest namespaces provide additional separation within the VM.
- **Filesystem boundary:** writable mounts deliberately permit host-file changes;
  read-only mounts permit reading. Credentials and sockets are shared only when
  explicitly provided. Do not mount a host container-engine socket.
- **Identity:** `--userns=keep-id` preserves a user namespace while mapping the
  agent to the calling host user. Startup runs as guest root, then drops to
  `agent` (UID/GID 1000). Agent and inner-container-root writes to a physical-host
  bind mount were verified as host UID/GID 1000. Other host IDs need adjusted mappings.
- **Network boundary:** `pasta` and `passt` are part of the isolation boundary.
  Inner publishing terminates at the guest; physical-host publication requires
  an outer `-p`. This is not an outbound firewall: Internet and LAN access remain.
- **Host-local access:** selected-service forwarding and denial of a live,
  unapproved IPv4 loopback service were tested from guest and inner containers.
  IPv6 loopback/link-local probes also failed to reach the unapproved service;
  global IPv6 was unavailable. These are bounded tests, not proof of every path.
- **Resources:** inner cgroups are disabled, so per-container resource limits are
  not established. Default container storage is RAM-backed and disappears with
  the VM. Optional persistent storage is described below.
- Synthetic Compose builds, DNS, ports, bind mounts, and named volumes passed;
  compatibility with a representative existing project and launcher integration
  remain unverified.

## Run the minimal sandbox

### Host prerequisites

- Working rootless Podman, accessible KVM, `krun`, `libkrun`, `libkrunfw`, and
  `pasta`/`passt`.
- Guest kernel built with `CONFIG_NF_CONNTRACK_MARK=y`. The tested libkrunfw 5.6.2
  required rebuilding; `libkrunfw-mark.patch` patches its Arch PKGBUILD.
- A compatible host runtime/networking combination. The tested krun 1.30.1 has a
  passt startup regression: use a compatible older host passt or a runtime with
  the [upstream /proc fix](https://github.com/containers/crun/commit/ca04fdb6c26f0c8259871f2cccd0e6e652ec03ce).

Build on the host, from this directory:

```bash
podman build -t localhost/krun-rootless-test -f Containerfile .
```

Launch on the host as your ordinary user (tested with UID/GID 1000):

```bash
podman run --rm -it \
  --name krun-rootless-test \
  --userns=keep-id \
  --user=0:0 \
  --runtime=krun \
  --annotation=krun.use_passt=1 \
  --annotation=krun.ram_mib=4096 \
  --network=pasta \
  -p 127.0.0.1:18080:8080 \
  localhost/krun-rootless-test
```

The entrypoint sets guest TUN/FUSE permissions, mounts guest tmpfs storage, and opens
a shell as `agent`. This command shares no project directory or host-service relay.
To share the current host directory, add `-v "$PWD:/workspace:rw" -w /workspace`
before the image name.

Inside the guest:

```bash
id
podman run --rm docker.io/library/alpine:latest \
  sh -ec 'id; wget -T 15 -O /dev/null https://example.com'

podman run -d --name web -p 8080:80 docker.io/library/nginx:alpine
curl --fail http://127.0.0.1:8080/
```

From another physical-host terminal:

```bash
curl --fail http://127.0.0.1:18080/
```

Stop nginx with `podman stop web` inside the guest. Exit the guest shell to dispose
of the VM and its container storage; changes to writable host mounts remain.

## Optional: persistent Podman image storage

Ordinary package caches generally need a volume mount and appropriate ownership.
Podman additionally needs a storage driver that handles virtiofs permissions:
VFS extraction failed, while `fuse-overlayfs` with `force_mask = "private"`
successfully downloaded and ran httpd, then reused it in a fresh VM without
registry requests. This recipe is tool-specific, not project-specific.

Create a host file named `podman-storage.conf`:

```toml
[storage]
driver = "overlay"
graphroot = "/cache/podman/overlay-force-mask"
runroot = "/run/agent-podman/runtime/storage"

[storage.options.overlay]
mount_program = "/usr/bin/fuse-overlayfs"
force_mask = "private"
```

Mount the volume and configuration, select it explicitly, and initialize directory
ownership before the base entrypoint drops privileges:

```bash
podman run --rm -it \
  --userns=keep-id \
  --user=0:0 \
  --runtime=krun \
  --annotation=krun.use_passt=1 \
  --annotation=krun.ram_mib=4096 \
  --network=pasta \
  -v podman-cache:/cache/podman \
  -v "$PWD/podman-storage.conf:/etc/containers/podman-cache.conf:ro" \
  -e CONTAINERS_STORAGE_CONF=/etc/containers/podman-cache.conf \
  --entrypoint /bin/bash \
  localhost/krun-rootless-test -ec '
    install -d -o agent -g agent -m 0700 /cache/podman/overlay-force-mask
    exec /usr/local/bin/guest-entrypoint /bin/bash
  '
```

Inside, use normal commands such as `podman run --rm httpd`. The volume persists
the full graph root (including images and named volumes); runtime and build
temporary files stay disposable. Use one active VM per writable store volume.
Builds and Compose were tested with the default store; their regression checks
with this cached configuration remain pending.
