use std::collections::BTreeMap;

use crate::{
    config::{Engine, Settings},
    container::persist::PersistMode,
};

#[derive(Debug, Clone)]
pub struct ParsedCLI {
    pub action: Action,
    pub settings: Settings,
    pub paths: Vec<String>,
    pub show_help: bool,
    pub show_version: bool,
    pub debug: bool,
    pub persist_mode: PersistMode,
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
    pub engine: Engine,
    pub flavor: ImageFlavor,
    pub tag: String,
    pub no_cache: bool,
    pub context: String,
}

#[derive(Debug, Clone)]
pub struct CacheResetAction {
    pub engine: Engine,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImageFlavor {
    Archlinux,
    Ubuntu,
}

impl ImageFlavor {
    pub fn containerfile_path(self) -> &'static str {
        match self {
            ImageFlavor::Archlinux => "images/Containerfile.archlinux",
            ImageFlavor::Ubuntu => "images/Containerfile.ubuntu",
        }
    }
}

#[derive(Default, Clone, Debug)]
pub struct GroupFlag {
    pub set: bool,
    pub order: usize,
}

pub fn build_version() -> String {
    let version = env!("CARGO_PKG_VERSION");
    if version != "" {
        return version.to_string();
    }
    "dev".to_string()
}
