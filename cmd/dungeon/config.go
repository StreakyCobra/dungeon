package main

import (
	"sort"
	"strings"

	"github.com/StreakyCobra/dungeon/internal/config"
)

func optionsFromConfig(cfg config.Config) (options, error) {
	groupSpecs, groupOn, err := config.BuildGroupDefaults(cfg)
	if err != nil {
		return options{}, err
	}

	opts := options{
		runCommand: cfg.RunCommand,
		groupSpecs: groupSpecs,
		groupOn:    groupOn,
		image:      strings.TrimSpace(cfg.Image),
		ports:      cfg.Ports,
		network:    strings.TrimSpace(cfg.Network),
		name:       strings.TrimSpace(cfg.Name),
		podmanArgs: cfg.PodmanArgs,
	}
	if cfg.Remove == nil {
		opts.remove = true
		if opts.name != "" {
			opts.remove = false
		}
	} else {
		opts.remove = *cfg.Remove
	}
	opts.removeSet = cfg.Remove != nil

	return opts, nil
}

func sortedGroupNames(values map[string]config.GroupConfig) []string {
	keys := make([]string, 0, len(values))
	for key := range values {
		keys = append(keys, key)
	}
	sort.Strings(keys)
	return keys
}
