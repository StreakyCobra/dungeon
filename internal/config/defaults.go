package config

import _ "embed"

//go:embed defaults.toml
var defaultConfigData []byte

func LoadDefaults() (Config, error) {
	if len(defaultConfigData) == 0 {
		return Config{}, nil
	}
	return parseConfig(defaultConfigData)
}
