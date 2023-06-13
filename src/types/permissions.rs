use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, PartialOrd,Eq,Hash)]
pub enum PermissionLevel {
    Normal,
    Important,
    Sensitive,
}
#[derive(Serialize, Deserialize, Clone, Debug,PartialEq)]
pub struct Permission {
    pub key: String,
    pub level: PermissionLevel,
    pub targets: Vec<String>,
}

pub trait Generalizable {
    fn generalize_permissions(&self) -> Result<Vec<Permission>>;
}
