package config

// Config captures the parsed configuration from defaults and user files.
// The TOML tags map directly to top-level config keys.
type Config struct {
	// The container command to execute.
	RunCommand string `toml:"run"`

	// The group names enabled by default.
	DefaultGroups []string `toml:"default_groups"`

	// The container image to run.
	Image string `toml:"image"`

	// Host:container port bindings.
	Ports []string `toml:"ports"`

	// The list of cache volume definitions.
	Cache []string `toml:"cache"`

	// Whether to persist the container after exit.
	Persist *bool `toml:"persist"`

	// Additional podman CLI args.
	PodmanArgs []string `toml:"podman_args"`

	// Named mount/cache/envvar sets.
	Groups map[string]GroupConfig
}

// GroupConfig defines the values allowed in a group table.
type GroupConfig struct {
	// The list of volume bindings for the group.
	Mounts []string `toml:"mounts"`

	// The list of cache volume definitions.
	Cache []string `toml:"cache"`

	// The list of environment variables for the group.
	EnvVars []string `toml:"envvar"`

	// The container command to execute when enabled.
	RunCommand string `toml:"run"`

	// The container image to run when enabled.
	Image string `toml:"image"`

	// Host:container port bindings for the group.
	Ports []string `toml:"ports"`
}
