use crate::types::permissions::{Permission, PermissionKey};

pub fn filter_permissions(key: PermissionKey, permissions: Vec<Permission>) -> Vec<Permission> {
    permissions.into_iter().filter(|p| p.key == key).collect()
}
