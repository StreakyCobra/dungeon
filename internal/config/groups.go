package config

import (
	"fmt"
	"strings"
)

func MergeGroupDefinitions(base map[string]GroupConfig, overrides map[string]GroupConfig) (map[string]GroupConfig, error) {
	merged := map[string]GroupConfig{}
	for name, group := range base {
		trimmed, err := normalizeGroupName(name)
		if err != nil {
			return nil, err
		}
		merged[trimmed] = group
	}
	for name, group := range overrides {
		trimmed, err := normalizeGroupName(name)
		if err != nil {
			return nil, err
		}
		if group.Disabled {
			delete(merged, trimmed)
			continue
		}
		group.Disabled = false
		merged[trimmed] = group
	}

	return merged, nil
}

func BuildGroupSelection(groups map[string]GroupConfig, defaultGroups []string) (map[string]bool, error) {
	enabled := make(map[string]bool, len(groups))
	for name := range groups {
		enabled[name] = false
	}
	for _, name := range defaultGroups {
		trimmed, err := normalizeGroupName(name)
		if err != nil {
			return nil, err
		}
		if _, ok := groups[trimmed]; !ok {
			return nil, fmt.Errorf("ERROR: always_on_groups includes unknown group %q", trimmed)
		}
		enabled[trimmed] = true
	}
	return enabled, nil
}

func NormalizeGroupOrder(groups []string) ([]string, error) {
	normalized := make([]string, 0, len(groups))
	for _, name := range groups {
		trimmed, err := normalizeGroupName(name)
		if err != nil {
			return nil, err
		}
		normalized = append(normalized, trimmed)
	}
	return normalized, nil
}

func normalizeGroupName(name string) (string, error) {
	trimmed := strings.TrimSpace(name)
	if trimmed == "" {
		return "", fmt.Errorf("ERROR: group name cannot be empty")
	}
	return trimmed, nil
}
