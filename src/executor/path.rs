use crate::types::StepPath;
use crate::utils::{get_path_bin, log, parse_relative_path};
use anyhow::{anyhow, Result};
use std::fs::{create_dir, remove_file, File};
use std::io::Write;
use std::path::Path;
use winreg::{enums::*, RegKey};

// 配置系统 PATH 变量，但是需要注销并重新登录以生效
// 返回的 bool 表示是否执行了操作
fn set_system_path(step: StepPath, is_add: bool) -> Result<bool> {
    // 打开 HKEY_CURRENT_USER\Environment
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let table_res = hkcu.open_subkey("Environment");
    if table_res.is_err() {
        return Err(anyhow!("Error(Path):Can't open user register"));
    }
    let table = table_res.unwrap();

    // 读取 Path 键值
    let p_res = table.get_value("Path");
    if p_res.is_err() {
        return Err(anyhow!("Error(Path):Can't get 'Path' in register"));
    }

    // 拆分 Path 为数组
    let origin_text: String = p_res.unwrap();
    let mut origin_arr: Vec<&str> = origin_text.split(";").collect();

    // 检查给定的值是否已经存在
    let is_exist = origin_arr.contains(&step.record.as_str());

    // 增删 Path 变量
    let ns = &step.record.as_str();
    if is_add {
        if is_exist {
            // log(format!("Warning(Path):Record '{}' already existed in PATH",&step.record));
            return Ok(false);
        } else {
            origin_arr.push(ns);
        }
    } else {
        if is_exist {
            origin_arr = origin_arr
                .into_iter()
                .filter(|x| x != &step.record)
                .collect();
        } else {
            log(format!(
                "Warning(Path):Record '{}' not exist in PATH",
                &step.record
            ));
            return Ok(false);
        }
    }

    // 生成新字符串
    let new_arr: Vec<&str> = origin_arr
        .into_iter()
        .filter(|x| x.to_owned() != "")
        .collect();
    let new_text = new_arr.join(";");
    log(format!("Debug(Path):Save register with '{}'", &new_text));

    // 写回注册表
    let (table, _) = hkcu.create_subkey("Environment")?;
    let w_res = table.set_value("Path", &new_text);
    if w_res.is_err() {
        return Err(anyhow!(
            "Error(Path):Can't write to register : {}",
            w_res.unwrap_err().to_string()
        ));
    }

    Ok(true)
}

// 如果是文件，在当前目录创建 bin 文件夹并放置批处理，随后将 bin 添加到系统 PATH 变量；
// 如果是目录，直接添加到 PATH 变量并提醒用户重启
pub fn step_path(step: StepPath, located: String) -> Result<i32> {
    // 解析 bin 绝对路径
    let bin_path = get_path_bin();
    let bin_abs = bin_path.to_string_lossy().to_string();

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
        log(format!("Warning(Path):Failed to add system PATH for '{}', manually add later to enable bin function of nep",&bin_abs));
    } else if add_res.unwrap() {
        log(format!(
            "Warning(Path):Added system PATH for '{}', restart to enable bin function of nep",
            &bin_abs
        ));
    }

    // 解析目标绝对路径
    let abs_target_path = parse_relative_path(
        Path::new(&located)
            .join(&step.record)
            .to_string_lossy()
            .to_string(),
    )?;
    let abs_target_str = abs_target_path
        .to_string_lossy()
        .to_string()
        .replace("/", "\\");

    // 处理为目录的情况
    if abs_target_path.is_dir() {
        let add_res = set_system_path(
            StepPath {
                record: abs_target_str,
            },
            true,
        );
        if add_res.is_err() {
            log(format!(
                "Warning(Path):Failed to add system PATH '{}', manually add later",
                &bin_abs
            ));
        } else if add_res.unwrap() {
            log(format!(
                "Warning(Path):Added system PATH '{}', restart to enable",
                &bin_abs
            ));
        }
        return Ok(0);
    }

    // 解析批处理路径
    let stem = Path::new(&step.record)
        .file_stem()
        .unwrap()
        .to_string_lossy()
        .to_string();
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
    log(format!(
        "Info(Path):Added path entrance '{}'",
        cmd_target_str
    ));

    Ok(0)
}

pub fn reverse_path(step: StepPath, located: String) -> Result<()> {
    // 解析 bin 绝对路径
    let bin_path = get_path_bin();
    let bin_abs = bin_path.to_string_lossy().to_string();

    // 创建 bin 目录
    if !bin_path.exists() {
        create_dir(bin_path.to_owned())?;
    }

    // 解析目标绝对路径
    let abs_target_path = parse_relative_path(
        Path::new(&located)
            .join(&step.record)
            .to_string_lossy()
            .to_string(),
    )?;
    let abs_target_str = abs_target_path
        .to_string_lossy()
        .to_string()
        .replace("/", "\\");

    // 处理为目录的情况
    if abs_target_path.is_dir() {
        let add_res = set_system_path(
            StepPath {
                record: abs_target_str,
            },
            false,
        );
        if add_res.is_err() {
            log(format!(
                "Warning(Path):Failed to remove system PATH for '{}', manually remove later",
                &bin_abs
            ));
        } else if add_res.unwrap() {
            log(format!("Info(Path):Removed system PATH '{}'", &bin_abs));
        }
        return Ok(());
    }

    // 解析批处理路径
    let stem = Path::new(&step.record)
        .file_stem()
        .unwrap()
        .to_string_lossy()
        .to_string();
    let cmd_target_str = format!("{}/{}.cmd", &bin_abs, &stem);
    let cmd_target_path = Path::new(&cmd_target_str);

    // 删除批处理
    if cmd_target_path.exists() {
        remove_file(cmd_target_path)?;
        log(format!(
            "Info(Path):Removed path entrance '{}'",
            cmd_target_str
        ));
    }

    Ok(())
}

#[test]
fn test_path() {
    step_path(
        StepPath {
            record: String::from("./bin"),
        },
        String::from("./apps/VSCode"),
    )
    .unwrap();
    step_path(
        StepPath {
            record: String::from("./Code.exe"),
        },
        String::from("./apps/VSCode"),
    )
    .unwrap();
}
