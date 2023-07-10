use anyhow::Result;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, PartialOrd, Eq, Hash, TS)]
#[ts(export)]
pub enum PermissionLevel {
    Normal,
    Important,
    Sensitive,
}
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, TS)]
#[ts(export)]
pub struct Permission {
    pub key: String,
    pub level: PermissionLevel,
    pub targets: Vec<String>,
}

pub trait Generalizable {
    fn generalize_permissions(&self) -> Result<Vec<Permission>>;
}
