use anyhow::Error;
use colored::Colorize;
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};

use crate::types::software::Software;
use crate::utils::fmt_print::{FmtPrint, FmtPrintCaller};

use super::meta::MetaResult;
use super::package::Package;
use super::permissions::PermissionLevel;

#[derive(Serialize, Deserialize, PartialEq, Clone, Debug)]
pub struct Info {
    pub name: String,
    pub scope: String,

    pub local: Option<InfoDiff>,
    pub online: Option<InfoDiff>,

    pub package: Option<Package>,
    pub software: Option<Software>,

    pub meta: Option<MetaResult>,
}

// 线上与本地的差异点
#[derive(Serialize, Deserialize, PartialEq, Clone, Debug)]
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

impl FmtPrint for Info {
    fn fmt_print(&self, _fmt_caller: FmtPrintCaller) -> String {
        let mut output = String::new();

        // 标题行
        output.push_str(&format!(
            "{}/{} ({}✅)\n",
            self.scope.italic(),
            self.name.bold(),
            self.local.as_ref().map_or("unknown", |l| &l.version)
        ));

        // 分割线
        output.push_str(&"-".repeat(71));
        output.push('\n');

        // Basic 部分
        output.push_str(&"Basic\n".bold());
        if let Some(package) = &self.package {
            output.push_str(&format!("· 📝 Description: {}\n", package.description));
            output.push_str(&format!(
                "· 👤 Author:      {}\n",
                package.authors.join(", ")
            ));
            if let Some(license) = &package.license {
                output.push_str(&format!("· 📜 License:     {}\n", license));
            }
            output.push('\n');
        }

        // Software 部分
        if let Some(software) = &self.software {
            output.push_str(&"Software\n".bold());
            output.push_str(&format!("· 🔗 Upstream:    {}\n", software.upstream));
            output.push_str(&format!("· 📂 Category:    {}\n", software.category));
            if let Some(arch) = &software.arch {
                output.push_str(&format!("· 🖥️ Arch:        {}\n", arch));
            }
            output.push_str(&format!("· 🌐 Language:    {}\n", software.language));
            if let Some(alias) = &software.alias {
                output.push_str(&format!("· 🌟 Alias:       {}\n", alias));
            }
            if let Some(tags) = &software.tags {
                output.push_str(&format!("· 🏷️ Tags:        {}\n", tags.join(", ")));
            }
            output.push('\n');

            // Meta 部分（权限）
            if let Some(meta) = &self.meta {
                output.push_str(&"Meta\n".bold());
                output.push_str("· 🛡️ Permissions: \n");
                for perm in &meta.permissions {
                    let key: &'static str = perm.key.clone().into();
                    let level: &'static str = perm.level.clone().into();
                    output.push_str(&format!("    · 👀 Key:     {}\n", key));
                    output.push_str(&format!(
                        "    · {} Level:   {}\n",
                        match perm.level {
                            PermissionLevel::Sensitive => "🔴",
                            PermissionLevel::Important => "🟡",
                            PermissionLevel::Normal => "🔵",
                        },
                        level
                    ));
                    output.push_str("    · 🎯 Targets: \n");
                    for target in &perm.targets {
                        output.push_str(&format!("      · {}\n", target));
                    }
                    output.push('\n');
                }
            }
        }

        output.push_str(&"-".repeat(71));
        output.push('\n');

        output
    }
}

#[test]
fn test_info() {
    let demo_pkg = GlobalPackage::_demo();
    let info = Info {
        name: "VSCode".to_string(),
        scope: "Microsoft".to_string(),
        local: Some(InfoDiff {
            version: "1.77.3".to_string(),
            authors: vec!["Microsoft".to_string()],
        }),
        online: Some(InfoDiff {
            version: "1.77.3".to_string(),
            authors: vec!["Microsoft".to_string()],
        }),
        package: Some(demo_pkg.package.clone()),
        software: demo_pkg.software.clone(),
        meta: Some(MetaResult {
            temp_dir: None,
            permissions: vec![Permission {
                key: super::permissions::PermissionKey::execute_installer,
                level: PermissionLevel::Important,
                targets: vec!["installer.exe".to_string()],
            }],
            workflows: vec!["setup.toml".to_string(), "remove.toml".to_string()],
            package: demo_pkg,
        }),
    };
    println!("{}", info.fmt_print(FmtPrintCaller::Install));
}
