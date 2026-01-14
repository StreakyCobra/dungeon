package config

import (
	"fmt"
	"os"
	"path/filepath"
)

func LoadFromFile() (Config, error) {
	path, err := configPath()
	if err != nil {
		return Config{}, err
	}

	data, err := os.ReadFile(path)
	if err != nil {
		if os.IsNotExist(err) {
			return Config{}, nil
		}
		return Config{}, fmt.Errorf("read config %s: %w", path, err)
	}

	cfg, err := parseConfig(data)
	if err != nil {
		return Config{}, fmt.Errorf("parse config %s: %w", path, err)
	}

	return cfg, nil
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
