use super::{package::GlobalPackage, permissions::Permission};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Serialize, Deserialize, Clone, Debug, TS)]
#[ts(export)]
pub struct MetaResult {
    pub temp_dir: String,
    pub permissions: Vec<Permission>,
    pub workflows: Vec<String>,
    pub package: GlobalPackage,
}
