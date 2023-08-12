use std::{collections::HashSet, path::Path};

use wildmatch::WildMatch;

use crate::{
    log, p2s,
    utils::{contains_wild_match, format_path, parse_wild_match},
};

pub struct MixedFS {
    located: String,

    // 不含通配符
    to_add: HashSet<String>,
    to_remove: HashSet<String>,

    // 含通配符的
    to_add_wild_match: HashSet<String>,
    to_remove_wild_match: HashSet<String>,

    // TEMP:是否被调用了会导致文件系统新增文件的步骤，若是则检测到白名单步骤时对 manifest 检测异常警告而非报错
    pub var_warn_manifest: bool,
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

// 拼接路径，解析出虚拟添加路径
fn merge_path(exact_from: &String, to: String) -> String {
    let file_name = p2s!(Path::new(exact_from).file_name().unwrap());

    to + &file_name
}

impl MixedFS {
    pub fn new(located: String) -> Self {
        log!("Debug:MixedFS instance created with located '{located}'");
        MixedFS {
            located,
            to_add: HashSet::new(),
            to_remove: HashSet::new(),
            to_add_wild_match: HashSet::new(),
            to_remove_wild_match: HashSet::new(),
            var_warn_manifest: false,
        }
    }

    fn a_add(&mut self, path: String) {
        log!("Debug:Add path '{path}' to mixed fs");
        self.to_remove.remove(&path);
        self.to_add.insert(path);
    }
    fn a_remove(&mut self, path: String) {
        log!("Debug:Remove path '{path}' to mixed fs");
        self.to_add.remove(&path);
        self.to_remove.insert(path);
    }
    fn a_add_wild_match(&mut self, path: String) {
        log!("Debug:Add wm '{path}' to mixed fs");
        self.to_remove_wild_match.remove(&path);
        self.to_add_wild_match.insert(path);
    }
    fn a_remove_wild_match(&mut self, path: String) {
        log!("Debug:Remove wm '{path}' to mixed fs");
        self.to_add_wild_match.remove(&path);
        self.to_remove_wild_match.insert(path);
    }

    pub fn add(&mut self, path: &String, from: &String) {
        debug_assert!(!contains_wild_match(path));
        if path.starts_with("${") {
            return;
        }

        // 配置 var_warn_manifest flag
        self.var_warn_manifest = true;

        // 特殊处理 New 的逻辑
        if from.is_empty() {
            let path = path.to_owned();
            if path.ends_with("/") {
                self.a_add_wild_match(path + "*");
                return;
            } else {
                self.a_add(path);
            }
            return;
        }
        let path = format_path(path);
        let from = format_path(from);

        // 检查 from 是否也为包内路径
        if !from.starts_with("${") {
            if contains_wild_match(&from) {
                // 直接根据真实文件系统拓展 from，拼接到 MixedFS 内
                for exact_path in parse_wild_match(from, &self.located).unwrap_or(Vec::new()) {
                    let exact_from = p2s!(exact_path);
                    let merged_path = merge_path(&exact_from, path.clone());
                    if exact_path.is_dir() {
                        self.a_add_wild_match(merged_path + "/*");
                    } else {
                        self.a_add(merged_path);
                    }
                }
            } else {
                self.a_add(path);
            }

            return;
        }

        // 两个之一为目录，直接添加通配记录
        if path.ends_with("/") || from.ends_with("/") {
            let to_insert = if path.ends_with("/") {
                path + "*"
            } else {
                path + "/*"
            };
            self.a_add_wild_match(to_insert);
            return;
        }

        // 兜底，无法从来源确定此路径是文件还是目录，则宽容地添加两条记录
        let with_wm_end = path.clone() + "/*";
        self.a_add_wild_match(with_wm_end);
        self.a_add(path);
    }
    pub fn remove(&mut self, path: &String) {
        if path.starts_with("${") {
            return;
        }
        let path = format_path(path);
        if contains_wild_match(&path) {
            for exact_path in parse_wild_match(path, &self.located).unwrap_or(Vec::new()) {
                let str = p2s!(exact_path);
                let str = &str[format_path(&self.located).len()..str.len()];
                self.a_remove(str.to_string());
            }
        } else {
            // 判断是否存在
            if !self.exists(&path) {
                log!("Warning(MixedFS):Trying to remove a non-existent target : '{path}'");
                return;
            }

            // 判断是文件夹还是文件
            if path.ends_with("/") || Path::new(&path).is_dir() {
                let path = if path.ends_with("/") {
                    path + "*"
                } else {
                    path + "/*"
                };
                self.a_remove_wild_match(path);
            } else {
                self.a_remove(path);
            }
        }
    }

    pub fn exists(&self, path: &String) -> bool {
        if path.starts_with("${") {
            return true;
        }
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

        Path::new(&self.located).join(path).exists()
    }
}

#[test]
fn test_mixed_fs() {
    envmnt::set("DEBUG", "true");
    let base = &"./".to_string();
    let mut mfs = MixedFS::new(base.clone());

    // 基础判断能力
    assert!(!mfs.exists(&"./1.txt".to_string()));
    assert!(mfs.exists(&"Cargo.toml".to_string()));

    // 增删指定文件
    mfs.add(&"./1.txt".to_string(), &"./backup/1.txt".to_string());
    mfs.remove(&"config.toml".to_string());

    assert!(mfs.exists(&"1.txt".to_string()));
    assert!(!mfs.exists(&"./backup/1.txt".to_string()));
    assert!(!mfs.exists(&"config.toml".to_string()));

    // 增删通配文件
    mfs.add(&"./c/".to_string(), &"./src/types/*.rs".to_string());
    mfs.remove(&"./src/main?rs".to_string());

    assert!(mfs.exists(&"c/mod.rs".to_string()));
    assert!(mfs.exists(&"src/types/mod.rs".to_string()));
    assert!(!mfs.exists(&"./src/main.rs".to_string()));

    // 指定文件与通配文件冲突
    mfs.remove(&"./c/mixed_fs.rs".to_string());

    assert!(mfs.exists(&"c/mod.rs".to_string()));
    assert!(!mfs.exists(&"c/mixed_fs.rs".to_string()));

    // 增删指定目录
    mfs.add(
        &"./233".to_string(),
        &"${AppData}/Edgeless/ept/".to_string(),
    );
    mfs.remove(&"target".to_string());

    assert!(mfs.exists(&"233/whats.ts".to_string()));
    assert!(!mfs.exists(&"./target/debug/ept.exe".to_string()));

    // 增删通配目录(暂不支持复杂操作)
    // mfs.add(&"./234/".to_string(), &"./src/util?".to_string());
    // assert!(mfs.exists(&"234/utils/exe_version.ts".to_string()));
    // mfs.remove(&"./23?".to_string());
    // assert!(!mfs.exists(&"234/utils/exe_version.ts".to_string()));
    // assert!(!mfs.exists(&"233".to_string()));
}
