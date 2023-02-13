use crate::{types::StepLink, utils::parse_relative_path};
use anyhow::{anyhow, Result};
use dirs::desktop_dir;
use mslnk::ShellLink;
use std::{fs::remove_file, path::Path};

use super::log;

fn get_desktop() -> Result<String> {
    let desktop_opt = desktop_dir();
    if desktop_opt.is_none() {
        return Err(anyhow!("Error(Link):Can't get desktop location"));
    }
    let d = desktop_opt.unwrap();
    let desktop = d.to_str().unwrap_or(r"C:\Users\Public\Desktop");
    Ok(String::from(desktop))
}

pub fn step_link(step: StepLink, located: String) -> Result<i32> {
    // 获取用户桌面位置
    let desktop = get_desktop()?;

    // 解析源文件绝对路径
    let abs_clear_source_path = parse_relative_path(
        Path::new(&located)
            .join(&step.source_file)
            .to_string_lossy()
            .to_string(),
    )?;
    // println!("{:?}",&abs_clear_source_path);
    let abs_clear_source = abs_clear_source_path.to_string_lossy().to_string();

    // 创建实例
    let sl_res = ShellLink::new(&abs_clear_source);
    if sl_res.is_err() {
        return Err(anyhow!(
            "Error(Link):Can't find source file '{}'",
            &abs_clear_source
        ));
    }

    // 创建快捷方式
    let target = format!("{}/{}.lnk", desktop, &step.target_name);
    let c_res = sl_res.unwrap().create_lnk(&target);
    if c_res.is_err() {
        return Err(anyhow!(
            "Error(Link):Can't create link {}->{} : {}",
            &abs_clear_source,
            &target,
            c_res.unwrap_err().to_string()
        ));
    }
    log(format!("Info(Link):Added shortcut '{}'", target));
    Ok(0)
}

pub fn reverse_link(step: StepLink, _: String) -> Result<()> {
    // 获取用户桌面位置
    let desktop = get_desktop()?;

    // 解析快捷方式路径
    let target = format!("{}/{}.lnk", desktop, &step.target_name);

    // 尝试删除
    let target_path = Path::new(&target);
    if target_path.exists() {
        remove_file(target_path)?;
        log(format!("Info(Link):Removed shortcut '{}'", target));
    }
    Ok(())
}

pub fn manifest_link(step: StepLink)->Vec<String>{
    vec![step.source_file]
}

#[test]
fn test_link() {
    step_link(
        StepLink {
            source_file: String::from("./Code.exe"),
            target_name: String::from("VSC"),
        },
        String::from("./apps/VSCode"),
    )
    .unwrap();
}
