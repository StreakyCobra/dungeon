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
	resetCache    bool
	image         string
	ports         []string
	cache         []string
	mounts        []string
	envVars       []string
	showVersion   bool
	podmanArgs    []string
	persistMode   persistMode
	containerName string
	keepContainer bool
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

	var runErr error
	switch opts.persistMode {
	case persistDiscard:
		runErr = discardContainer(opts.containerName)
	case persistReuse:
		exists, err := podmanContainerExists(opts.containerName)
		if err != nil {
			fmt.Fprintln(os.Stderr, err)
			os.Exit(1)
		}
		if !exists {
			fmt.Fprintf(os.Stderr, "ERROR: container %q does not exist\n", opts.containerName)
			os.Exit(1)
		}
		runErr = ensureContainerSession(opts.containerName)
	case persistCreate:
		runErr = runPersistedSession(opts, paths)
	default:
		cmd, err := buildPodmanCommand(opts, paths)
		if err != nil {
			fmt.Fprintln(os.Stderr, err)
			os.Exit(1)
		}
		runErr = runPodmanCommand(cmd)
	}

	if runErr != nil {
		if exitErr := new(exec.ExitError); errors.As(runErr, &exitErr) {
			os.Exit(exitErr.ExitCode())
		}
		fmt.Fprintln(os.Stderr, runErr)
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
