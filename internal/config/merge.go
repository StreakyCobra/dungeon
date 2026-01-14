package config

func Merge(base, override Config) Config {
	merged := base
	if override.RunCommand != "" {
		merged.RunCommand = override.RunCommand
	}
	if override.Image != "" {
		merged.Image = override.Image
	}
	if override.Network != "" {
		merged.Network = override.Network
	}
	if override.Name != "" {
		merged.Name = override.Name
	}
	if override.Ports != nil {
		merged.Ports = append([]string{}, override.Ports...)
	}
	if override.PodmanArgs != nil {
		merged.PodmanArgs = append([]string{}, override.PodmanArgs...)
	}
	if override.DefaultGroups != nil {
		merged.DefaultGroups = append([]string{}, override.DefaultGroups...)
	}
	if override.Remove != nil {
		value := *override.Remove
		merged.Remove = &value
	}
	if override.Groups != nil {
		groups := cloneGroupMap(merged.Groups)
		for name, group := range override.Groups {
			groups[name] = group
		}
		merged.Groups = groups
	}

	return merged
}

func Reduce(configs ...Config) Config {
	merged := Config{}
	for _, cfg := range configs {
		merged = Merge(merged, cfg)
	}
	return merged
}
