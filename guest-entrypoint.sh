#!/bin/bash
set -euo pipefail

# These are guest devtmpfs devices, not physical-host device mounts.
chgrp agent /dev/net/tun /dev/fuse
chmod 0660 /dev/net/tun /dev/fuse

# Keep runtime and Buildah temporary files on guest tmpfs.
# Default image storage also lives here; optional tool configuration may override it.
mkdir -p /run/agent-podman
mount -t tmpfs -o size=2g,mode=0755 tmpfs /run/agent-podman
install -d -o agent -g agent -m 0700 \
  /run/agent-podman/graph \
  /run/agent-podman/runtime \
  /run/agent-podman/tmp

exec /usr/bin/runuser -u agent -- "$@"
