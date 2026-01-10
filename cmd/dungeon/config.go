package main

import (
	"fmt"
	"os"
	"path/filepath"
	"sort"
	"strings"

	"github.com/pelletier/go-toml/v2"
)

type config struct {
	RunCommand    string   `toml:"run"`
	DefaultGroups []string `toml:"default_groups"`
	Image         string   `toml:"image"`
	Ports         []string `toml:"ports"`
	Network       string   `toml:"network"`
	Name          string   `toml:"name"`
	Remove        *bool    `toml:"rm"`
	PodmanArgs    []string `toml:"podman_args"`
	Groups        map[string]groupConfig
}

type groupConfig struct {
	Mounts []string `toml:"mounts"`
	Cache  []string `toml:"cache"`
	Env    []string `toml:"env"`
}

func loadDefaults() (options, error) {
	base, err := loadDefaultConfig()
	if err != nil {
		return options{}, err
	}

	cfg, _, err := loadConfigWithBase(base)
	if err != nil {
		return options{}, err
	}

	groupSpecs, groupOn, err := buildGroupDefaults(cfg)
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
	} else {
		opts.remove = *cfg.Remove
	}
	opts.removeSet = cfg.Remove != nil

	return opts, nil
}

func buildGroupDefaults(cfg config) (map[string]groupConfig, map[string]bool, error) {
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

func loadConfigWithBase(base config) (config, string, error) {
	path, err := configPath()
	if err != nil {
		return config{}, "", err
	}

	data, err := os.ReadFile(path)
	if err != nil {
		if os.IsNotExist(err) {
			return base, path, nil
		}
		return config{}, path, fmt.Errorf("read config %s: %w", path, err)
	}

	cfg, err := parseConfigWithBase(data, base)
	if err != nil {
		return config{}, path, fmt.Errorf("parse config %s: %w", path, err)
	}

	return cfg, path, nil
}

func parseConfig(data []byte) (config, error) {
	return parseConfigWithBase(data, config{})
}

func parseConfigWithBase(data []byte, base config) (config, error) {
	var raw map[string]interface{}
	if err := toml.Unmarshal(data, &raw); err != nil {
		return config{}, err
	}

	cfg := base
	if cfg.Groups == nil {
		cfg.Groups = map[string]groupConfig{}
	} else {
		cfg.Groups = cloneGroupMap(cfg.Groups)
	}

	reserved := map[string]bool{
		"run":            true,
		"image":          true,
		"ports":          true,
		"network":        true,
		"name":           true,
		"rm":             true,
		"podman_args":    true,
		"default_groups": true,
		"mount_defaults": true,
	}

	for key, value := range raw {
		switch key {
		case "run":
			run, err := parseStringField("run", value)
			if err != nil {
				return config{}, err
			}
			cfg.RunCommand = run
		case "image":
			image, err := parseStringField("image", value)
			if err != nil {
				return config{}, err
			}
			cfg.Image = image
		case "ports":
			values, err := parseStringSliceField("ports", value)
			if err != nil {
				return config{}, err
			}
			cfg.Ports = values
		case "network":
			network, err := parseStringField("network", value)
			if err != nil {
				return config{}, err
			}
			cfg.Network = network
		case "name":
			name, err := parseStringField("name", value)
			if err != nil {
				return config{}, err
			}
			cfg.Name = name
		case "rm":
			remove, err := parseBoolField("rm", value)
			if err != nil {
				return config{}, err
			}
			cfg.Remove = &remove
		case "podman_args":
			values, err := parseStringSliceField("podman_args", value)
			if err != nil {
				return config{}, err
			}
			cfg.PodmanArgs = values
		case "default_groups":
			values, err := parseStringSliceField("default_groups", value)
			if err != nil {
				return config{}, err
			}
			cfg.DefaultGroups = values
		case "mount_defaults":
			values, err := parseStringSliceField("mount_defaults", value)
			if err != nil {
				return config{}, err
			}
			if len(cfg.DefaultGroups) > 0 {
				return config{}, fmt.Errorf("default_groups and mount_defaults are both set")
			}
			cfg.DefaultGroups = values
		default:
			if !reserved[key] {
				group, err := parseGroupConfig(key, value)
				if err != nil {
					return config{}, err
				}
				cfg.Groups[key] = group
			}
		}
	}

	return cfg, nil
}

func cloneGroupMap(values map[string]groupConfig) map[string]groupConfig {
	if values == nil {
		return map[string]groupConfig{}
	}
	clone := make(map[string]groupConfig, len(values))
	for key, value := range values {
		clone[key] = value
	}
	return clone
}

func parseGroupConfig(name string, value interface{}) (groupConfig, error) {
	raw, ok := value.(map[string]interface{})
	if !ok {
		return groupConfig{}, fmt.Errorf("group %q must be a table", name)
	}

	group := groupConfig{}
	for key, value := range raw {
		switch key {
		case "mounts":
			values, err := parseStringSliceField(name+".mounts", value)
			if err != nil {
				return groupConfig{}, err
			}
			group.Mounts = values
		case "cache":
			values, err := parseStringSliceField(name+".cache", value)
			if err != nil {
				return groupConfig{}, err
			}
			group.Cache = values
		case "env":
			values, err := parseStringSliceField(name+".env", value)
			if err != nil {
				return groupConfig{}, err
			}
			group.Env = values
		default:
			return groupConfig{}, fmt.Errorf("group %q has unknown key %q", name, key)
		}
	}

	return group, nil
}

func parseStringField(name string, value interface{}) (string, error) {
	str, ok := value.(string)
	if !ok {
		return "", fmt.Errorf("%s must be a string", name)
	}
	return str, nil
}

func parseBoolField(name string, value interface{}) (bool, error) {
	boolean, ok := value.(bool)
	if !ok {
		return false, fmt.Errorf("%s must be a boolean", name)
	}
	return boolean, nil
}

func parseStringSliceField(name string, value interface{}) ([]string, error) {
	switch v := value.(type) {
	case []string:
		return v, nil
	case []interface{}:
		values := make([]string, 0, len(v))
		for _, item := range v {
			str, ok := item.(string)
			if !ok {
				return nil, fmt.Errorf("%s must be a list of strings", name)
			}
			values = append(values, str)
		}
		return values, nil
	default:
		return nil, fmt.Errorf("%s must be a list of strings", name)
	}
}

func configPath() (string, error) {
	configHome := os.Getenv("XDG_CONFIG_HOME")
	if configHome == "" {
		home, err := os.UserHomeDir()
		if err != nil {
			return "", err
		}
		configHome = filepath.Join(home, ".config")
	}
	return filepath.Join(configHome, "dungeon", "config.toml"), nil
}

func sortedGroupNames(values map[string]groupConfig) []string {
	keys := make([]string, 0, len(values))
	for key := range values {
		keys = append(keys, key)
	}
	sort.Strings(keys)
	return keys
}
