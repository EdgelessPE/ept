use crate::types::StepLink;
use anyhow::{anyhow, Result};
use dirs::desktop_dir;
use mslnk::ShellLink;

pub fn step_link(step: StepLink,located:String) -> Result<i32> {
    // 获取用户桌面位置
    let desktop_opt = desktop_dir();
    if desktop_opt.is_none() {
        return Err(anyhow!("Error(Link):Can't get desktop location"));
    }
    let d = desktop_opt.unwrap();
    let desktop = d.to_str().unwrap_or(r"C:\Users\Public\Desktop");

    // 创建实例
    let sl_res = ShellLink::new(&step.source_file);
    if sl_res.is_err() {
        return Err(anyhow!(
            "Error(Link):Can't find source file '{}'",
            &step.source_file
        ));
    }

    // 创建快捷方式
    let target = format!("{}/{}.lnk", desktop, &step.target_name);
    let c_res = sl_res.unwrap().create_lnk(&target);
    if c_res.is_err() {
        return Err(anyhow!(
            "Error(Link):Can't create link {}->{} : {}",
            &step.source_file,
            &target,
            c_res.unwrap_err().to_string()
        ));
    }
    Ok(0)
}

#[test]
fn test_link() {
    step_link(StepLink {
        source_file: String::from(r"D:\CnoRPS\Beyond Compare 4\BCompare.exe"),
        target_name: String::from("Beyond Compare 4"),
    },String::from("D:/Desktop/Projects/EdgelessPE/ept/apps/VSCode"))
    .unwrap();
}
