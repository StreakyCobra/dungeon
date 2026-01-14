package main

import (
	"flag"
	"fmt"
	"os"
	"strconv"
	"strings"

	"github.com/StreakyCobra/dungeon/internal/config"
)

func parseArgs(args []string) (options, []string, error) {
	defaultsConfig, err := config.LoadDefaults()
	if err != nil {
		return options{}, nil, err
	}
	fileConfig, err := config.LoadFromFile()
	if err != nil {
		return options{}, nil, err
	}
	envConfig, err := config.LoadFromEnv()
	if err != nil {
		return options{}, nil, err
	}

	baseConfig := config.Reduce(defaultsConfig, fileConfig, envConfig)
	baseOptions, err := optionsFromConfig(baseConfig)
	if err != nil {
		return options{}, nil, err
	}

	fs := flag.NewFlagSet("dungeon", flag.ContinueOnError)
	fs.SetOutput(os.Stderr)
	fs.Usage = func() {
		printUsage(fs)
	}

	runFlag := &stringFlag{value: baseOptions.runCommand}
	fs.Var(runFlag, "run", "run a command inside the container")

	var resetCache bool
	var showVersion bool
	fs.BoolVar(&resetCache, "reset-cache", false, "delete the dungeon-cache volume before running")
	fs.BoolVar(&showVersion, "version", false, "print version and exit")

	nameFlag := &stringFlag{value: baseOptions.name}
	fs.Var(nameFlag, "name", "assign a name to the container")

	networkFlag := &stringFlag{value: baseOptions.network}
	fs.Var(networkFlag, "network", "set the network mode")

	rmFlag := &boolFlag{value: baseOptions.remove}
	fs.Var(rmFlag, "rm", "remove the container after exit")

	portsFlag := &stringSliceFlag{values: append([]string{}, baseOptions.ports...)}
	fs.Var(portsFlag, "port", "publish a container port (repeatable)")

	groupFlags := make(map[string]*boolFlag, len(baseOptions.groupSpecs))
	groupNames := sortedGroupNames(baseOptions.groupSpecs)
	for _, name := range groupNames {
		value := baseOptions.groupOn[name]
		flagValue := &boolFlag{value: value}
		fs.Var(flagValue, name, fmt.Sprintf("enable group %q", name))
		groupFlags[name] = flagValue
	}

	if err := fs.Parse(args); err != nil {
		return options{}, nil, err
	}

	cliConfig := cliConfigFromFlags(runFlag, nameFlag, networkFlag, rmFlag, portsFlag, groupFlags, groupNames)
	finalConfig := config.Reduce(baseConfig, cliConfig)
	finalOptions, err := optionsFromConfig(finalConfig)
	if err != nil {
		return options{}, nil, err
	}

	finalOptions.resetCache = resetCache
	finalOptions.showVersion = showVersion

	return finalOptions, fs.Args(), nil
}

func cliConfigFromFlags(runFlag, nameFlag, networkFlag *stringFlag, rmFlag *boolFlag, portsFlag *stringSliceFlag, groupFlags map[string]*boolFlag, groupNames []string) config.Config {
	cfg := config.Config{}
	if runFlag.set {
		cfg.RunCommand = runFlag.value
	}
	if nameFlag.set {
		cfg.Name = nameFlag.value
	}
	if networkFlag.set {
		cfg.Network = networkFlag.value
	}
	if portsFlag.set {
		cfg.Ports = append([]string{}, portsFlag.values...)
	}
	if rmFlag.set {
		value := rmFlag.value
		cfg.Remove = &value
	}

	hasGroupOverride := false
	for _, name := range groupNames {
		if groupFlags[name].set {
			hasGroupOverride = true
			break
		}
	}
	if hasGroupOverride {
		enabled := make([]string, 0, len(groupNames))
		for _, name := range groupNames {
			if groupFlags[name].value {
				enabled = append(enabled, name)
			}
		}
		cfg.DefaultGroups = enabled
	}

	return cfg
}

func printUsage(fs *flag.FlagSet) {
	w := fs.Output()
	fmt.Fprintf(w, "Usage of %s:\n", fs.Name())
	fs.VisitAll(func(f *flag.Flag) {
		name := "--" + f.Name
		usage := f.Usage
		if isBoolFlag(f) {
			if f.DefValue == "true" {
				usage = usage + " (default true)"
			}
		} else if f.DefValue != "" {
			usage = usage + " (default " + f.DefValue + ")"
		}
		fmt.Fprintf(w, "  %s\n    \t%s\n", name, usage)
	})
}

func isBoolFlag(f *flag.Flag) bool {
	boolFlag, ok := f.Value.(interface{ IsBoolFlag() bool })
	return ok && boolFlag.IsBoolFlag()
}

type stringFlag struct {
	value string
	set   bool
}

type stringSliceFlag struct {
	values []string
	set    bool
}

func (s *stringFlag) String() string {
	return s.value
}

func (s *stringFlag) Set(value string) error {
	s.value = value
	s.set = true
	return nil
}

func (s *stringSliceFlag) String() string {
	return strings.Join(s.values, ",")
}

func (s *stringSliceFlag) Set(value string) error {
	s.values = append(s.values, value)
	s.set = true
	return nil
}

type boolFlag struct {
	value bool
	set   bool
}

func (b *boolFlag) String() string {
	return strconv.FormatBool(b.value)
}

func (b *boolFlag) Set(value string) error {
	parsed, err := strconv.ParseBool(value)
	if err != nil {
		return err
	}
	b.value = parsed
	b.set = true
	return nil
}

func (b *boolFlag) IsBoolFlag() bool {
	return true
}
