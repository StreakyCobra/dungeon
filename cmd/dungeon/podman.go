package main

import (
	"fmt"
	"os"
	"os/exec"
	"path/filepath"
	"strings"
)

func buildPodmanCommand(opts options, paths []string) (*exec.Cmd, error) {
	cwd, err := os.Getwd()
	if err != nil {
		return nil, err
	}

	home, err := os.UserHomeDir()
	if err != nil {
		return nil, err
	}

	mounts := []string{
		"-v", "dungeon-cache:" + userHome + "/.cache",
		"-v", "dungeon-cache:" + userHome + "/.npm",
	}

	envSpecs := []string{}
	groupNames := sortedGroupNames(opts.groupSpecs)
	for _, name := range groupNames {
		if !opts.groupOn[name] {
			continue
		}
		group := opts.groupSpecs[name]
		for _, target := range group.Cache {
			cacheTarget, mode, err := parseCacheMountSpec(target)
			if err != nil {
				return nil, err
			}
			mounts = append(mounts, "-v", "dungeon-cache:"+cacheTarget+mode)
		}
		for _, spec := range group.Mounts {
			source, target, mode, err := parseHostMountSpec(spec)
			if err != nil {
				return nil, err
			}
			hostPath, err := resolveHostPath(home, source)
			if err != nil {
				return nil, err
			}
			if _, err := os.Stat(hostPath); err != nil {
				return nil, fmt.Errorf("ERROR: mount source '%s' does not exist", hostPath)
			}
			targetPath := containerPath(target)
			mounts = append(mounts, "-v", hostPath+":"+targetPath+mode)
		}
		envSpecs = append(envSpecs, group.Env...)
	}

	envVars, err := buildEnvArgs(envSpecs)
	if err != nil {
		return nil, err
	}

	var workdir string
	if len(paths) == 0 {
		if sameDir(cwd, home) {
			return nil, fmt.Errorf("ERROR: refusing to run from home directory")
		}
		base := filepath.Base(cwd)
		workdir = userHome + "/" + base
		mounts = append(mounts, "-v", cwd+":"+workdir)
	} else {
		workdir = userHome + "/project"
		for _, path := range paths {
			if _, err := os.Stat(path); err != nil {
				return nil, fmt.Errorf("ERROR: '%s' does not exist", path)
			}
			abs, err := filepath.Abs(path)
			if err != nil {
				return nil, err
			}
			base := filepath.Base(path)
			mounts = append(mounts, "-v", abs+":"+workdir+"/"+base)
		}
	}

	args := []string{"run", "-it", "--userns=keep-id", "-w", workdir}
	if opts.remove {
		args = append(args, "--rm")
	}
	if opts.name != "" {
		args = append(args, "--name", opts.name)
	}
	if opts.network != "" {
		args = append(args, "--network", opts.network)
	}
	args = append(args, envVars...)
	for _, port := range opts.ports {
		trimmed := strings.TrimSpace(port)
		if trimmed == "" {
			continue
		}
		args = append(args, "-p", trimmed)
	}
	args = append(args, opts.podmanArgs...)

	runCommand := strings.TrimSpace(opts.runCommand)

	args = append(args, mounts...)
	image := defaultImage
	if opts.image != "" {
		image = opts.image
	}
	args = append(args, image)

	if runCommand == "" {
		args = append(args, "bash")
	} else {
		args = append(args, "bash", "-lc", runCommand)
	}

	return exec.Command("podman", args...), nil
}
