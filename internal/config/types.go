package config

// Settings captures the configuration options shared across sources.
// The TOML tags map directly to config keys.
type Settings struct {
	// The container command to execute.
	RunCommand string `toml:"run"`

	// The container image to run.
	Image string `toml:"image"`

	// Host:container port bindings.
	Ports []string `toml:"ports"`

	// The list of cache volume definitions.
	Cache []string `toml:"cache"`

	// The list of volume bindings.
	Mounts []string `toml:"mounts"`

	// The list of environment variables.
	EnvVars []string `toml:"envvar"`

	// Additional podman CLI args.
	PodmanArgs []string `toml:"podman_args"`
}

// Config captures the parsed configuration from defaults and user files.
// The TOML tags map directly to top-level config keys.
type Config struct {
	Settings

	// The group names enabled by default.
	DefaultGroups []string `toml:"default_groups"`

	// Named preset configurations.
	Groups map[string]GroupConfig
}

// GroupConfig defines the values allowed in a group table.
type GroupConfig struct {
	Settings

	// Disabled marks that the group should be removed.
	Disabled bool `toml:"-"`
}
