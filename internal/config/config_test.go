package config

import (
	"bufio"
	"bytes"
	"flag"
	"fmt"
	"io"
	"os"
	"path/filepath"
	"reflect"
	"runtime"
	"sort"
	"strconv"
	"strings"
	"testing"
)

type markdownTestCase struct {
	name     string
	config   Config
	env      map[string]string
	cliArgs  []string
	expected Settings
}

func TestMarkdownConfig(t *testing.T) {
	testsDir := markdownTestsDir(t)
	files, err := filepath.Glob(filepath.Join(testsDir, "*.md"))
	if err != nil {
		t.Fatalf("find markdown tests: %v", err)
	}
	if len(files) == 0 {
		t.Fatalf("no markdown tests found in %s", testsDir)
	}

	for _, path := range files {
		testCase, err := readMarkdownTest(path)
		if err != nil {
			t.Fatalf("read %s: %v", path, err)
		}
		name := testCase.name
		if name == "" {
			name = filepath.Base(path)
		}

		t.Run(name, func(t *testing.T) {
			settings, err := resolveSettingsFromTest(testCase)
			if err != nil {
				t.Fatalf("resolve settings: %v", err)
			}
			if !reflect.DeepEqual(settings, testCase.expected) {
				t.Fatalf("settings mismatch\nexpected: %#v\n     got: %#v", testCase.expected, settings)
			}
		})
	}
}

func markdownTestsDir(t *testing.T) string {
	_, filename, _, ok := runtime.Caller(0)
	if !ok {
		t.Fatal("unable to locate test file")
	}
	return filepath.Clean(filepath.Join(filepath.Dir(filename), "..", "..", "tests"))
}

func resolveSettingsFromTest(testCase markdownTestCase) (Settings, error) {
	defaultsConfig, err := LoadDefaults()
	if err != nil {
		return Settings{}, err
	}

	envConfig, err := LoadFromEnvMap(testCase.env)
	if err != nil {
		return Settings{}, err
	}

	groupDefs, err := MergeGroupDefinitions(defaultsConfig.Groups, testCase.config.Groups)
	if err != nil {
		return Settings{}, err
	}

	alwaysOnGroups := ResolveAlwaysOnGroups(defaultsConfig, testCase.config, envConfig)
	alwaysOnGroupOrder, err := NormalizeGroupOrder(alwaysOnGroups)
	if err != nil {
		return Settings{}, err
	}
	groupEnabled, err := BuildGroupSelection(groupDefs, alwaysOnGroupOrder)
	if err != nil {
		return Settings{}, err
	}

	cliSettings, groupFlags, err := parseCLISettings(testCase.cliArgs, sortedGroupNames(groupDefs), groupEnabled)
	if err != nil {
		return Settings{}, err
	}
	groupOrder := resolveGroupOrder(alwaysOnGroupOrder, groupFlags)

	settings, err := ResolveSettings(Sources{
		Defaults: defaultsConfig,
		File:     testCase.config,
		Env:      envConfig,
		CLI:      cliSettings,
	}, groupDefs, groupOrder)
	if err != nil {
		return Settings{}, err
	}

	return settings, nil
}

func readMarkdownTest(path string) (markdownTestCase, error) {
	data, err := os.ReadFile(path)
	if err != nil {
		return markdownTestCase{}, err
	}

	var title string
	sections := map[string]*strings.Builder{
		"Config":   {},
		"Env":      {},
		"CLI":      {},
		"Expected": {},
	}
	currentSection := ""
	inCodeBlock := false

	scanner := bufio.NewScanner(bytes.NewReader(data))
	scanner.Buffer(make([]byte, 0, 64*1024), 1024*1024)
	for scanner.Scan() {
		line := scanner.Text()
		trimmed := strings.TrimSpace(line)
		if strings.HasPrefix(line, "# ") && title == "" {
			title = strings.TrimSpace(line[2:])
			continue
		}
		if strings.HasPrefix(line, "## ") {
			currentSection = strings.TrimSpace(line[3:])
			inCodeBlock = false
			continue
		}
		if strings.HasPrefix(trimmed, "```") {
			if currentSection != "" {
				inCodeBlock = !inCodeBlock
			}
			continue
		}
		if inCodeBlock {
			if builder, ok := sections[currentSection]; ok {
				if builder.Len() > 0 {
					builder.WriteByte('\n')
				}
				builder.WriteString(line)
			}
		}
	}
	if err := scanner.Err(); err != nil {
		return markdownTestCase{}, err
	}

	configText := strings.TrimSpace(sections["Config"].String())
	envText := strings.TrimSpace(sections["Env"].String())
	cliText := strings.TrimSpace(sections["CLI"].String())
	expectedText := strings.TrimSpace(sections["Expected"].String())
	if expectedText == "" {
		return markdownTestCase{}, fmt.Errorf("expected section is required")
	}

	config, err := parseConfigText(configText)
	if err != nil {
		return markdownTestCase{}, fmt.Errorf("parse Config: %w", err)
	}
	values, err := parseEnvSection(envText)
	if err != nil {
		return markdownTestCase{}, fmt.Errorf("parse Env: %w", err)
	}
	cliArgs, err := parseCLISection(cliText)
	if err != nil {
		return markdownTestCase{}, fmt.Errorf("parse CLI: %w", err)
	}
	expectedConfig, err := parseConfigText(expectedText)
	if err != nil {
		return markdownTestCase{}, fmt.Errorf("parse Expected: %w", err)
	}

	return markdownTestCase{
		name:     title,
		config:   config,
		env:      values,
		cliArgs:  cliArgs,
		expected: expectedConfig.Settings,
	}, nil
}

func parseConfigText(content string) (Config, error) {
	if strings.TrimSpace(content) == "" {
		return Config{}, nil
	}
	return parseConfig([]byte(content))
}

func parseEnvSection(content string) (map[string]string, error) {
	values := map[string]string{}
	for _, line := range strings.Split(content, "\n") {
		trimmed := strings.TrimSpace(line)
		if trimmed == "" {
			continue
		}
		parts := strings.SplitN(trimmed, "=", 2)
		if len(parts) != 2 {
			return nil, fmt.Errorf("invalid env line %q", trimmed)
		}
		values[parts[0]] = parts[1]
	}
	return values, nil
}

func parseCLISection(content string) ([]string, error) {
	if strings.TrimSpace(content) == "" {
		return nil, nil
	}
	fields := strings.Fields(content)
	if len(fields) == 0 {
		return nil, nil
	}
	if fields[0] != "dungeon" {
		return nil, fmt.Errorf("CLI section must start with dungeon")
	}
	return fields[1:], nil
}

func parseCLISettings(args []string, groupNames []string, groupEnabled map[string]bool) (Settings, map[string]*boolFlag, error) {
	fs := flag.NewFlagSet("dungeon", flag.ContinueOnError)
	fs.SetOutput(io.Discard)

	runFlag := &stringFlag{}
	fs.Var(runFlag, "run", "")

	imageFlag := &stringFlag{}
	fs.Var(imageFlag, "image", "")

	portsFlag := &stringSliceFlag{}
	fs.Var(portsFlag, "port", "")

	cacheFlag := &stringSliceFlag{}
	fs.Var(cacheFlag, "cache", "")

	mountsFlag := &stringSliceFlag{}
	fs.Var(mountsFlag, "mount", "")

	envVarFlag := &stringSliceFlag{}
	fs.Var(envVarFlag, "env", "")

	podmanArgsFlag := &stringSliceFlag{}
	fs.Var(podmanArgsFlag, "podman-arg", "")

	groupFlags := make(map[string]*boolFlag, len(groupNames))
	var groupOrderCounter int
	for _, name := range groupNames {
		value := groupEnabled[name]
		flagValue := &boolFlag{value: value, counter: &groupOrderCounter}
		fs.Var(flagValue, name, "")
		groupFlags[name] = flagValue
	}

	if err := fs.Parse(args); err != nil {
		return Settings{}, nil, err
	}

	settings := Settings{}
	if runFlag.set {
		settings.RunCommand = runFlag.value
	}
	if imageFlag.set {
		settings.Image = imageFlag.value
	}
	if portsFlag.set {
		settings.Ports = append([]string{}, portsFlag.values...)
	}
	if cacheFlag.set {
		settings.Cache = append([]string{}, cacheFlag.values...)
	}
	if mountsFlag.set {
		settings.Mounts = append([]string{}, mountsFlag.values...)
	}
	if envVarFlag.set {
		settings.EnvVars = append([]string{}, envVarFlag.values...)
	}
	if podmanArgsFlag.set {
		settings.PodmanArgs = append([]string{}, podmanArgsFlag.values...)
	}

	return settings, groupFlags, nil
}

func resolveGroupOrder(defaultGroups []string, groupFlags map[string]*boolFlag) []string {
	order := append([]string{}, defaultGroups...)
	if len(groupFlags) == 0 {
		return order
	}

	type selection struct {
		name  string
		order int
	}
	selected := []selection{}
	selectedSet := map[string]struct{}{}
	for name, flagValue := range groupFlags {
		if flagValue.set && flagValue.value {
			selected = append(selected, selection{name: name, order: flagValue.order})
			selectedSet[name] = struct{}{}
		}
	}
	if len(selected) == 0 {
		return order
	}

	sort.Slice(selected, func(i, j int) bool {
		return selected[i].order < selected[j].order
	})
	filteredDefaults := make([]string, 0, len(order))
	for _, name := range order {
		if _, ok := selectedSet[name]; ok {
			continue
		}
		filteredDefaults = append(filteredDefaults, name)
	}
	order = filteredDefaults
	for _, item := range selected {
		order = append(order, item.name)
	}
	return order
}

func sortedGroupNames(values map[string]GroupConfig) []string {
	keys := make([]string, 0, len(values))
	for key := range values {
		keys = append(keys, key)
	}
	sort.Strings(keys)
	return keys
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
