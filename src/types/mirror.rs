use std::{
    collections::HashMap,
    time::{SystemTime, UNIX_EPOCH},
};

use crate::utils::{download::fill_url_template, mirror::filter_service_from_meta};
use anyhow::Result;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use super::{extended_semver::ExSemVer, verifiable::Verifiable};

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
    fn verify_self(&self, _located: &String) -> Result<()> {
        // 必须有 hello 服务
        let _hello_res = filter_service_from_meta(self.clone(), ServiceKeys::Hello)?;

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
    fn verify_self(&self, _located: &String) -> Result<()> {
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
                    version: ExSemVer::parse(&"1.85.1.0".to_string()).unwrap(),
                    size: 94245376,
                    timestamp: 1704554724,
                    integrity: None,
                }],
            }],
        );
        tree.insert(
            "github".to_string(),
            vec![TreeItem {
                name: "Visual Studio Code Portable".to_string(),
                releases: vec![MirrorPkgSoftwareRelease {
                    file_name: "VSCode_1.85.1.0_Cno.nep".to_string(),
                    version: ExSemVer::parse(&"1.85.1.0".to_string()).unwrap(),
                    size: 94245376,
                    timestamp: 1704554724,
                    integrity: None,
                }],
            }],
        );
        tree.insert(
            "Google".to_string(),
            vec![TreeItem {
                name: "Chrome".to_string(),
                releases: vec![MirrorPkgSoftwareRelease {
                    file_name: "Chrome_120.0.6099.200_Cno.nep".to_string(),
                    version: ExSemVer::parse(&"120.0.6099.200".to_string()).unwrap(),
                    size: 133763072,
                    timestamp: 1704554608,
                    integrity: None,
                }],
            }],
        );
        tree.insert(
            "360极速浏览器X".to_string(),
            vec![TreeItem {
                name: "Chrome".to_string(),
                releases: vec![MirrorPkgSoftwareRelease {
                    file_name: "360极速浏览器X_22.1.1073.64_Cno.nep".to_string(),
                    version: ExSemVer::parse(&"22.1.1073.64".to_string()).unwrap(),
                    size: 192179712,
                    timestamp: 1704554110,
                    integrity: None,
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
    // meta 和 permissions 在这里被省略
}
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct SearchResult {
    pub name: String,
    pub scope: String,
    pub version: String,
    pub from_mirror: Option<String>,
}

#[test]
fn test_mirror_pkg_software() {
    MirrorPkgSoftware::_demo()
        .verify_self(&"".to_string())
        .unwrap()
}
