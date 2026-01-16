package config

import "fmt"

type Sources struct {
	Defaults Config
	File     Config
	Env      Config
	CLI      Settings
}

func ResolveAlwaysOnGroups(defaults Config, file Config, env Config) []string {
	groups := MergeAlwaysOnGroups(nil, defaults.AlwaysOnGroups)
	groups = MergeAlwaysOnGroups(groups, file.AlwaysOnGroups)
	groups = MergeAlwaysOnGroups(groups, env.AlwaysOnGroups)
	return groups
}

func ResolveSettings(sources Sources, groups map[string]GroupConfig, groupOrder []string) (Settings, error) {
	settings := sources.Defaults.Settings
	for _, name := range groupOrder {
		group, ok := groups[name]
		if !ok {
			return Settings{}, fmt.Errorf("ERROR: unknown group %q", name)
		}
		settings = MergeSettings(settings, group.Settings)
	}
	settings = MergeSettings(settings, sources.File.Settings)
	settings = MergeSettings(settings, sources.Env.Settings)
	settings = MergeSettings(settings, sources.CLI)
	return settings, nil
}
