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

pub struct ResolvedConfig {
    pub settings: Settings,
    pub paths: Vec<String>,
    pub persist_mode: crate::container::persist::PersistMode,
    pub container_name: String,
}
