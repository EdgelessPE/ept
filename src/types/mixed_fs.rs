use std::{collections::HashSet, path::Path};

use wildmatch::WildMatch;

use crate::utils::{contains_wild_match, format_path};

pub struct MixedFS {
    // 不含通配符
    to_add: HashSet<String>,
    to_remove: HashSet<String>,

    // 含通配符的
    to_add_wild_match: HashSet<String>,
    to_remove_wild_match: HashSet<String>,
}

// 输入的 path 来自 manifest，不会携带通配符
fn is_match_wild_match_set(path: &String, set: &HashSet<String>) -> bool {
    for wild_match_path in set.clone() {
        let wm = WildMatch::new(&wild_match_path);
        if wm.matches(&path) {
            return true;
        }
    }

    false
}

impl MixedFS {
    pub fn new() -> Self {
        MixedFS {
            to_add: HashSet::new(),
            to_remove: HashSet::new(),
            to_add_wild_match: HashSet::new(),
            to_remove_wild_match: HashSet::new(),
        }
    }
    pub fn add(&mut self, path: &String) {
        let path = format_path(path);
        if contains_wild_match(&path) {
            self.to_remove_wild_match.remove(&path);
            self.to_add_wild_match.insert(path);
        } else {
            self.to_remove.remove(&path);
            self.to_add.insert(path);
        }
    }
    pub fn remove(&mut self, path: &String) {
        let path = format_path(path);
        if contains_wild_match(&path) {
            self.to_add_wild_match.remove(&path);
            self.to_remove_wild_match.insert(path);
        } else {
            self.to_add.remove(&path);
            self.to_remove.insert(path);
        }
    }

    pub fn exists(&self, path: &String, base: &String) -> bool {
        let path = format_path(path);
        if self.to_add.contains(&path) {
            return true;
        }
        if !self.to_add_wild_match.is_empty()
            && is_match_wild_match_set(&path, &self.to_add_wild_match)
        {
            return true;
        }
        if self.to_remove.contains(&path) {
            return false;
        }
        if !self.to_remove_wild_match.is_empty()
            && is_match_wild_match_set(&path, &self.to_remove_wild_match)
        {
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
    let base=&"./".to_string();

    assert!(!mfs.exists(&"./1.txt".to_string(), base));
    assert!(mfs.exists(&"config.toml".to_string(), base));

    mfs.add(&"./1.txt".to_string());
    mfs.remove(&"config.toml".to_string());

    assert!(mfs.exists(&"1.txt".to_string(), base));
    assert!(!mfs.exists(&"config.toml".to_string(), base));

    mfs.add(&"./b/*.rs".to_string());
    mfs.remove(&"./a/main?rs".to_string());

    assert!(mfs.exists(&"b/mod.rs".to_string(), base));
    assert!(!mfs.exists(&"./a/main.rs".to_string(), base));
}
