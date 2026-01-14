package config

import (
	"fmt"
	"strings"
)

func BuildGroupDefaults(cfg Config) (map[string]GroupConfig, map[string]bool, error) {
	groups := cloneGroupMap(cfg.Groups)
	for name, group := range cfg.Groups {
		trimmed := strings.TrimSpace(name)
		if trimmed == "" {
			return nil, nil, fmt.Errorf("ERROR: group name cannot be empty")
		}
		groups[trimmed] = group
	}

	enabled := make(map[string]bool, len(groups))
	for name := range groups {
		enabled[name] = false
	}
	for _, name := range cfg.DefaultGroups {
		trimmed := strings.TrimSpace(name)
		if trimmed == "" {
			continue
		}
		if _, ok := groups[trimmed]; !ok {
			return nil, nil, fmt.Errorf("ERROR: default_groups includes unknown group %q", trimmed)
		}
		enabled[trimmed] = true
	}

	return groups, enabled, nil
}
