package config

import (
	"os"
	"strings"
)

const envPrefix = "DUNGEON_"

func LoadFromEnv() (Config, error) {
	return LoadFromEnvLookup(os.LookupEnv)
}

func LoadFromEnvMap(values map[string]string) (Config, error) {
	return LoadFromEnvLookup(func(key string) (string, bool) {
		value, ok := values[key]
		return value, ok
	})
}

func LoadFromEnvLookup(lookup func(string) (string, bool)) (Config, error) {
	cfg := Config{}

	if value, ok := lookup(envPrefix + "RUN"); ok {
		cfg.RunCommand = strings.TrimSpace(value)
	}
	if value, ok := lookup(envPrefix + "IMAGE"); ok {
		cfg.Image = strings.TrimSpace(value)
	}
	if value, ok := lookup(envPrefix + "PORTS"); ok {
		cfg.Ports = splitEnvList(value)
	}
	if value, ok := lookup(envPrefix + "CACHES"); ok {
		cfg.Cache = splitEnvList(value)
	}
	if value, ok := lookup(envPrefix + "MOUNTS"); ok {
		cfg.Mounts = splitEnvList(value)
	}
	if value, ok := lookup(envPrefix + "ENVS"); ok {
		cfg.EnvVars = splitEnvList(value)
	}
	if value, ok := lookup(envPrefix + "PODMAN_ARGS"); ok {
		cfg.PodmanArgs = splitEnvList(value)
	}
	if value, ok := lookup(envPrefix + "ALWAYS_ON_GROUPS"); ok {
		cfg.AlwaysOnGroups = splitEnvList(value)
	}

	return cfg, nil
}

func splitEnvList(value string) []string {
	parts := strings.Split(value, ",")
	values := make([]string, 0, len(parts))
	for _, part := range parts {
		trimmed := strings.TrimSpace(part)
		if trimmed == "" {
			continue
		}
		values = append(values, trimmed)
	}
	return values
}
