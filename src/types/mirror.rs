use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[derive(Debug, PartialEq, Clone)]
pub enum Locale {
    ZhCn,
    EnUs,
    Multi,
}

#[derive(Debug, PartialEq, Clone)]
pub enum ServiceKeys {
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
            "EPT_TOOLCHAIN" => Ok(ServiceKeys::EptToolchain),
            "PKG_SOFTWARE" => Ok(ServiceKeys::PkgSoftware),
            _ => Err(serde::de::Error::custom("invalid service key")),
        }
    }
}
