package main

import (
	"fmt"
	"os"
	"path/filepath"
	"strings"
)

func parseHostMountSpec(spec string) (string, string, string, error) {
	parts := strings.Split(spec, ":")
	if len(parts) < 2 || len(parts) > 3 {
		return "", "", "", fmt.Errorf("ERROR: invalid mount spec %q (expected source:target[:ro|rw])", spec)
	}

	source := strings.TrimSpace(parts[0])
	target := strings.TrimSpace(parts[1])
	if source == "" || target == "" {
		return "", "", "", fmt.Errorf("ERROR: invalid mount spec %q (source and target required)", spec)
	}

	var mode string
	if len(parts) == 3 {
		parsed, err := mountMode(parts[2])
		if err != nil {
			return "", "", "", err
		}
		mode = parsed
	}

	return source, target, mode, nil
}

func parseCacheMountSpec(spec string) (string, string, error) {
	parts := strings.Split(spec, ":")
	if len(parts) < 1 || len(parts) > 2 {
		return "", "", fmt.Errorf("ERROR: invalid cache mount spec %q (expected target[:ro|rw])", spec)
	}

	target := strings.TrimSpace(parts[0])
	if target == "" {
		return "", "", fmt.Errorf("ERROR: invalid cache mount spec %q (target required)", spec)
	}

	var mode string
	if len(parts) == 2 {
		parsed, err := mountMode(parts[1])
		if err != nil {
			return "", "", err
		}
		mode = parsed
	}

	return containerPath(target), mode, nil
}

func resolveHostPath(home, path string) (string, error) {
	trimmed := strings.TrimSpace(path)
	if trimmed == "" {
		return "", fmt.Errorf("ERROR: mount source cannot be empty")
	}
	if trimmed == "~" {
		return home, nil
	}
	if strings.HasPrefix(trimmed, "~/") {
		return filepath.Clean(filepath.Join(home, trimmed[2:])), nil
	}
	if filepath.IsAbs(trimmed) {
		return filepath.Clean(trimmed), nil
	}
	return filepath.Clean(filepath.Join(home, trimmed)), nil
}

func containerPath(path string) string {
	trimmed := strings.TrimSpace(path)
	if filepath.IsAbs(trimmed) {
		return filepath.Clean(trimmed)
	}
	return filepath.Clean(filepath.Join(userHome, trimmed))
}

func mountMode(mode string) (string, error) {
	trimmed := strings.TrimSpace(strings.ToLower(mode))
	if trimmed == "" || trimmed == "rw" {
		return "", nil
	}
	if trimmed == "ro" {
		return ":ro", nil
	}
	return "", fmt.Errorf("ERROR: invalid mount mode '%s' (use 'ro' or 'rw')", mode)
}

func buildEnvArgs(envSpecs []string) ([]string, error) {
	args := []string{}
	for _, spec := range envSpecs {
		trimmed := strings.TrimSpace(spec)
		if trimmed == "" {
			continue
		}
		if !strings.Contains(trimmed, "=") {
			value, ok := os.LookupEnv(trimmed)
			if !ok {
				return nil, fmt.Errorf("ERROR: env %q is not set on host", trimmed)
			}
			args = append(args, "--env", trimmed+"="+value)
			continue
		}
		name, value, ok := strings.Cut(trimmed, "=")
		if !ok || strings.TrimSpace(name) == "" {
			return nil, fmt.Errorf("ERROR: invalid env spec %q", spec)
		}
		args = append(args, "--env", name+"="+value)
	}
	return args, nil
}
