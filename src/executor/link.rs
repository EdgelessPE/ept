use crate::types::StepLink;
use anyhow::{anyhow, Result};
use dirs::desktop_dir;
use mslnk::ShellLink;
use std::{path::Path, fs::canonicalize, env::current_dir};

pub fn step_link(step: StepLink, located: String) -> Result<i32> {
    // 获取用户桌面位置
    let desktop_opt = desktop_dir();
    if desktop_opt.is_none() {
        return Err(anyhow!("Error(Link):Can't get desktop location"));
    }
    let d = desktop_opt.unwrap();
    let desktop = d.to_str().unwrap_or(r"C:\Users\Public\Desktop");

    // 解析源文件绝对路径
    let abs_clear_source_path=current_dir()?.join(&located.replace("./", "")).join(&step.source_file.replace("./", ""));
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
    Ok(0)
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
