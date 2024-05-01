use std::collections::HashMap;

use crate::utils::mirror::filter_service_from_meta;
use anyhow::Result;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use super::verifiable::Verifiable;

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
            _ => Err(serde::de::Error::custom("invalid locale variant")),
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
            _ => Err(serde::de::Error::custom("invalid service key")),
        }
    }
}

impl Verifiable for MirrorHello {
    fn verify_self(&self, _located: &String) -> Result<()> {
        // 必须有 hello 服务
        let hello_res = filter_service_from_meta(self.clone(), ServiceKeys::Hello);
        if let Err(e) = hello_res {
            return Err(e);
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

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct TreeItem {
    pub name: String,
    pub releases: Vec<MirrorPkgSoftwareRelease>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct MirrorPkgSoftwareRelease {
    pub file_name: String,
    pub size: u64,
    pub timestamp: u64,
    pub integrity: Option<String>,
    // meta 和 permissions 在这里被省略
}
