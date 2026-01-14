package config

import (
	"fmt"
	"os"
	"strconv"
	"strings"
)

const envPrefix = "DUNGEON_"

func LoadFromEnv() (Config, error) {
	cfg := Config{}

	if value, ok := os.LookupEnv(envPrefix + "RUN"); ok {
		cfg.RunCommand = strings.TrimSpace(value)
	}
	if value, ok := os.LookupEnv(envPrefix + "IMAGE"); ok {
		cfg.Image = strings.TrimSpace(value)
	}
	if value, ok := os.LookupEnv(envPrefix + "PORTS"); ok {
		cfg.Ports = splitEnvList(value)
	}
	if value, ok := os.LookupEnv(envPrefix + "CACHE"); ok {
		cfg.Cache = splitEnvList(value)
	}
	if value, ok := os.LookupEnv(envPrefix + "PODMAN_ARGS"); ok {
		cfg.PodmanArgs = splitEnvList(value)
	}
	if value, ok := os.LookupEnv(envPrefix + "DEFAULT_GROUPS"); ok {
		cfg.DefaultGroups = splitEnvList(value)
	}
	if value, ok := os.LookupEnv(envPrefix + "PERSIST"); ok {
		parsed, err := strconv.ParseBool(strings.TrimSpace(value))
		if err != nil {
			return Config{}, fmt.Errorf(envPrefix+"PERSIST must be a boolean: %w", err)
		}
		cfg.Persist = &parsed
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
