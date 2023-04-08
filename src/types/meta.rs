use super::{package::GlobalPackage, permissions::Permission};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct MetaResult {
    pub temp_dir: String,
    pub permissions: Vec<Permission>,
    pub workflows: Vec<String>,
    pub package: GlobalPackage,
}
