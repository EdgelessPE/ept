use serde::{Deserialize, Serialize};

use super::Software;

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
