# Group Overrides

## Config
```toml
always_on_groups = ["extras"]

[extras]
mounts = ["~/extras:/home/dungeon/extras:rw"]

[expose]
ports = ["127.0.0.1:3333:3333"]
```

## Env
```
a=12
```

## CLI
```
dungeon --expose --extras
```

## Expected
```toml
mounts = ["~/extras:/home/dungeon/extras:rw"]
ports = ["127.0.0.1:3333:3333"]
```
