package main

import (
	"errors"
	"flag"
	"fmt"
	"os"
	"os/exec"
	"runtime/debug"
)

const (
	defaultImage = "localhost/dungeon"
	userHome     = "/home/dungeon"
)

var version = "dev"

type options struct {
	runCommand  string
	resetCache  bool
	image       string
	ports       []string
	cache       []string
	mounts      []string
	envVars     []string
	persist     bool
	showVersion bool
	podmanArgs  []string
}

func main() {
	opts, paths, err := parseArgs(os.Args[1:])
	if err != nil {
		if errors.Is(err, flag.ErrHelp) {
			return
		}
		fmt.Fprintln(os.Stderr, err)
		os.Exit(1)
	}

	if opts.showVersion {
		fmt.Fprintln(os.Stdout, buildVersion())
		return
	}

	if opts.resetCache {
		if err := resetCacheVolume(); err != nil {
			fmt.Fprintln(os.Stderr, err)
			os.Exit(1)
		}
	}

	cmd, err := buildPodmanCommand(opts, paths)
	if err != nil {
		fmt.Fprintln(os.Stderr, err)
		os.Exit(1)
	}

	cmd.Stdin = os.Stdin
	cmd.Stdout = os.Stdout
	cmd.Stderr = os.Stderr

	if err := cmd.Run(); err != nil {
		if exitErr := new(exec.ExitError); errors.As(err, &exitErr) {
			os.Exit(exitErr.ExitCode())
		}
		fmt.Fprintln(os.Stderr, err)
		os.Exit(1)
	}
}

func resetCacheVolume() error {
	cmd := exec.Command("podman", "volume", "rm", "-f", "dungeon-cache")
	cmd.Stdin = os.Stdin
	cmd.Stdout = os.Stdout
	cmd.Stderr = os.Stderr

	if err := cmd.Run(); err != nil {
		if exitErr := new(exec.ExitError); errors.As(err, &exitErr) {
			return fmt.Errorf("podman volume rm: %w", exitErr)
		}
		return err
	}
	return nil
}

func buildVersion() string {
	if version != "" && version != "dev" {
		return version
	}
	if info, ok := debug.ReadBuildInfo(); ok {
		if info.Main.Version != "" && info.Main.Version != "(devel)" {
			return info.Main.Version
		}
	}
	return "dev"
}
