use anyhow::Error;
use colored::Colorize;
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};

use crate::types::software::Software;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Info {
    pub name: String,
    pub template: String,
    pub license: Option<String>,
    pub local: Option<InfoDiff>,
    pub online: Option<InfoDiff>,
    pub software: Option<Software>,
}

// 线上与本地的差异点
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct InfoDiff {
    pub version: String,
    pub authors: Vec<String>,
}

pub struct UpdateInfo {
    pub name: String,
    pub scope: String,
    pub from_version: String,
    pub to_version: String,
}

impl Display for UpdateInfo {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(
            f,
            "  {scope}/{name}    ({from_version} → {to_version})",
            scope = &self.scope.cyan().italic(),
            name = &self.name.cyan().bold(),
            from_version = &self.from_version.yellow(),
            to_version = &self.to_version.green()
        )
    }
}

impl UpdateInfo {
    pub fn format_success(&self) -> String {
        format!(
            "Success:Package '{scope}/{name}' updated successfully from '{from_ver}' to '{to_ver}'",
            scope = &self.scope,
            name = &self.name,
            from_ver = &self.from_version,
            to_ver = &self.to_version
        )
    }
    pub fn format_failure(&self, e: Error) -> String {
        format!(
            "Error:Failed to update '{scope}/{name}' from '{from_ver}' to '{to_ver}' : {e}",
            scope = &self.scope,
            name = &self.name,
            from_ver = &self.from_version,
            to_ver = &self.to_version
        )
    }
}

#[test]
fn test_update_info() {
    let info = UpdateInfo {
        name: "Visual Studio Code".to_string(),
        scope: "Microsoft".to_string(),
        from_version: "1.0.0".to_string(),
        to_version: "2.0.0".to_string(),
    };
    println!("{}", info);
    println!("{}", info.format_success());
    println!("{}", info.format_failure(Error::msg("test error")));
}
