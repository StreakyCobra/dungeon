# CLI Overrides

## Config
```toml
run = "bash -l"
image = "localhost/base"
ports = ["127.0.0.1:1111:1111"]
caches = ["workspace-cache:rw"]
```

## Env
```
DUNGEON_IMAGE=localhost/from-env
DUNGEON_PORTS=127.0.0.1:2222:2222
```

## CLI
```
dungeon --run zsh --image localhost/cli --port 127.0.0.1:3333:3333 --cache workspace-cache:ro --mount ~/cli:/home/dungeon/cli:rw --env CLI_ONE=1 --podman-arg --log-level=debug
```

## Expected
```toml
run = "zsh"
image = "localhost/cli"
ports = ["127.0.0.1:1111:1111", "127.0.0.1:2222:2222", "127.0.0.1:3333:3333"]
caches = ["workspace-cache:rw", "workspace-cache:ro"]
mounts = ["~/cli:/home/dungeon/cli:rw"]
envs = ["CLI_ONE=1"]
podman_args = ["--log-level=debug"]
```
