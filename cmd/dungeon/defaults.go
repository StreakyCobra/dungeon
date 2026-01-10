package main

import _ "embed"

//go:embed defaults.toml
var defaultConfigData []byte

func loadDefaultConfig() (config, error) {
	if len(defaultConfigData) == 0 {
		return config{}, nil
	}
	return parseConfig(defaultConfigData)
}
