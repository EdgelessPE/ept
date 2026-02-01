use std::{
    collections::HashMap,
    time::{SystemTime, UNIX_EPOCH},
};

use crate::utils::{download::fill_url_template, mirror::filter_service_from_meta};
use anyhow::Result;
use colored::Colorize;
use regex::Regex;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use super::{
    extended_semver::ExSemVer,
    meta::MetaResult,
    verifiable::{Verifiable, VerifiableCtx},
};

lazy_static! {
    static ref FLAGS_RE: Regex = Regex::new(r"\.([A-Z]+)\.nep$").unwrap();
}

#[derive(Debug, PartialEq, Clone)]
pub enum Locale {
    ZhCn,
    EnUs,
    Multi,
}

#[derive(Debug, PartialEq, Clone)]
pub enum ServiceKeys {
    Hello,
    EptToolchain,
    PkgSoftware,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct MirrorHello {
    pub name: String,
    pub locale: Locale,
    pub description: String,
    pub maintainer: String,
    pub protocol: String,
    pub root_url: String,
    pub property: Property,
    pub service: Vec<Service>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct Property {
    pub deploy_region: Locale,
    pub proxy_storage: bool,
    pub upload_bandwidth: u64,
    pub sync_interval: u64,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct QuickMaps {
    // 用于通过小写名称查找 scope 和真实名称的 Map
    // HashMap<小写名称, (可能的 scope, 真实名称)>
    pub scope_map: HashMap<String, (Vec<String>, String)>,
    // 用于通过小写 scope 和小写名称查找 TreeItem 的 Map
    // HashMap<(小写 scope, 小写名称), TreeItem>>
    pub full_map: HashMap<(String, String), TreeItem>,
    pub url_template: String,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct Service {
    pub key: ServiceKeys,
    pub path: String,
}

impl Serialize for Locale {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let s = match self {
            Locale::ZhCn => "zh-CN",
            Locale::EnUs => "en-US",
            Locale::Multi => "Multi",
        };
        serializer.serialize_str(s)
    }
}

impl<'de> Deserialize<'de> for Locale {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        match s.as_str() {
            "zh-CN" => Ok(Locale::ZhCn),
            "en-US" => Ok(Locale::EnUs),
            "Multi" => Ok(Locale::Multi),
            _ => Err(serde::de::Error::custom("Error:Invalid locale variant")),
        }
    }
}
impl Serialize for ServiceKeys {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let s = match self {
            ServiceKeys::Hello => "HELLO",
            ServiceKeys::EptToolchain => "EPT_TOOLCHAIN",
            ServiceKeys::PkgSoftware => "PKG_SOFTWARE",
        };
        serializer.serialize_str(s)
    }
}

impl<'de> Deserialize<'de> for ServiceKeys {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        match s.as_str() {
            "HELLO" => Ok(ServiceKeys::Hello),
            "EPT_TOOLCHAIN" => Ok(ServiceKeys::EptToolchain),
            "PKG_SOFTWARE" => Ok(ServiceKeys::PkgSoftware),
            _ => Err(serde::de::Error::custom("Error:Invalid service key")),
        }
    }
}

impl Verifiable for MirrorHello {
    fn verify_self(&self, _ctx: &VerifiableCtx) -> Result<()> {
        // 必须有 hello 服务
        filter_service_from_meta(self, ServiceKeys::Hello)?;

        // 协议版本号为 1.0.0
        if self.protocol != "1.0.0" {
            return Err(anyhow::anyhow!(
                "Error:Invalid protocol version '{}', expected '1.0.0'",
                self.protocol
            ));
        }

        Ok(())
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct MirrorPkgSoftware {
    pub timestamp: u64,
    pub url_template: String,
    pub tree: HashMap<String, Vec<TreeItem>>,
}

impl Verifiable for MirrorPkgSoftware {
    fn verify_self(&self, _ctx: &VerifiableCtx) -> Result<()> {
        // 检查 url 模板
        let str = String::new();
        fill_url_template(&self.url_template, &str, &str, &str)?;

        Ok(())
    }
}

impl MirrorPkgSoftware {
    pub fn _demo() -> Self {
        let mut tree = HashMap::new();
        tree.insert(
            "Microsoft".to_string(),
            vec![TreeItem {
                name: "Visual Studio Code".to_string(),
                releases: vec![MirrorPkgSoftwareRelease {
                    file_name: "VSCode_1.85.1.0_Cno.nep".to_string(),
                    version: ExSemVer::parse("1.85.1.0").unwrap(),
                    size: 94245376,
                    timestamp: 1704554724,
                    integrity: None,
                    meta: None,
                }],
            }],
        );
        tree.insert(
            "github".to_string(),
            vec![TreeItem {
                name: "Visual Studio Code Portable".to_string(),
                releases: vec![MirrorPkgSoftwareRelease {
                    file_name: "VSCode_1.85.1.0_Cno.P.nep".to_string(),
                    version: ExSemVer::parse("1.85.1.0").unwrap(),
                    size: 94245376,
                    timestamp: 1704554724,
                    integrity: None,
                    meta: None,
                }],
            }],
        );
        tree.insert(
            "Google".to_string(),
            vec![TreeItem {
                name: "Chrome".to_string(),
                releases: vec![MirrorPkgSoftwareRelease {
                    file_name: "Chrome_120.0.6099.200_Cno.EI.nep".to_string(),
                    version: ExSemVer::parse("120.0.6099.200").unwrap(),
                    size: 133763072,
                    timestamp: 1704554608,
                    integrity: None,
                    meta: None,
                }],
            }],
        );
        tree.insert(
            "360极速浏览器X".to_string(),
            vec![TreeItem {
                name: "Chrome".to_string(),
                releases: vec![MirrorPkgSoftwareRelease {
                    file_name: "360极速浏览器X_22.1.1073.64_Cno.nep".to_string(),
                    version: ExSemVer::parse("22.1.1073.64").unwrap(),
                    size: 192179712,
                    timestamp: 1704554110,
                    integrity: None,
                    meta: None,
                }],
            }],
        );
        Self {
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_micros() as u64,
            url_template:
                "http:/localhost:3000/api/redirect?path=/nep/{scope}/{software}/{file_name}"
                    .to_string(),
            tree,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct TreeItem {
    pub name: String,
    pub releases: Vec<MirrorPkgSoftwareRelease>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct MirrorPkgSoftwareRelease {
    pub file_name: String,
    pub version: ExSemVer,
    pub size: u64,
    pub timestamp: u64,
    pub integrity: Option<String>,
    pub meta: Option<MetaResult>,
}

impl MirrorPkgSoftwareRelease {
    pub fn get_flags(&self) -> Option<String> {
        // 正则匹配 flags
        FLAGS_RE
            .captures(&self.file_name)
            .and_then(|cap| cap.get(1))
            .map(|m| m.as_str().to_string())
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct SearchResult {
    pub name: String,
    pub scope: String,
    pub version: String,
    pub description: String,
    pub from_mirror: Option<String>,
}

impl SearchResult {
    pub fn fmt_brief_print(&self) -> String {
        format!(
            "· {}/{} ({})\n  {} {}\n",
            self.scope.italic(),
            self.name.bold(),
            self.version,
            "Description:".truecolor(100, 100, 100),
            self.description.truecolor(100, 100, 100)
        )
    }
}

pub struct MirrorInfo {
    pub name: String,
    pub updated_at: SystemTime,
    pub root_url: String,
}

#[test]
fn test_mirror_pkg_software() {
    use crate::types::{cfg::Cfg, mixed_fs::MixedFS, verifiable::VerifiableCtx};
    use crate::utils::test::_default_test_cfg;

    let mixed_fs = MixedFS::new("");
    let cfg = _default_test_cfg();
    let ctx = VerifiableCtx {
        mixed_fs: &mixed_fs,
        cfg: &cfg,
    };
    MirrorPkgSoftware::_demo().verify_self(&ctx).unwrap()
}

#[test]
fn test_get_flags() {
    use std::str::FromStr;
    let r = MirrorPkgSoftwareRelease {
        file_name: "VSCode_1.75.0.0_Cno.nep".to_string(),
        version: ExSemVer::from_str("1.75.0.0").unwrap(),
        size: 114514,
        timestamp: 114514,
        integrity: None,
        meta: None,
    };
    assert_eq!(r.get_flags(), None);

    let r = MirrorPkgSoftwareRelease {
        file_name: "VSCode_1.75.0.0_Cno.P.nep".to_string(),
        version: ExSemVer::from_str("1.75.0.0").unwrap(),
        size: 114514,
        timestamp: 114514,
        integrity: None,
        meta: None,
    };
    assert_eq!(r.get_flags(), Some("P".to_string()));

    let r = MirrorPkgSoftwareRelease {
        file_name: "VSCode_1.75.0.0_Cno.EI.nep".to_string(),
        version: ExSemVer::from_str("1.75.0.0").unwrap(),
        size: 114514,
        timestamp: 114514,
        integrity: None,
        meta: None,
    };
    assert_eq!(r.get_flags(), Some("EI".to_string()));

    let r = MirrorPkgSoftwareRelease {
        file_name: "VSCode_1.75.0.0_C. n.o.EI.nep".to_string(),
        version: ExSemVer::from_str("1.75.0.0").unwrap(),
        size: 114514,
        timestamp: 114514,
        integrity: None,
        meta: None,
    };
    assert_eq!(r.get_flags(), Some("EI".to_string()));

    let r = MirrorPkgSoftwareRelease {
        file_name: "VSCode_1.75.0.0_C.n.o.ei.nep".to_string(),
        version: ExSemVer::from_str("1.75.0.0").unwrap(),
        size: 114514,
        timestamp: 114514,
        integrity: None,
        meta: None,
    };
    assert_eq!(r.get_flags(), None);
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MirrorEptToolchain {
    pub update: MirrorEptToolchainUpdate,
    pub releases: Vec<MirrorEptToolchainRelease>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MirrorEptToolchainUpdate {
    pub wild_gaps: Vec<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MirrorEptToolchainRelease {
    pub name: String,
    pub version: String,
    pub url: String,
    pub size: i64,
    pub timestamp: i64,
}
