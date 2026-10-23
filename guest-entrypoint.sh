#!/bin/bash
set -euo pipefail

# This is the guest's devtmpfs device, not a physical-host device mount.
chgrp agent /dev/net/tun
chmod 0660 /dev/net/tun

# Keep image extraction and Buildah temporary files off virtiofs.
# Images and named volumes last only for this microVM session.
mkdir -p /run/agent-podman
mount -t tmpfs -o size=2g,mode=0755 tmpfs /run/agent-podman
install -d -o agent -g agent -m 0700 \
  /run/agent-podman/graph \
  /run/agent-podman/runtime \
  /run/agent-podman/tmp

exec /usr/bin/runuser -u agent -- "$@"
