#[macro_use]
extern crate lazy_static;
extern crate tar;
#[macro_use]
extern crate tantivy;

pub mod ca;
pub mod compression;
pub mod entrances;
pub mod executor;
pub mod parsers;
pub mod signature;
#[macro_use]
pub mod types;
#[macro_use]
pub mod utils;

// Re-export commonly used types
pub use types::{
    cfg::Cfg,
    cli::{Action, ActionConfig, Args},
};

// Re-export entrance functions
pub use entrances::{
    auto_mirror_update_all, clean,
    config::{config_get, config_init, config_list, config_set, config_which},
    info, install_using_package, install_using_parsed, list, meta, mirror_add, mirror_list,
    mirror_remove, mirror_update, mirror_update_all, pack, search, uninstall, update_all,
    update_using_parsed, upgrade,
};

// Re-export utility functions
pub use utils::{
    flags::{get_flag, set_flag, Flag},
    launch_clean,
};
