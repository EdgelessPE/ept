use serde::{Deserialize, Serialize};

use super::Software;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Package {
    pub name: String,
    pub description: String,
    pub template: String,
    pub version: String,
    pub authors: Vec<String>,
    pub license: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct GlobalPackage {
    pub nep: String,
    pub package: Package,
    pub software: Option<Software>,
}
