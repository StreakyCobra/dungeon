package main

import (
	"flag"
	"fmt"
	"os"
	"strconv"
	"strings"
)

func parseArgs(args []string) (options, []string, error) {
	defaults, err := loadDefaults()
	if err != nil {
		return options{}, nil, err
	}

	opts := defaults
	fs := flag.NewFlagSet("dungeon", flag.ContinueOnError)
	fs.SetOutput(os.Stderr)
	fs.Usage = func() {
		printUsage(fs)
	}

	fs.StringVar(&opts.runCommand, "run", defaults.runCommand, "run a command inside the container")
	fs.BoolVar(&opts.resetCache, "reset-cache", false, "delete the dungeon-cache volume before running")
	fs.BoolVar(&opts.showVersion, "version", false, "print version and exit")
	fs.StringVar(&opts.name, "name", defaults.name, "assign a name to the container")
	fs.StringVar(&opts.network, "network", defaults.network, "set the network mode")
	rmFlag := &boolFlag{value: defaults.remove}
	fs.Var(rmFlag, "rm", "remove the container after exit")
	portsFlag := &stringSliceFlag{values: append([]string{}, defaults.ports...)}
	fs.Var(portsFlag, "port", "publish a container port (repeatable)")

	groupFlags := make(map[string]*bool, len(defaults.groupSpecs))
	groupNames := sortedGroupNames(defaults.groupSpecs)
	for _, name := range groupNames {
		value := defaults.groupOn[name]
		ptr := new(bool)
		*ptr = value
		fs.BoolVar(ptr, name, value, fmt.Sprintf("enable group %q", name))
		groupFlags[name] = ptr
	}

	if err := fs.Parse(args); err != nil {
		return options{}, nil, err
	}

	opts.ports = portsFlag.values
	opts.remove = rmFlag.value
	if opts.name != "" && !rmFlag.set && !opts.removeSet {
		opts.remove = false
	}

	opts.groupSpecs = defaults.groupSpecs
	opts.groupOn = make(map[string]bool, len(groupFlags))
	for name, ptr := range groupFlags {
		opts.groupOn[name] = *ptr
	}

	return opts, fs.Args(), nil
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

type stringSliceFlag struct {
	values []string
}

func (s *stringSliceFlag) String() string {
	return strings.Join(s.values, ",")
}

func (s *stringSliceFlag) Set(value string) error {
	s.values = append(s.values, value)
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
