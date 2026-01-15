package config

import (
	"fmt"

	"github.com/pelletier/go-toml/v2"
)

func parseConfig(data []byte) (Config, error) {
	var raw map[string]interface{}
	if err := toml.Unmarshal(data, &raw); err != nil {
		return Config{}, err
	}

	cfg := Config{}
	reserved := map[string]bool{
		"run":            true,
		"image":          true,
		"ports":          true,
		"cache":          true,
		"mounts":         true,
		"envvar":         true,
		"podman_args":    true,
		"default_groups": true,
	}

	for key, value := range raw {
		switch key {
		case "run":
			run, err := parseStringField("run", value)
			if err != nil {
				return Config{}, err
			}
			cfg.RunCommand = run
		case "image":
			image, err := parseStringField("image", value)
			if err != nil {
				return Config{}, err
			}
			cfg.Image = image
		case "ports":
			values, err := parseStringSliceField("ports", value)
			if err != nil {
				return Config{}, err
			}
			cfg.Ports = values
		case "cache":
			values, err := parseStringSliceField("cache", value)
			if err != nil {
				return Config{}, err
			}
			cfg.Cache = values
		case "mounts":
			values, err := parseStringSliceField("mounts", value)
			if err != nil {
				return Config{}, err
			}
			cfg.Mounts = values
		case "envvar":
			values, err := parseStringSliceField("envvar", value)
			if err != nil {
				return Config{}, err
			}
			cfg.EnvVars = values
		case "podman_args":
			values, err := parseStringSliceField("podman_args", value)
			if err != nil {
				return Config{}, err
			}
			cfg.PodmanArgs = values
		case "default_groups":
			values, err := parseStringSliceField("default_groups", value)
			if err != nil {
				return Config{}, err
			}
			cfg.DefaultGroups = values
		default:
			if !reserved[key] {
				group, err := parseGroupConfig(key, value)
				if err != nil {
					return Config{}, err
				}
				if cfg.Groups == nil {
					cfg.Groups = map[string]GroupConfig{}
				}
				cfg.Groups[key] = group
			}
		}
	}

	return cfg, nil
}

func cloneGroupMap(values map[string]GroupConfig) map[string]GroupConfig {
	if values == nil {
		return map[string]GroupConfig{}
	}
	clone := make(map[string]GroupConfig, len(values))
	for key, value := range values {
		clone[key] = value
	}
	return clone
}

func parseGroupConfig(name string, value interface{}) (GroupConfig, error) {
	raw, ok := value.(map[string]interface{})
	if !ok {
		return GroupConfig{}, fmt.Errorf("group %q must be a table", name)
	}

	group := GroupConfig{}
	if len(raw) == 0 {
		group.Disabled = true
		return group, nil
	}

	for key, value := range raw {
		switch key {
		case "mounts":
			values, err := parseStringSliceField(name+".mounts", value)
			if err != nil {
				return GroupConfig{}, err
			}
			group.Mounts = values
		case "cache":
			values, err := parseStringSliceField(name+".cache", value)
			if err != nil {
				return GroupConfig{}, err
			}
			group.Cache = values
		case "envvar":
			values, err := parseStringSliceField(name+".envvar", value)
			if err != nil {
				return GroupConfig{}, err
			}
			group.EnvVars = values
		case "run":
			run, err := parseStringField(name+".run", value)
			if err != nil {
				return GroupConfig{}, err
			}
			group.RunCommand = run
		case "image":
			image, err := parseStringField(name+".image", value)
			if err != nil {
				return GroupConfig{}, err
			}
			group.Image = image
		case "ports":
			values, err := parseStringSliceField(name+".ports", value)
			if err != nil {
				return GroupConfig{}, err
			}
			group.Ports = values
		case "podman_args":
			values, err := parseStringSliceField(name+".podman_args", value)
			if err != nil {
				return GroupConfig{}, err
			}
			group.PodmanArgs = values
		default:
			return GroupConfig{}, fmt.Errorf("group %q has unknown key %q", name, key)
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
