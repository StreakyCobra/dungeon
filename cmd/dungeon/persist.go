package main

import (
	"crypto/sha256"
	"encoding/hex"
	"errors"
	"fmt"
	"os"
	"os/exec"
	"path/filepath"
	"strings"
	"unicode"
)

type persistMode int

const (
	persistNone persistMode = iota
	persistCreate
	persistReuse
	persistDiscard
)

func persistedContainerName(paths []string) (string, error) {
	cwd, err := os.Getwd()
	if err != nil {
		return "", err
	}
	absCwd, err := filepath.Abs(cwd)
	if err != nil {
		return "", err
	}

	hashInputs := []string{absCwd}
	for _, path := range paths {
		absPath, err := filepath.Abs(path)
		if err != nil {
			return "", err
		}
		hashInputs = append(hashInputs, absPath)
	}

	sum := sha256.Sum256([]byte(strings.Join(hashInputs, "\n")))
	shortHash := hex.EncodeToString(sum[:])[:8]

	base := sanitizeContainerBase(filepath.Base(absCwd))
	return fmt.Sprintf("dungeon-%s-%s", base, shortHash), nil
}

func sanitizeContainerBase(name string) string {
	var builder strings.Builder
	for _, char := range name {
		if unicode.IsLetter(char) || unicode.IsNumber(char) || char == '-' || char == '_' || char == '.' {
			builder.WriteRune(char)
			continue
		}
		builder.WriteRune('-')
	}
	cleaned := strings.Trim(builder.String(), "-")
	if cleaned == "" {
		return "project"
	}
	return cleaned
}

func podmanContainerExists(name string) (bool, error) {
	cmd := exec.Command("podman", "container", "exists", name)
	if err := cmd.Run(); err != nil {
		if exitErr := new(exec.ExitError); errors.As(err, &exitErr) {
			if exitErr.ExitCode() == 1 {
				return false, nil
			}
		}
		return false, err
	}
	return true, nil
}

func podmanContainerRunning(name string) (bool, error) {
	cmd := exec.Command("podman", "inspect", "-f", "{{.State.Running}}", name)
	output, err := cmd.Output()
	if err != nil {
		return false, err
	}
	return strings.TrimSpace(string(output)) == "true", nil
}

func startContainer(name string) error {
	cmd := exec.Command("podman", "start", name)
	return runPodmanCommand(cmd)
}

func execIntoContainer(name string) error {
	cmd := exec.Command("podman", "exec", "-it", name, "bash")
	return runPodmanCommand(cmd)
}

func ensureContainerSession(name string) error {
	running, err := podmanContainerRunning(name)
	if err != nil {
		return err
	}
	if !running {
		if err := startContainer(name); err != nil {
			return err
		}
	}
	return execIntoContainer(name)
}

func discardContainer(name string) error {
	cmd := exec.Command("podman", "rm", "-f", name)
	return runPodmanCommand(cmd)
}

func runPersistedSession(opts options, paths []string) error {
	exists, err := podmanContainerExists(opts.containerName)
	if err != nil {
		return err
	}
	if exists {
		return ensureContainerSession(opts.containerName)
	}
	cmd, err := buildPodmanCommand(opts, paths)
	if err != nil {
		return err
	}
	return runPodmanCommand(cmd)
}

func runPodmanCommand(cmd *exec.Cmd) error {
	cmd.Stdin = os.Stdin
	cmd.Stdout = os.Stdout
	cmd.Stderr = os.Stderr
	return cmd.Run()
}
