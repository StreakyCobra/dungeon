package config

func MergeSettings(base, override Settings) Settings {
	merged := base
	if override.RunCommand != "" {
		merged.RunCommand = override.RunCommand
	}
	if override.Image != "" {
		merged.Image = override.Image
	}
	if override.Ports != nil {
		merged.Ports = appendStrings(base.Ports, override.Ports)
	}
	if override.Cache != nil {
		merged.Cache = appendStrings(base.Cache, override.Cache)
	}
	if override.Mounts != nil {
		merged.Mounts = appendStrings(base.Mounts, override.Mounts)
	}
	if override.EnvVars != nil {
		merged.EnvVars = appendStrings(base.EnvVars, override.EnvVars)
	}
	if override.PodmanArgs != nil {
		merged.PodmanArgs = appendStrings(base.PodmanArgs, override.PodmanArgs)
	}

	return merged
}

func MergeAlwaysOnGroups(base []string, override []string) []string {
	if override == nil {
		return base
	}
	return appendStrings(base, override)
}

func appendStrings(base []string, extra []string) []string {
	if base == nil && extra == nil {
		return nil
	}
	merged := append([]string{}, base...)
	return append(merged, extra...)
}
