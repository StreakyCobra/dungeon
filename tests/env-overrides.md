# Env Overrides

## Config
```toml
run = "bash -l"
image = "localhost/base"
ports = ["127.0.0.1:1111:1111"]
```

## Env
```
DUNGEON_RUN=codex
DUNGEON_IMAGE=localhost/from-env
DUNGEON_PORTS=127.0.0.1:2222:2222,127.0.0.1:3333:3333
DUNGEON_CACHES=workspace-cache:ro
DUNGEON_MOUNTS=~/extra:/home/dungeon/extra:rw
DUNGEON_ENVS=ENV_ONE=1,ENV_TWO=two
DUNGEON_PODMAN_ARGS=--tz=UTC
```

## CLI
```
dungeon
```

## Expected
```toml
run = "codex"
image = "localhost/from-env"
ports = ["127.0.0.1:1111:1111", "127.0.0.1:2222:2222", "127.0.0.1:3333:3333"]
caches = ["workspace-cache:ro"]
mounts = ["~/extra:/home/dungeon/extra:rw"]
envs = ["ENV_ONE=1", "ENV_TWO=two"]
podman_args = ["--tz=UTC"]
```
