use anyhow::Result;
use serde::{Deserialize, Serialize};
use strum_macros::EnumString;
use ts_rs::TS;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, PartialOrd, Eq, Hash, TS)]
#[ts(export)]
pub enum PermissionLevel {
    Normal,
    Important,
    Sensitive,
}

#[derive(
    Serialize, Deserialize, Clone, Debug, PartialEq, PartialOrd, Eq, Ord, Hash, TS, EnumString,
)]
#[allow(non_camel_case_types)]
pub enum PermissionKey {
    path_entrances,
    path_dirs,
    link_desktop,
    link_startmenu,
    execute_installer,
    execute_custom,
    fs_read,
    fs_write,
    process_query,
    nep_installed,
    notify_toast,
    process_kill,
}
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, TS)]
#[ts(export)]
pub struct Permission {
    pub key: PermissionKey,
    pub level: PermissionLevel,
    pub targets: Vec<String>,
}

pub trait Generalizable {
    fn generalize_permissions(&self) -> Result<Vec<Permission>>;
}
