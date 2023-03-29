use std::{collections::HashSet, path::Path};

use crate::utils::format_path;

pub struct MixedFS {
    to_add: HashSet<String>,
    to_remove: HashSet<String>,
}

impl MixedFS {
    pub fn new() -> Self {
        MixedFS {
            to_add: HashSet::new(),
            to_remove: HashSet::new(),
        }
    }
    pub fn add(&mut self, path: &String) {
        let path = format_path(path);
        self.to_remove.remove(&path);
        self.to_add.insert(path);
    }
    pub fn remove(&mut self, path: &String) {
        let path = format_path(path);
        self.to_add.remove(&path);
        self.to_remove.insert(path);
    }

    pub fn exists(&self, path: &String, base: &String) -> bool {
        let path = format_path(path);
        if self.to_add.contains(&path) {
            return true;
        }
        if self.to_remove.contains(&path) {
            return false;
        }
        // 此处不关心内置变量是否合法
        if path.starts_with("${") {
            return true;
        }
        Path::new(base).join(path).exists()
    }
}

#[test]
fn test_mixed_fs() {
    let mut mfs = MixedFS::new();

    assert_eq!(mfs.exists(&"./1.txt".to_string(), &"./".to_string()), false);
    assert!(mfs.exists(&"config.toml".to_string(), &"./".to_string()));

    mfs.add(&"./1.txt".to_string());
    mfs.remove(&"config.toml".to_string());

    assert!(mfs.exists(&"1.txt".to_string(), &"./".to_string()));
    assert_eq!(
        mfs.exists(&"config.toml".to_string(), &"./".to_string()),
        false
    );
}
