package main

import (
	"flag"
	"fmt"
	"os"
	"sort"
	"strconv"
	"strings"

	"github.com/StreakyCobra/dungeon/internal/config"
)

type usageSections struct {
	options map[string]struct{}
	config  map[string]struct{}
	groups  map[string]struct{}
}

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

	groupDefs, err := config.MergeGroupDefinitions(defaultsConfig.Groups, fileConfig.Groups)
	if err != nil {
		return options{}, nil, err
	}

	defaultGroups := config.ResolveDefaultGroups(defaultsConfig, fileConfig, envConfig)
	defaultGroupOrder, err := config.NormalizeGroupOrder(defaultGroups)
	if err != nil {
		return options{}, nil, err
	}
	groupEnabled, err := config.BuildGroupSelection(groupDefs, defaultGroupOrder)
	if err != nil {
		return options{}, nil, err
	}

	baseSettings, err := config.ResolveSettings(config.Sources{Defaults: defaultsConfig, File: fileConfig, Env: envConfig}, groupDefs, defaultGroupOrder)
	if err != nil {
		return options{}, nil, err
	}
	baseOptions := optionsFromSettings(baseSettings)

	groupNames := sortedGroupNames(groupDefs)

	fs := flag.NewFlagSet("dungeon", flag.ContinueOnError)
	fs.SetOutput(os.Stderr)

	sections := usageSections{
		options: map[string]struct{}{
			"help":       {},
			"reset-cache": {},
			"version":    {},
			"persist":    {},
			"persisted":  {},
			"discard":    {},
		},
		config: map[string]struct{}{
			"run":        {},
			"image":      {},
			"port":       {},
			"cache":      {},
			"mount":      {},
			"env":        {},
			"podman-arg": {},
		},
		groups: map[string]struct{}{},
	}
	for _, name := range groupNames {
		sections.groups[name] = struct{}{}
	}

	fs.Usage = func() {
		printUsage(fs, sections)
	}

	var showHelp bool
	var resetCache bool
	var showVersion bool
	var persistContainer bool
	var persistedContainer bool
	var discardContainer bool
	fs.BoolVar(&showHelp, "help", false, "show help and exit")
	fs.BoolVar(&resetCache, "reset-cache", false, "delete the dungeon-cache volume before running")
	fs.BoolVar(&showVersion, "version", false, "print version and exit")
	fs.BoolVar(&persistContainer, "persist", false, "create or reuse a persisted container")
	fs.BoolVar(&persistedContainer, "persisted", false, "reuse the persisted container")
	fs.BoolVar(&discardContainer, "discard", false, "remove the persisted container")

	runFlag := &stringFlag{value: baseOptions.runCommand}
	fs.Var(runFlag, "run", "run a command inside the container")

	imageFlag := &stringFlag{value: baseOptions.image}
	fs.Var(imageFlag, "image", "container image to run")

	portsFlag := &stringSliceFlag{values: append([]string{}, baseOptions.ports...)}
	fs.Var(portsFlag, "port", "publish a container port (repeatable)")

	cacheFlag := &stringSliceFlag{values: append([]string{}, baseOptions.cache...)}
	fs.Var(cacheFlag, "cache", "mount cache volume targets (repeatable)")

	mountsFlag := &stringSliceFlag{values: append([]string{}, baseOptions.mounts...)}
	fs.Var(mountsFlag, "mount", "bind-mount a host path (repeatable)")

	envVarFlag := &stringSliceFlag{values: append([]string{}, baseOptions.envVars...)}
	fs.Var(envVarFlag, "env", "add container env vars (repeatable)")

	podmanArgsFlag := &stringSliceFlag{values: append([]string{}, baseOptions.podmanArgs...)}
	fs.Var(podmanArgsFlag, "podman-arg", "append a podman run arg (repeatable)")

	groupFlags := make(map[string]*boolFlag, len(groupDefs))
	var groupOrderCounter int
	for _, name := range groupNames {
		value := groupEnabled[name]
		flagValue := &boolFlag{value: value, counter: &groupOrderCounter}
		fs.Var(flagValue, name, fmt.Sprintf("enable group %q", name))
		groupFlags[name] = flagValue
	}

	if err := fs.Parse(args); err != nil {
		return options{}, nil, err
	}

	if showHelp {
		fs.Usage()
		return options{}, nil, flag.ErrHelp
	}

	persistMode, err := resolvePersistMode(persistContainer, persistedContainer, discardContainer)
	if err != nil {
		return options{}, nil, err
	}

	configOverrides := runFlag.set || imageFlag.set || portsFlag.set || cacheFlag.set || mountsFlag.set || envVarFlag.set || podmanArgsFlag.set
	groupOverrides := hasGroupOverrides(groupFlags)
	paths := fs.Args()

	if persistMode == persistReuse || persistMode == persistDiscard {
		if configOverrides || groupOverrides || len(paths) > 0 {
			return options{}, nil, fmt.Errorf("ERROR: --persisted and --discard do not accept config, group, or path arguments")
		}
		name, err := persistedContainerName(nil)
		if err != nil {
			return options{}, nil, err
		}
		return options{resetCache: resetCache, showVersion: showVersion, persistMode: persistMode, containerName: name}, nil, nil
	}

	cliSettings := cliSettingsFromFlags(runFlag, imageFlag, portsFlag, cacheFlag, mountsFlag, envVarFlag, podmanArgsFlag)
	groupOrder := resolveGroupOrder(defaultGroupOrder, groupFlags)

	finalSettings, err := config.ResolveSettings(config.Sources{Defaults: defaultsConfig, File: fileConfig, Env: envConfig, CLI: cliSettings}, groupDefs, groupOrder)
	if err != nil {
		return options{}, nil, err
	}
	finalOptions := optionsFromSettings(finalSettings)
	finalOptions.resetCache = resetCache
	finalOptions.showVersion = showVersion
	finalOptions.persistMode = persistMode
	if persistMode == persistCreate {
		name, err := persistedContainerName(paths)
		if err != nil {
			return options{}, nil, err
		}
		finalOptions.containerName = name
		finalOptions.keepContainer = true
	}

	return finalOptions, paths, nil
}

func cliSettingsFromFlags(runFlag *stringFlag, imageFlag *stringFlag, portsFlag *stringSliceFlag, cacheFlag *stringSliceFlag, mountsFlag *stringSliceFlag, envVarFlag *stringSliceFlag, podmanArgsFlag *stringSliceFlag) config.Settings {
	cfg := config.Settings{}
	if runFlag.set {
		cfg.RunCommand = runFlag.value
	}
	if imageFlag.set {
		cfg.Image = imageFlag.value
	}
	if portsFlag.set {
		cfg.Ports = append([]string{}, portsFlag.values...)
	}
	if cacheFlag.set {
		cfg.Cache = append([]string{}, cacheFlag.values...)
	}
	if mountsFlag.set {
		cfg.Mounts = append([]string{}, mountsFlag.values...)
	}
	if envVarFlag.set {
		cfg.EnvVars = append([]string{}, envVarFlag.values...)
	}
	if podmanArgsFlag.set {
		cfg.PodmanArgs = append([]string{}, podmanArgsFlag.values...)
	}

	return cfg
}

func resolveGroupOrder(defaultGroups []string, groupFlags map[string]*boolFlag) []string {
	if len(groupFlags) == 0 {
		return append([]string{}, defaultGroups...)
	}

	var hasOverride bool
	type selection struct {
		name  string
		order int
	}
	selected := []selection{}
	for name, flagValue := range groupFlags {
		if flagValue.set {
			hasOverride = true
		}
		if flagValue.set && flagValue.value {
			selected = append(selected, selection{name: name, order: flagValue.order})
		}
	}
	if !hasOverride {
		return append([]string{}, defaultGroups...)
	}

	sort.Slice(selected, func(i, j int) bool {
		return selected[i].order < selected[j].order
	})
	order := make([]string, 0, len(selected))
	for _, item := range selected {
		order = append(order, item.name)
	}
	return order
}

func resolvePersistMode(persistContainer bool, persistedContainer bool, discardContainer bool) (persistMode, error) {
	total := 0
	if persistContainer {
		total++
	}
	if persistedContainer {
		total++
	}
	if discardContainer {
		total++
	}
	if total > 1 {
		return persistNone, fmt.Errorf("ERROR: --persist, --persisted, and --discard are mutually exclusive")
	}
	if discardContainer {
		return persistDiscard, nil
	}
	if persistedContainer {
		return persistReuse, nil
	}
	if persistContainer {
		return persistCreate, nil
	}
	return persistNone, nil
}

func hasGroupOverrides(groupFlags map[string]*boolFlag) bool {
	for _, flagValue := range groupFlags {
		if flagValue.set {
			return true
		}
	}
	return false
}

func printUsage(fs *flag.FlagSet, sections usageSections) {
	w := fs.Output()
	fmt.Fprintf(w, "Usage of %s:\n\n", fs.Name())

	formatFlag := func(f *flag.Flag) {
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
	}

	printSection := func(title string, names map[string]struct{}, printed *int) {
		if len(names) == 0 {
			return
		}
		if *printed > 0 {
			fmt.Fprintln(w)
		}
		fmt.Fprintln(w, title+":")
		fs.VisitAll(func(f *flag.Flag) {
			if _, ok := names[f.Name]; !ok {
				return
			}
			formatFlag(f)
		})
		*printed = *printed + 1
	}

	printed := 0
	printSection("Options", sections.options, &printed)
	printSection("Configuration", sections.config, &printed)
	printSection("Groups", sections.groups, &printed)
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
	value   bool
	set     bool
	order   int
	counter *int
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
	if b.counter != nil {
		*b.counter = *b.counter + 1
		b.order = *b.counter
	}
	return nil
}

func (b *boolFlag) IsBoolFlag() bool {
	return true
}
