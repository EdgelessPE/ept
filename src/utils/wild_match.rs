use crate::p2s;
use anyhow::{anyhow, Result};
use std::fs::read_dir;
use std::path::{Path, PathBuf};
use wildmatch::WildMatch;

use super::parse_relative_path_with_located;

fn split_parent(raw: &String, located: &String) -> (PathBuf, String) {
    // 解析为绝对路径
    let abs_path = parse_relative_path_with_located(raw, located);

    // 拿到 parent
    let parent = abs_path
        .parent()
        .unwrap_or_else(|| Path::new(located))
        .to_path_buf();

    // 拿到 base name
    let base = p2s!(abs_path.file_name().unwrap());

    (parent, base)
}

pub fn contains_wild_match(raw: &String) -> bool {
    raw.contains("*") || raw.contains("?")
}

/// 返回 Ok(bool) 表示路径有效，bool 表示是否使用到了通配符；返回 Err(e) 表示使用方式非法
pub fn is_valid_wild_match(raw: &String, located: &String) -> Result<bool> {
    // 检查是否存在通配符
    if !contains_wild_match(raw) {
        return Ok(false);
    }

    // 拆分父子路径
    let (parent, _) = split_parent(raw, located);
    let parent = p2s!(parent);

    // 判断父路径是否存在通配符
    if contains_wild_match(&parent) {
        Err(anyhow!("Error:Invalid wild match usage in '{raw}' : wild match shouldn't appear in parent path '{parent}'"))
    } else {
        Ok(true)
    }
}

/// 将给定的带有通配符的路径解析为文件匹配数组
pub fn parse_wild_match(raw: String, located: &String) -> Result<Vec<PathBuf>> {
    // 拆分父子路径
    let (parent, child) = split_parent(&raw, located);

    // 判断父目录存在
    if !parent.exists() {
        return Err(anyhow!(
            "Error:Parent directory '{p}' doesn't exist",
            p = p2s!(parent)
        ));
    }

    // 创建 WildMatch 实例
    let wm = WildMatch::new(&child);

    // 读取父目录
    let res = read_dir(&parent)
        .map_err(|e| {
            anyhow!(
                "Error:Can't read '{p}' as directory : {e}",
                p = p2s!(parent)
            )
        })?
        .into_iter()
        .filter_map(|entry_res| {
            if let Ok(entry) = entry_res {
                let file_name = p2s!(entry.file_name());
                if wm.matches(&file_name) {
                    Some(parent.join(&file_name))
                } else {
                    None
                }
            } else {
                log!(
                    "Debug:Failed to get entry : {e}",
                    e = entry_res.unwrap_err()
                );
                None
            }
        })
        .collect();

    Ok(res)
}

/// 支持通配符步骤的通用校验函数
pub fn common_wild_match_verify(from: &String, to: &String, located: &String) -> Result<()> {
    is_valid_wild_match(from, located)?;
    if contains_wild_match(to) {
        return Err(anyhow!(
            "Error:Field 'to' shouldn't contain wild match : '{to}'"
        ));
    }
    if contains_wild_match(from) && !to.ends_with("/") {
        return Err(anyhow!(
            "Error:Field 'to' should end with '/' when field 'from' contains wild match"
        ));
    }

    Ok(())
}

/// 将 from 中的通配符合并到 to（然后提交到 MixedFS），需要保证输入已经过 common_wild_match_verify 的校验
pub fn common_merge_wild_match(from: &String, to: &String) -> String {
    if contains_wild_match(from) {
        let (_, item) = split_parent(from, &"".to_string());
        to.to_owned() + &item
    } else {
        to.clone()
    }
}

#[test]
fn test_is_valid_wild_match() {
    let located = String::from("D:/Desktop/Projects/EdgelessPE/ept");
    assert!(is_valid_wild_match(&"*.toml".to_string(), &located).is_ok());
    assert!(is_valid_wild_match(&"src/*.rs".to_string(), &located).is_ok());
    assert!(is_valid_wild_match(&"src/*s/mod.rs".to_string(), &located).is_err());
    assert!(is_valid_wild_match(&"src/types/mod?rs".to_string(), &located).is_ok());
}

#[test]
fn test_parse_wild_match() {
    let located = String::from("D:/Desktop/Projects/EdgelessPE/ept");
    println!(
        "{res:#?}",
        res = parse_wild_match("*.toml".to_string(), &located).unwrap()
    );
    println!(
        "{res:#?}",
        res = parse_wild_match("src/*.rs".to_string(), &located).unwrap()
    );
    println!(
        "{res:#?}",
        res = parse_wild_match("src/types/mod?rs".to_string(), &located).unwrap()
    );
    assert!(parse_wild_match("src/*s/mod.rs".to_string(), &located).is_err());
}

#[test]
fn test_common_merge_wild_match() {
    let located = String::from("D:/Desktop/Projects/EdgelessPE/ept");
    assert_eq!(
        common_merge_wild_match(&"src/types/*.rs".to_string(), &"src/utils/".to_string()).as_str(),
        "src/utils/*.rs"
    );
}
