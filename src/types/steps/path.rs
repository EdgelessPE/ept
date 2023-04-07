use super::TStep;
use crate::types::mixed_fs::MixedFS;
use crate::types::package::GlobalPackage;
use crate::types::permissions::{Generalizable, Permission, PermissionLevel};
use crate::types::verifiable::Verifiable;
use crate::utils::{ask_yn, get_path_bin, parse_relative_path};
use crate::{log, p2s};
use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::fs::{create_dir, remove_file, File};
use std::io::Write;
use std::path::Path;
use std::ptr::null_mut;
use which::which;
use winapi::shared::minwindef::{LPARAM, WPARAM};
use winapi::um::winuser::{
    SendMessageTimeoutA, HWND_BROADCAST, SMTO_ABORTIFHUNG, WM_SETTINGCHANGE,
};
use winreg::{enums::*, RegKey};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct StepPath {
    pub record: String,
    pub alias: Option<String>,
}

fn conflict_resolver(bin_abs: &String, stem: &String, scope: &String) -> String {
    let origin = format!("{bin_abs}/{stem}.cmd");
    let scoped = format!("{bin_abs}/{scope}-{stem}.cmd");

    // 检查入口文件冲突
    if Path::new(&origin).exists() {
        log!("Warning(Path):Entrance '{stem}.cmd' already exists in '{bin_abs}', overwrite? (y/n)");
        if ask_yn() {
            return origin;
        } else {
            log!("Warning(Path):Renamed entrance to '{scope}-{stem}.cmd, use '{scope}-{stem}' instead to call this program later");
            return scoped;
        }
    }

    // 检查系统全局 PATH 冲突
    let which_res = which(stem);
    if let Ok(res) = which_res {
        let output = p2s!(res);
        log!("Warning(Path):Command '{stem}' already exists at '{output}', rename to '{scope}-{stem}'? (y/n)");
        if ask_yn() {
            log!("Warning(Path):Renamed entrance to '{scope}-{stem}.cmd, use '{scope}-{stem}' instead to call this program later");
            return scoped;
        } else {
            log!("Warning(Path):You may need to rename '{origin}' to access the newly installed program. If do so, don't run 'ept clean' since the renamed entrance would be cleaned");
            return origin;
        }
    }

    origin
}

// 配置系统 PATH 变量，但是需要注销并重新登录以生效
// 返回的 bool 表示是否执行了操作
fn set_system_path(record: &String, is_add: bool) -> Result<bool> {
    // 转换 record 为反斜杠
    let record = record.replace("/", r"\");
    let record_str = record.as_str();

    // 打开 HKEY_CURRENT_USER\Environment
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let table = hkcu
        .open_subkey("Environment")
        .map_err(|_| anyhow!("Error(Path):Can't open user register"))?;

    // 读取 Path 键值
    let origin_text: String = table.get_value("Path").unwrap_or(String::new());

    // 拆分 Path 为数组
    let mut origin_arr: Vec<&str> = origin_text.split(";").collect();

    // 检查给定的值是否已经存在
    let is_exist = origin_arr.contains(&record_str);

    // 增删 Path 变量
    if is_add {
        if is_exist {
            // log!("Warning(Path):Record '{record_str}' already existed in PATH");
            return Ok(false);
        } else {
            origin_arr.push(record_str);
        }
    } else {
        if is_exist {
            origin_arr = origin_arr
                .into_iter()
                .filter(|x| x != &record_str)
                .collect();
        } else {
            log!("Warning(Path):Record '{record_str}' not exist in PATH");
            return Ok(false);
        }
    }

    // 生成新字符串
    let new_arr: Vec<&str> = origin_arr
        .into_iter()
        .filter(|x| x.to_owned() != "")
        .collect();
    let new_text = new_arr.join(";");
    log!("Debug(Path):Save register with '{new_text}'");

    // 写回注册表
    let (table, _) = hkcu.create_subkey("Environment")?;
    table
        .set_value("Path", &new_text)
        .map_err(|err| anyhow!("Error(Path):Can't write to register : {err}"))?;

    // 发送全局广播
    let result = unsafe {
        SendMessageTimeoutA(
            HWND_BROADCAST,
            WM_SETTINGCHANGE,
            0 as WPARAM,
            "Environment\0".as_ptr() as LPARAM,
            SMTO_ABORTIFHUNG,
            3000,
            null_mut(),
        )
    };
    if result == 0 {
        log!("Warning(Path):Failed to apply PATH change, restart is required")
    }

    Ok(true)
}

impl TStep for StepPath {
    fn run(self, located: &String, pkg: &GlobalPackage) -> Result<i32> {
        // 解析 bin 绝对路径
        let bin_path = get_path_bin()?;
        let bin_abs = p2s!(bin_path);

        // 创建 bin 目录
        if !bin_path.exists() {
            create_dir(bin_path.to_owned())?;
        }

        // 添加系统 PATH 变量
        let add_res = set_system_path(&bin_abs, true);
        if add_res.is_err() {
            log!("Warning(Path):Failed to add system PATH for '{bin_abs}', manually add later to enable bin function of nep");
        } else if add_res.unwrap() {
            log!("Warning(Path):Added system PATH for '{bin_abs}'");
        }

        // 解析目标绝对路径
        let abs_target_path = parse_relative_path(&p2s!(Path::new(&located).join(&self.record)))?;
        let abs_target_str = p2s!(abs_target_path).replace("/", r"\");

        // 处理为目录的情况
        if abs_target_path.is_dir() {
            if self.alias.is_some() {
                log!(
                    "Warning(Path):Ignoring alias '{a}', since record refers to a dictionary",
                    a = self.alias.unwrap()
                );
            }
            let add_res = set_system_path(&abs_target_str, true);
            if add_res.is_err() {
                log!("Warning(Path):Failed to add system PATH '{bin_abs}', manually add later");
            } else if add_res.unwrap() {
                log!("Warning(Path):Added system PATH '{bin_abs}', restart to enable");
            }
            return Ok(0);
        }

        // 解析批处理路径
        let stem = self
            .alias
            .unwrap_or_else(|| p2s!(Path::new(&self.record).file_stem().unwrap()));
        let cmd_target_str =
            conflict_resolver(&bin_abs, &stem, &pkg.software.clone().unwrap().scope);
        if !abs_target_path.exists() {
            return Err(anyhow!(
                "Error(Path):Failed to add path : final target '{abs_target_str}' not exist"
            ));
        }

        // 写批处理
        let cmd_content = format!("@\"{abs_target_str}\" %*");
        let mut file = File::create(&cmd_target_str)?;
        file.write_all(cmd_content.as_bytes())?;
        log!("Info(Path):Added path entrance '{cmd_target_str}'");

        Ok(0)
    }
    fn reverse_run(self, located: &String, pkg: &GlobalPackage) -> Result<()> {
        // 解析 bin 绝对路径
        let bin_path = get_path_bin()?;
        let bin_abs = p2s!(bin_path);

        // 创建 bin 目录
        if !bin_path.exists() {
            create_dir(bin_path.to_owned())?;
        }

        // 解析目标绝对路径
        let abs_target_path = parse_relative_path(&p2s!(Path::new(&located).join(&self.record)))?;
        let abs_target_str = p2s!(abs_target_path).replace("/", r"\");

        // 处理为目录的情况
        if abs_target_path.is_dir() {
            let add_res = set_system_path(&abs_target_str, false);
            if add_res.is_err() {
                log!(
                    "Warning(Path):Failed to remove system PATH for '{bin_abs}', manually remove later"
                );
            } else if add_res.unwrap() {
                log!("Info(Path):Removed system PATH '{bin_abs}'");
            }
            return Ok(());
        }

        // 解析入口路径，根据优先级选择删除一个
        let stem = self
            .alias
            .unwrap_or_else(|| p2s!(Path::new(&self.record).file_stem().unwrap()));
        let scope = pkg.software.clone().unwrap().scope;
        let delete_list = vec![
            format!("{bin_abs}/{scope}-{stem}.cmd"),
            format!("{bin_abs}/{stem}.cmd"),
        ];
        for cmd_target_str in delete_list {
            let cmd_target_path = Path::new(&cmd_target_str);
            // 删除入口
            if cmd_target_path.exists() {
                remove_file(cmd_target_path)?;
                log!("Info(Path):Removed path entrance '{cmd_target_str}'");
                break;
            }
        }

        Ok(())
    }
    fn get_manifest(&self, _fs: &mut MixedFS) -> Vec<String> {
        vec![self.record.to_owned()]
    }
    fn interpret<F>(self, interpreter: F) -> Self
    where
        F: Fn(String) -> String,
    {
        Self {
            record: interpreter(self.record),
            alias: self.alias,
        }
    }
}

impl Verifiable for StepPath {
    fn verify_self(&self,_:&String) -> Result<()> {
        Ok(())
    }
}

impl Generalizable for StepPath {
    fn generalize_permissions(&self) -> Result<Vec<Permission>> {
        // 检查是否有拓展名，以此判断添加的是目录还是单文件
        let p = Path::new(&self.record);
        let node = if p.extension().is_some() {
            Permission {
                key: "path_entrances".to_string(),
                level: PermissionLevel::Normal,
                targets: vec![self.record.to_owned()],
            }
        } else {
            Permission {
                key: "path_dirs".to_string(),
                level: PermissionLevel::Important,
                targets: vec![self.record.to_owned()],
            }
        };
        Ok(vec![node])
    }
}

#[test]
fn test_set_system_path() {
    set_system_path(&"D:/CnoRPS/aria2".to_string(), false).unwrap();
}

#[test]
fn test_path() {
    envmnt::set("DEBUG", "true");
    let pkg = GlobalPackage::_demo();
    StepPath {
        record: String::from(r"C:\Users\dsyou\scoop\shims\rclone.exe"),
        // alias: Some("aria".to_string()),
        alias: None,
    }
    .run(&String::from("./apps/VSCode"), &pkg)
    .unwrap();
}
