use std::path::PathBuf;

use super::{package::GlobalPackage, permissions::Permission};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug, TS)]
#[ts(export)]
pub struct MetaResult {
    pub temp_dir: Option<PathBuf>,
    pub permissions: Vec<Permission>,
    pub workflows: Vec<String>,
    pub package: GlobalPackage,
}
