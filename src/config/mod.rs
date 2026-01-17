mod groups;
mod merge;
mod parse;
mod types;

pub use groups::{
    build_group_selection, merge_group_definitions, normalize_group_order, resolve_group_order,
};
pub use merge::{resolve_always_on_groups, resolve_persist_mode, resolve_settings};
pub use types::{Config, GroupConfig, Settings, Sources};

use crate::error::AppError;

pub fn load_defaults() -> Result<Config, AppError> {
    parse::load_defaults()
}

pub fn load_from_file() -> Result<Config, AppError> {
    parse::load_from_file()
}

pub fn load_from_env() -> Result<Config, AppError> {
    parse::load_from_env()
}
