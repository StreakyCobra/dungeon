package main

import (
	"sort"
	"strings"

	"github.com/StreakyCobra/dungeon/internal/config"
)

func optionsFromSettings(settings config.Settings) options {
	opts := options{
		runCommand: settings.RunCommand,
		image:      strings.TrimSpace(settings.Image),
		ports:      settings.Ports,
		cache:      settings.Cache,
		mounts:     settings.Mounts,
		envVars:    settings.EnvVars,
		podmanArgs: settings.PodmanArgs,
	}

	return opts
}

func sortedGroupNames(values map[string]config.GroupConfig) []string {
	keys := make([]string, 0, len(values))
	for key := range values {
		keys = append(keys, key)
	}
	sort.Strings(keys)
	return keys
}
