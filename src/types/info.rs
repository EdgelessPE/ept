use anyhow::{Error, Result};
use colored::Colorize;
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};

use crate::utils::fmt_print::{FmtPrint, FmtPrintCaller};

use super::extended_semver::ExSemVer;
use super::meta::MetaResult;
use super::permissions::PermissionLevel;

#[derive(Serialize, Deserialize, PartialEq, Clone, Debug)]
pub struct Info {
    pub name: String,
    pub scope: String,

    pub local: Option<InfoDiff>,
    pub online: Option<InfoDiff>,

    // pub package: Option<Package>,
    // pub software: Option<Software>,
    pub meta: Option<MetaResult>,
}

// 线上与本地的差异点
#[derive(Serialize, Deserialize, PartialEq, Clone, Debug)]
pub struct InfoDiff {
    pub version: String,
    pub authors: Vec<String>,
}

impl Default for InfoDiff {
    fn default() -> Self {
        Self {
            version: "0.0.0.0".to_string(),
            authors: vec![],
        }
    }
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
    fn fmt_print(&self, fmt_caller: FmtPrintCaller) -> Result<String> {
        let mut output = String::new();

        // 版本提示
        let has_installed = self.local.is_some();
        let local_ver = self.local.clone().unwrap_or_default().version;
        let online_ver = self.online.clone().unwrap_or_default().version;
        let has_update = ExSemVer::parse(&local_ver)? < ExSemVer::parse(&online_ver)?;
        let local_tip = format!("({local_ver})");
        let online_tip = format!("({online_ver})");
        let updated_tip = format!("(✅ {local_ver})");
        let has_update_tip = format!("({local_ver} ➡️  {online_ver})");
        let version_tip = match fmt_caller {
            FmtPrintCaller::Info => {
                if !has_installed {
                    online_tip
                } else if has_update {
                    has_update_tip
                } else {
                    updated_tip
                }
            }
            FmtPrintCaller::Install => online_tip,
            FmtPrintCaller::Update => has_update_tip,
            FmtPrintCaller::Uninstall => local_tip,
        };

        // 标题行
        output.push_str(&format!(
            "{}/{} {}\n",
            self.scope.italic(),
            self.name.bold(),
            version_tip.truecolor(100, 100, 100)
        ));

        // 分割线
        output.push_str(&"-".repeat(71));
        output.push('\n');

        if let Some(meta) = &self.meta {
            let package = &meta.package.package;
            // Basic 部分
            output.push_str(&format!("{}\n", "Basic".bold()));
            output.push_str(&format!("· 📝 Description: {}\n", package.description));
            output.push_str(&format!(
                "· 👤 Author:      {}\n",
                package.authors.join(", ")
            ));
            if let Some(license) = &package.license {
                output.push_str(&format!("· 📜 License:     {}\n", license));
            }
            output.push('\n');

            // Software 部分
            if let Some(software) = &meta.package.software {
                output.push_str(&format!("{}\n", "Software".bold()));
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
                output.push_str(&format!("{}\n", "Meta".bold()));
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

        Ok(output)
    }
}

#[test]
fn test_info() {
    use crate::types::package::GlobalPackage;
    use crate::types::permissions::Permission;
    let demo_pkg = GlobalPackage::_demo();
    let info = Info {
        name: "VSCode".to_string(),
        scope: "Microsoft".to_string(),
        local: Some(InfoDiff {
            version: "1.77.3".to_string(),
            authors: vec!["Microsoft".to_string()],
        }),
        online: Some(InfoDiff {
            version: "1.77.4".to_string(),
            authors: vec!["Microsoft".to_string()],
        }),
        // package: Some(demo_pkg.package.clone()),
        // software: demo_pkg.software.clone(),
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
    println!("{}", info.fmt_print(FmtPrintCaller::Info).unwrap());
}
