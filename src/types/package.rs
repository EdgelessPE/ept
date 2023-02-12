use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Package {
    pub name: String,
    pub template: String,
    pub version: String,
    pub authors: Vec<String>,
    pub license: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Software {
    pub upstream: String,
    pub category: String,
    pub main_program: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct GlobalPackage {
    pub nep: String,
    pub package: Package,
    pub software: Option<Software>,
}
