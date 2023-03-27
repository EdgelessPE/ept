use anyhow::Result;

pub enum PermissionLevel {
    Normal,
    Important,
    Sensitive
}

pub struct Permission {
    pub key:String,
    pub level:PermissionLevel,
    pub targets:Vec<String>,
}

pub trait Generalizable {
    fn generalize_permissions(&self)->Result<Vec<Permission>>;
}