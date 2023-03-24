use super::TStep;
use crate::types::Verifiable;
use crate::utils::{get_path_bin, parse_relative_path};
use crate::{log, p2s};
use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::fs::{create_dir, remove_file, File};
use std::io::Write;
use std::path::Path;
use std::ptr::null_mut;
use winapi::shared::minwindef::{LPARAM, WPARAM};
use winapi::um::winuser::{
    SendMessageTimeoutA, HWND_BROADCAST, SMTO_ABORTIFHUNG, WM_SETTINGCHANGE,
};
use winreg::{enums::*, RegKey};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct StepPath {
    pub record: String,
}

// 配置系统 PATH 变量，但是需要注销并重新登录以生效
// 返回的 bool 表示是否执行了操作
fn set_system_path(step: StepPath, is_add: bool) -> Result<bool> {
    // 转换 record 为反斜杠
    let record = step.record.replace("/", "\\");
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
            // log!("Warning(Path):Record '{}' already existed in PATH",record_str);
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
            log!("Warning(Path):Record '{}' not exist in PATH", record_str);
            return Ok(false);
        }
    }

    // 生成新字符串
    let new_arr: Vec<&str> = origin_arr
        .into_iter()
        .filter(|x| x.to_owned() != "")
        .collect();
    let new_text = new_arr.join(";");
    log!("Debug(Path):Save register with '{}'", &new_text);

    // 写回注册表
    let (table, _) = hkcu.create_subkey("Environment")?;
    table
        .set_value("Path", &new_text)
        .map_err(|err| anyhow!("Error(Path):Can't write to register : {}", err.to_string()))?;

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
    fn run(self, located: &String) -> Result<i32> {
        // 解析 bin 绝对路径
        let bin_path = get_path_bin()?;
        let bin_abs = p2s!(bin_path);

        // 创建 bin 目录
        if !bin_path.exists() {
            create_dir(bin_path.to_owned())?;
        }

        // 添加系统 PATH 变量
        let add_res = set_system_path(
            StepPath {
                record: bin_abs.clone(),
            },
            true,
        );
        if add_res.is_err() {
            log!("Warning(Path):Failed to add system PATH for '{}', manually add later to enable bin function of nep",&bin_abs);
        } else if add_res.unwrap() {
            log!("Warning(Path):Added system PATH for '{}'", &bin_abs);
        }

        // 解析目标绝对路径
        let abs_target_path = parse_relative_path(&p2s!(Path::new(&located).join(&self.record)))?;
        let abs_target_str = p2s!(abs_target_path).replace("/", "\\");

        // 处理为目录的情况
        if abs_target_path.is_dir() {
            let add_res = set_system_path(
                StepPath {
                    record: abs_target_str,
                },
                true,
            );
            if add_res.is_err() {
                log!(
                    "Warning(Path):Failed to add system PATH '{}', manually add later",
                    &bin_abs
                );
            } else if add_res.unwrap() {
                log!(
                    "Warning(Path):Added system PATH '{}', restart to enable",
                    &bin_abs
                );
            }
            return Ok(0);
        }

        // 解析批处理路径
        let stem = p2s!(Path::new(&self.record).file_stem().unwrap());
        let cmd_target_str = format!("{}/{}.cmd", &bin_abs, &stem);
        if !abs_target_path.exists() {
            return Err(anyhow!(
                "Error(Path):Failed to add path : final target '{}' not exist",
                &abs_target_str
            ));
        }

        // 写批处理
        let cmd_content = format!("@\"{}\" %*", &abs_target_str);
        let mut file = File::create(&cmd_target_str)?;
        file.write_all(cmd_content.as_bytes())?;
        log!("Info(Path):Added path entrance '{}'", cmd_target_str);

        Ok(0)
    }
    fn reverse_run(self, located: &String) -> Result<()> {
        // 解析 bin 绝对路径
        let bin_path = get_path_bin()?;
        let bin_abs = p2s!(bin_path);

        // 创建 bin 目录
        if !bin_path.exists() {
            create_dir(bin_path.to_owned())?;
        }

        // 解析目标绝对路径
        let abs_target_path = parse_relative_path(&p2s!(Path::new(&located).join(&self.record)))?;
        let abs_target_str = p2s!(abs_target_path).replace("/", "\\");

        // 处理为目录的情况
        if abs_target_path.is_dir() {
            let add_res = set_system_path(
                StepPath {
                    record: abs_target_str,
                },
                false,
            );
            if add_res.is_err() {
                log!(
                    "Warning(Path):Failed to remove system PATH for '{}', manually remove later",
                    &bin_abs
                );
            } else if add_res.unwrap() {
                log!("Info(Path):Removed system PATH '{}'", &bin_abs);
            }
            return Ok(());
        }

        // 解析批处理路径
        let stem = p2s!(Path::new(&self.record).file_stem().unwrap());
        let cmd_target_str = format!("{}/{}.cmd", &bin_abs, &stem);
        let cmd_target_path = Path::new(&cmd_target_str);

        // 删除批处理
        if cmd_target_path.exists() {
            remove_file(cmd_target_path)?;
            log!("Info(Path):Removed path entrance '{}'", cmd_target_str);
        }

        Ok(())
    }
    fn get_manifest(&self) -> Vec<String> {
        vec![self.record.to_owned()]
    }
    fn interpret<F>(self, interpreter: F) -> Self
    where
        F: Fn(String) -> String,
    {
        Self {
            record: interpreter(self.record),
        }
    }
}

impl Verifiable for StepPath {
    fn verify_self(&self) -> Result<()> {
        Ok(())
    }
}

#[test]
fn test_set_system_path() {
    set_system_path(
        StepPath {
            record: "D:/CnoRPS/aria2".to_string(),
        },
        false,
    )
    .unwrap();
}

#[test]
fn test_path() {
    StepPath {
        record: String::from("./bin"),
    }
    .run(&String::from("./apps/VSCode"))
    .unwrap();
}
