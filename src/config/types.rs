use std::collections::BTreeMap;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Settings {
    pub run_command: Option<String>,
    pub image: Option<String>,
    pub ports: Option<Vec<String>>,
    pub cache: Option<Vec<String>>,
    pub mounts: Option<Vec<String>>,
    pub env_vars: Option<Vec<String>>,
    pub env_files: Option<Vec<String>>,
    pub podman_args: Option<Vec<String>>,
}

#[derive(Debug, Clone, Default)]
pub struct Config {
    pub settings: Settings,
    pub always_on_groups: Option<Vec<String>>,
    pub groups: BTreeMap<String, GroupConfig>,
}

#[derive(Debug, Clone, Default)]
pub struct GroupConfig {
    pub settings: Settings,
    pub disabled: bool,
}

pub struct Sources {
    pub defaults: Settings,
    pub file: Settings,
    pub env: Settings,
    pub cli: Settings,
}

impl Settings {
    pub fn from_cli(matches: &clap::ArgMatches) -> Settings {
        let mut settings = Settings::default();

        if let Some(value) = matches.get_one::<String>("run") {
            settings.run_command = Some(value.to_string());
        }
        if let Some(value) = matches.get_one::<String>("image") {
            settings.image = Some(value.to_string());
        }
        if let Some(values) = matches.get_many::<String>("port") {
            settings.ports = Some(values.map(|s| s.to_string()).collect());
        }
        if let Some(values) = matches.get_many::<String>("cache") {
            settings.cache = Some(values.map(|s| s.to_string()).collect());
        }
        if let Some(values) = matches.get_many::<String>("mount") {
            settings.mounts = Some(values.map(|s| s.to_string()).collect());
        }
        if let Some(values) = matches.get_many::<String>("env") {
            settings.env_vars = Some(values.map(|s| s.to_string()).collect());
        }
        if let Some(values) = matches.get_many::<String>("env-file") {
            settings.env_files = Some(values.map(|s| s.to_string()).collect());
        }
        if let Some(values) = matches.get_many::<String>("podman-arg") {
            settings.podman_args = Some(values.map(|s| s.to_string()).collect());
        }

        settings
    }

    pub fn always_on_groups_from_cli(
        matches: &clap::ArgMatches,
        group_names: &[String],
    ) -> Option<Vec<String>> {
        let mut groups = Vec::new();
        for name in group_names {
            if matches.get_flag(name) {
                groups.push(name.clone());
            }
        }

        if groups.is_empty() {
            None
        } else {
            Some(groups)
        }
    }
}
