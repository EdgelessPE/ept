use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone,Debug)]
pub struct Package {
    pub name:String,
    pub template:String,
    pub version:String,
    pub authors:Vec<String>,
    pub licence:String
}

#[derive(Serialize, Deserialize, Clone,Debug)]
pub struct Software {
    pub upstream:String,
    pub category:String
}

#[derive(Serialize, Deserialize, Clone,Debug)]
pub struct GlobalPackage {
    pub nep:String,
    pub package:Package,
    pub software:Software
}