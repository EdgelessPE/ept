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

impl Info {
    pub fn get_common_tips(&self, fmt_caller: &FmtPrintCaller) -> Result<(String, String)> {
        // 更新提示
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
            FmtPrintCaller::Install(_) => online_tip,
            FmtPrintCaller::Update(_) => has_update_tip,
            FmtPrintCaller::Uninstall => local_tip,
        };

        // 标题
        let title = format!(
            "{}/{} {}\n",
            self.scope.italic(),
            self.name.bold(),
            version_tip
        );

        // 来源行
        let source_tip = match fmt_caller {
            FmtPrintCaller::Install(matcher) => format!(
                "{}{}\n",
                "Source: ".truecolor(100, 100, 100),
                matcher.to_string().truecolor(100, 100, 100)
            ),
            FmtPrintCaller::Update(matcher) => format!(
                "{}{}\n",
                "Source: ".truecolor(100, 100, 100),
                matcher.to_string().truecolor(100, 100, 100)
            ),
            _ => "".to_string(),
        };

        Ok((title, source_tip))
    }
}

impl FmtPrint for Info {
    fn fmt_print(&self, fmt_caller: FmtPrintCaller) -> Result<String> {
        let mut output = String::new();

        let (title, source) = self.get_common_tips(&fmt_caller)?;

        // 标题行
        output.push_str(&title);

        // 来源行
        output.push_str(&source);

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
                    if !tags.is_empty() {
                        output.push_str(&format!("· 🏷️ Tags:        {}\n", tags.join(", ")));
                    }
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
                }
            }
        }

        output.push_str(&"-".repeat(71));
        output.push('\n');

        Ok(output)
    }
    fn fmt_brief_print(&self, fmt_caller: FmtPrintCaller) -> Result<String> {
        let mut output = String::new();
        let (title, source) = self.get_common_tips(&fmt_caller)?;
        output.push_str(&format!("· {title}"));
        if !source.is_empty() {
            output.push_str(&format!("  {source}"));
        }

        // 收集权限简报
        let need_permission = matches!(
            fmt_caller,
            FmtPrintCaller::Install(_) | FmtPrintCaller::Update(_)
        );
        if need_permission {
            let mut sensitive_count = 0;
            let mut important_count = 0;
            for perm in &self.meta.as_ref().unwrap().permissions {
                match perm.level {
                    PermissionLevel::Sensitive => sensitive_count += 1,
                    PermissionLevel::Important => important_count += 1,
                    _ => {}
                }
            }
            if sensitive_count + important_count > 0 {
                let mut perm = format!("{}", "Permission:".truecolor(100, 100, 100));
                if sensitive_count > 0 {
                    perm.push_str(&format!(
                        " {} {}",
                        sensitive_count.to_string().red(),
                        "Sensitive".red()
                    ));
                }
                if important_count > 0 {
                    perm.push_str(&format!(
                        " {} {}",
                        important_count.to_string().yellow(),
                        "Important".yellow()
                    ));
                }

                output.push_str(&format!("  {}\n", perm));
            }
        }

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
            permissions: vec![
                Permission {
                    key: super::permissions::PermissionKey::execute_custom,
                    level: PermissionLevel::Sensitive,
                    targets: vec!["cmd.exe".to_string()],
                },
                Permission {
                    key: super::permissions::PermissionKey::execute_installer,
                    level: PermissionLevel::Important,
                    targets: vec!["installer.exe".to_string()],
                },
                Permission {
                    key: super::permissions::PermissionKey::link_desktop,
                    level: PermissionLevel::Normal,
                    targets: vec!["Install".to_string()],
                },
            ],
            workflows: vec!["setup.toml".to_string(), "remove.toml".to_string()],
            package: demo_pkg,
        }),
    };
    println!(
        "{}",
        info.fmt_print(FmtPrintCaller::Install(
            crate::utils::fmt_print::PackageSource::Mirror("Official".to_string())
        ))
        .unwrap()
    );
    println!(
        "{}",
        info.fmt_brief_print(FmtPrintCaller::Update(
            crate::utils::fmt_print::PackageSource::Url("https://114.514/sodayo.nep".to_string())
        ))
        .unwrap()
    );
}
