FROM docker.io/library/archlinux:base

# The guest kernel must provide CONFIG_NF_CONNTRACK_MARK=y for Netavark.
RUN pacman -Syu --noconfirm --needed \
      podman crun shadow passt slirp4netns fuse-overlayfs \
      iproute2 iptables-nft nftables ca-certificates curl python python-pip \
    && pacman -Scc --noconfirm

RUN python -m venv /opt/compose \
    && /opt/compose/bin/python -m pip install --no-cache-dir 'podman-compose==1.5.0'

RUN chmod 4755 /usr/bin/newuidmap /usr/bin/newgidmap

RUN useradd --create-home --user-group --uid 1000 --shell /bin/bash agent \
    && touch /etc/subuid /etc/subgid \
    && sed -i '/^agent:/d' /etc/subuid /etc/subgid \
    && printf 'agent:100000:65536\n' >> /etc/subuid \
    && printf 'agent:100000:65536\n' >> /etc/subgid \
    && install -d -o agent -g agent /home/agent/.config/containers \
    && printf '[storage]\ndriver = "vfs"\ngraphroot = "/run/agent-podman/graph"\nrunroot = "/run/agent-podman/runtime/storage"\n' \
         > /etc/containers/agent-storage.conf \
    && printf '[engine]\nruntime = "crun"\ncgroup_manager = "cgroupfs"\nevents_logger = "file"\n[containers]\ncgroups = "disabled"\nlog_driver = "k8s-file"\n' \
         > /home/agent/.config/containers/containers.conf \
    && chown -R agent:agent /home/agent/.config

COPY guest-entrypoint.sh /usr/local/bin/guest-entrypoint
RUN chmod 0755 /usr/local/bin/guest-entrypoint

ENV HOME=/home/agent \
    XDG_RUNTIME_DIR=/run/agent-podman/runtime \
    TMPDIR=/run/agent-podman/tmp \
    CONTAINERS_STORAGE_CONF=/etc/containers/agent-storage.conf \
    PODMAN_COMPOSE_PROVIDER=/opt/compose/bin/podman-compose

USER root
WORKDIR /home/agent
ENTRYPOINT ["/usr/local/bin/guest-entrypoint"]
CMD ["/bin/bash"]
