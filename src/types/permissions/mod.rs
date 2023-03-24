mod path;

pub trait TPerm {
    fn to_markdown()->String
}

pub enum PermissionLevel {
    Normal,
    Important,
    Sensitive
}

pub enum PermissionContent {
    PermPath(PermPath),
}

pub struct Permission {
    pub level:PermissionLevel,
    pub content:PermissionContent
}