use std::{
    fs::{write, File},
    process::{self, Command},
};

use crate::{
    log,
    utils::{
        allocate_path_temp,
        download::download,
        get_path_toolchain,
        term::ask_yn,
        upgrade::{check_has_upgrade, read_local_mirror_ept_toolchain},
    },
};
use anyhow::{anyhow, Ok, Result};
use zip::ZipArchive;

pub fn upgrade() -> Result<String> {
    // 读取工具链信息
    let toolchain_data = read_local_mirror_ept_toolchain()?;
    let current_version = env!("CARGO_PKG_VERSION");

    // 检查是否有更新
    let (has_upgrade, is_cross_wid_gap, latest_release) =
        check_has_upgrade(current_version, toolchain_data)?;
    if !has_upgrade {
        return Ok(format!(
            "Success:The current ept version ('{current_version}') is the latest"
        ));
    }
    if is_cross_wid_gap {
        return Err(anyhow!(
            "Error:The latest ept requires reinstall, visit 'https://ept.edgeless.top' to upgrade"
        ));
    }

    // 确认执行自更新
    if !ask_yn(
        format!(
            "Ready to upgrade ept toolchain from '{current_version}' to '{}', start now?",
            &latest_release.version
        ),
        true,
    ) {
        return Err(anyhow!("Error:Operation cancelled by user"));
    }

    // 下载最新的 zip 包，不带缓存
    let temp_dir = allocate_path_temp("download", false)?;
    let zip_path = temp_dir.join("latest.zip");
    let _ = download(&latest_release.url, zip_path.clone(), None)?;

    // 解压到临时目录
    let temp_release_dir = temp_dir.join("release");
    let file = File::open(&zip_path)
        .map_err(|e| anyhow!("Error:Failed to open '{zip_path:?}' as file : {e}"))?;
    let mut zip_ins = ZipArchive::new(file)
        .map_err(|e| anyhow!("Error:Failed to open '{zip_path:?}' as zip file : {e}"))?;
    zip_ins
        .extract(&temp_release_dir)
        .map_err(|e| anyhow!("Error:Failed to extract zip file '{zip_path:?}' : {e}"))?;
    if !temp_release_dir.join("ept.exe").exists() {
        return Err(anyhow!(
            "Error:Invalid zip file : Failed to find 'ept.exe' in '{temp_release_dir:?}'"
        ));
    }

    // 写 cmd 脚本
    let toolchain_path = get_path_toolchain()?;
    let script_path = temp_dir
        .join("upgrade.cmd")
        .to_string_lossy()
        .replace("/", "\\");
    let script_content = format!(
        "copy /y release\\* \"{}\"",
        toolchain_path.to_string_lossy()
    );
    write(&script_path, script_content)
        .map_err(|e| anyhow!("Error:Failed to write to '{script_path:?}' : {e}"))?;

    // 执行脚本
    Command::new("cmd")
        .args(vec!["/c", script_path.as_str()])
        .current_dir(temp_release_dir)
        .spawn()
        .map_err(|e| anyhow!("Error:Failed to execute command : {e}"))?;

    // 立即退出当前进程
    log!("Info:Waiting for external script to finish upgrading, current process exiting...");
    process::exit(0);
}
