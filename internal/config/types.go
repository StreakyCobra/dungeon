package config

type Config struct {
	RunCommand    string   `toml:"run"`
	DefaultGroups []string `toml:"default_groups"`
	Image         string   `toml:"image"`
	Ports         []string `toml:"ports"`
	Network       string   `toml:"network"`
	Name          string   `toml:"name"`
	Remove        *bool    `toml:"rm"`
	PodmanArgs    []string `toml:"podman_args"`
	Groups        map[string]GroupConfig
}

type GroupConfig struct {
	Mounts []string `toml:"mounts"`
	Cache  []string `toml:"cache"`
	Env    []string `toml:"env"`
}
