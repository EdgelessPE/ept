use crate::types::software::Software;
use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::path::Path;
use ts_rs::TS;

use super::{extended_semver::ExSemVer, verifiable::Verifiable};

#[derive(Serialize, Deserialize, Clone, Debug, TS)]
#[ts(export)]
pub struct Package {
    pub name: String,
    pub description: String,
    pub template: String,
    pub version: String,
    pub authors: Vec<String>,
    pub license: Option<String>,
}

impl Verifiable for Package {
    fn verify_self(&self, _: &String) -> Result<()> {
        let err_wrapper = |e: anyhow::Error| {
            anyhow!("Error:Failed to verify table 'package' in 'package.toml' : {e}")
        };

        // 模板只能是 Software
        if self.template != "Software".to_string() {
            return Err(err_wrapper(anyhow!(
                "field 'template' should be 'Software'"
            )));
        }

        // 版本号必须可以解析
        ExSemVer::parse(&self.version).map_err(err_wrapper)?;

        Ok(())
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, TS)]
#[ts(export)]
pub struct GlobalPackage {
    pub nep: String,
    pub package: Package,
    pub software: Option<Software>,
}

impl GlobalPackage {
    pub fn _demo() -> Self {
        GlobalPackage {
            nep: env!("CARGO_PKG_VERSION")[0..3].to_string(),
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
                arch: None,
                language: "en-US".to_string(),
                main_program: None,
                tags: None,
                alias: None,
                installed: None,
            }),
        }
    }
}

impl Verifiable for GlobalPackage {
    fn verify_self(&self, located: &String) -> Result<()> {
        if !Path::new(located).exists() {
            return Err(anyhow!("Error:Path '{located}' not exist"));
        }
        self.package.verify_self(located)?;
        if let Some(software) = &self.software {
            software.verify_self(located)?;
        }

        Ok(())
    }
}
