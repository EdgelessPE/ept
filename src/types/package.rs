use serde::{Deserialize, Serialize};

use crate::types::software::Software;

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

impl GlobalPackage {
    pub fn _demo() -> Self {
        GlobalPackage {
            nep: "0.2".to_string(),
            package: Package {
                name: "ept".to_string(),
                description: "demo package".to_string(),
                template: "Software".to_string(),
                version: "1.0.0".to_string(),
                authors: vec!["Cno".to_string()],
                license: None,
            },
            software: Some(Software {
                scope: "Edgeless".to_string(),
                upstream: "https://github.com/EdgelessPE/ept".to_string(),
                category: "实用工具".to_string(),
                language: "en-US".to_string(),
                main_program: None,
                tags: None,
            }),
        }
    }
}
