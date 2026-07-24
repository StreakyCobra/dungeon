use std::collections::BTreeMap;

use crate::config::Settings;

#[derive(Debug, Clone)]
pub struct ParsedCLI {
    pub action: Action,
    pub settings: Settings,
    pub paths: Vec<String>,
    pub show_help: bool,
    pub show_version: bool,
    pub debug: bool,
    pub group_flags: BTreeMap<String, GroupFlag>,
    pub skip_cwd: bool,
}

#[derive(Debug, Clone)]
pub enum Action {
    None,
    Run,
    ImageBuild(ImageBuildAction),
    CacheReset(CacheResetAction),
}

#[derive(Debug, Clone)]
pub struct ImageBuildAction {
    pub tag: String,
    pub no_cache: bool,
    pub context: String,
}

#[derive(Debug, Clone)]
pub struct CacheResetAction;

#[derive(Default, Clone, Debug)]
pub struct GroupFlag {
    pub set: bool,
    pub order: usize,
}

pub fn build_version() -> String {
    let version = env!("CARGO_PKG_VERSION");
    if !version.is_empty() {
        return version.to_string();
    }
    "dev".to_string()
}
