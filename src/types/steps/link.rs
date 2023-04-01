use super::TStep;
use crate::types::mixed_fs::MixedFS;
use crate::types::package::GlobalPackage;
use crate::types::permissions::{Generalizable, Permission, PermissionLevel};
use crate::types::steps::log;
use crate::utils::env::{env_desktop, env_start_menu};
use crate::utils::{count_sub_files, try_recycle};
use crate::{log, p2s, types::verifiable::Verifiable, utils::parse_relative_path};
use anyhow::{anyhow, Result};
use mslnk::ShellLink;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fs::create_dir_all;
use std::ptr::null_mut;
use std::{fs::remove_file, path::Path};
use winapi::shared::minwindef::{LPARAM, WPARAM};
use winapi::um::winuser::{
    SendMessageTimeoutA, HWND_BROADCAST, SMTO_ABORTIFHUNG, WM_SETTINGCHANGE,
};

lazy_static! {
    static ref TARGET_RE: Regex = Regex::new(r"^(([^/]+)/)?([^/]+)$").unwrap();
}

// 返回的第二参数表示是否创建了父目录
fn parse_target(name: &String, base: &String) -> Result<(String, bool)> {
    // 匹配 target_name 模式
    let sp: Vec<&str> = name.split("/").collect();
    let length = sp.len();
    let (lnk_folder_opt, lnk_name) = if length > 2 {
        return Err(anyhow!("Error:Invalid filed 'target_name' : '{name}'"));
    } else if length == 2 {
        (
            Some(sp.get(0).unwrap().to_string()),
            sp.get(1).unwrap().to_string(),
        )
    } else {
        (None, sp.get(0).unwrap().to_string())
    };

    // 解析目标位置
    let target = if let Some(lnk_folder) = lnk_folder_opt {
        let dir = Path::new(base).join(&lnk_folder);
        if !dir.exists() {
            create_dir_all(dir).map_err(|e| {
                anyhow!("Error(Link):Failed to create directory '{base}/{lnk_folder}' : {e}")
            })?;
        }
        (format!("{base}/{lnk_folder}/{lnk_name}.lnk"), true)
    } else {
        (format!("{base}/{lnk_name}.lnk"), false)
    };

    Ok(target)
}

fn create_shortcut(sl: &ShellLink, name: &String, base: &String) -> Result<()> {
    let (target, _) = parse_target(name, base)?;
    sl.create_lnk(&target)
        .map_err(|err| anyhow!("Error(Link):Can't create shortcut {target} : {err}"))?;
    log!("Info(Link):Added shortcut '{target}'");
    Ok(())
}

fn delete_shortcut(name: &String, base: &String) -> Result<()> {
    let (target, parent) = parse_target(name, base)?;
    try_recycle(Path::new(&target).to_path_buf())?;
    if parent {
        let parent_path = Path::new(&target).parent().unwrap();
        if count_sub_files(parent_path, |name| {
            name.ends_with(".lnk") || name.ends_with(".LNK")
        })? == 0
        {
            if let Err(e) = try_recycle(parent_path.to_path_buf()) {
                log!(
                    "Warning(Link):Failed to delete empty shortcut directory '{p}' : {e}",
                    p = p2s!(parent_path)
                );
            }
        }
    }
    log!("Info(Link):Removed shortcut '{target}'");
    Ok(())
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct StepLink {
    pub source_file: String,
    pub target_name: String,
    pub target_args: Option<String>,
    pub target_icon: Option<String>,
    pub at: Option<Vec<String>>,
}

impl TStep for StepLink {
    fn run(self, located: &String, _: &GlobalPackage) -> anyhow::Result<i32> {
        // 解析源文件绝对路径
        let abs_clear_source_path =
            parse_relative_path(&p2s!(Path::new(located).join(&self.source_file)))?;
        // println!("{abs_clear_source_path:?}");
        let abs_clear_source = p2s!(abs_clear_source_path);

        // 创建实例
        let mut sl = ShellLink::new(&abs_clear_source)
            .map_err(|_| anyhow!("Error(Link):Can't find source file '{abs_clear_source}'"))?;

        // 填充额外参数
        if self.target_icon.is_some() {
            sl.set_icon_location(self.target_icon);
        }
        if self.target_args.is_some() {
            sl.set_arguments(self.target_args);
        }

        // 分流
        let set: HashSet<String> =
            HashSet::from_iter(self.at.clone().unwrap_or(vec!["Desktop".to_string()]));
        if set.contains("Desktop") {
            create_shortcut(&sl, &self.target_name, &env_desktop())?;
        }
        if set.contains("StartMenu") {
            create_shortcut(&sl, &self.target_name, &env_start_menu())?;
            // 发送全局广播
            let result = unsafe {
                SendMessageTimeoutA(
                    HWND_BROADCAST,
                    WM_SETTINGCHANGE,
                    0 as WPARAM,
                    0 as LPARAM,
                    SMTO_ABORTIFHUNG,
                    3000,
                    null_mut(),
                )
            };
            if result == 0 {
                log!("Warning(Link):Failed to apply start menu change, restart is required")
            }
        }

        Ok(0)
    }
    fn reverse_run(self, _: &String, _: &GlobalPackage) -> Result<()> {
        let set: HashSet<String> =
            HashSet::from_iter(self.at.clone().unwrap_or(vec!["Desktop".to_string()]));
        if set.contains("Desktop") {
            delete_shortcut(&self.target_name, &env_desktop())?;
        }
        if set.contains("StartMenu") {
            delete_shortcut(&self.target_name, &env_start_menu())?;
            // 发送全局广播
            let result = unsafe {
                SendMessageTimeoutA(
                    HWND_BROADCAST,
                    WM_SETTINGCHANGE,
                    0 as WPARAM,
                    0 as LPARAM,
                    SMTO_ABORTIFHUNG,
                    3000,
                    null_mut(),
                )
            };
            if result == 0 {
                log!("Warning(Link):Failed to apply start menu change, restart is required")
            }
        }
        Ok(())
    }
    fn get_manifest(&self, _fs: &mut MixedFS) -> Vec<String> {
        vec![self.source_file.to_owned()]
    }
    fn interpret<F>(self, interpreter: F) -> Self
    where
        F: Fn(String) -> String,
    {
        Self {
            source_file: interpreter(self.source_file),
            target_name: interpreter(self.target_name),
            target_args: self.target_args.map(&interpreter),
            target_icon: self.target_icon.map(&interpreter),
            at: self.at,
        }
    }
}

impl Verifiable for StepLink {
    fn verify_self(&self) -> Result<()> {
        if !TARGET_RE.is_match(&self.target_name) {
            return Err(anyhow!(
                "Error(Link):Invalid field 'target_name', expect 'NAME' or 'FOLDER/NAME', got '{name}'",
                name=self.target_name
            ));
        }
        if self.target_name.contains("..") {
            return Err(anyhow!(
                "Error(Link):Invalid field 'target_name' : shouldn't contain '..'"
            ));
        }
        Ok(())
    }
}

impl Generalizable for StepLink {
    fn generalize_permissions(&self) -> Result<Vec<Permission>> {
        Ok(vec![Permission {
            key: "link_desktop".to_string(),
            level: PermissionLevel::Normal,
            targets: vec![self.target_name.to_owned()],
        }])
    }
}

#[test]
fn test_link() {
    let pkg = GlobalPackage::_demo();
    let step = StepLink {
        source_file: String::from("Code.exe"),
        target_name: String::from("VSC"),
        target_args: Some("--debug".to_string()),
        target_icon: Some(r"D:\Download\favicon.ico".to_string()),
        at: Some(vec!["Desktop".to_string()]),
    };
    step.verify_self().unwrap();
    step.run(&String::from("./apps/Microsoft/VSCode"), &pkg)
        .unwrap();
}
