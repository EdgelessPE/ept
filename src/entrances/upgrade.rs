use std::{
    fs::{write, File},
    process::{self, Command},
};

use crate::{
    log, p2s,
    utils::{
        allocate_path_temp,
        download::download,
        get_path_toolchain,
        term::ask_yn,
        upgrade::{check_has_upgrade, fmt_upgradable, fmt_upgradable_cross_wid_gap},
    },
};
use anyhow::{anyhow, Ok, Result};
use zip::ZipArchive;

// dry_run: 干运行，仅检查是否有更新
// need_exit_process: 仅当单测时传入 false，以此防止跑单测时进程退出
pub fn upgrade(dry_run: bool, need_exit_process: bool) -> Result<String> {
    let current_version = env!("CARGO_PKG_VERSION");
    // 检查是否有更新
    let (has_upgrade, is_cross_wid_gap, latest_release) = check_has_upgrade()?;
    if !has_upgrade || dry_run {
        return Ok(if has_upgrade {
            fmt_upgradable(latest_release)
        } else {
            format!("Success:The ept toolchain is up to date ('{current_version}')")
        });
    }
    if is_cross_wid_gap {
        return Err(anyhow!(
            "{}",
            fmt_upgradable_cross_wid_gap(false, latest_release)
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
    let temp_dir = allocate_path_temp("upgrade", false)?;
    let zip_path = temp_dir.join("latest.zip");
    let _ = download(&latest_release.url, zip_path.clone(), None)?;

    // 解压到临时目录
    let temp_release_dir = temp_dir.join("release");
    let file = File::open(&zip_path)
        .map_err(|e| anyhow!("Error:Failed to open '{}' as file : {e}", p2s!(zip_path)))?;
    let mut zip_ins = ZipArchive::new(file).map_err(|e| {
        anyhow!(
            "Error:Failed to open '{}' as zip file : {e}",
            p2s!(zip_path)
        )
    })?;
    zip_ins.extract(&temp_release_dir).map_err(|e| {
        anyhow!(
            "Error:Failed to extract zip file '{}' : {e}",
            p2s!(zip_path)
        )
    })?;
    if !temp_release_dir.join("ept.exe").exists() {
        return Err(anyhow!(
            "Error:Invalid zip file : Failed to find 'ept.exe' in '{}'",
            p2s!(temp_release_dir)
        ));
    }

    // 写 cmd 脚本
    let toolchain_path = get_path_toolchain()?;
    let script_path = temp_dir
        .join("upgrade.cmd")
        .to_string_lossy()
        .replace("/", "\\");
    let script_content = include_str!("../../scripts/toolchain_utils/upgrade.cmd")
        .to_string()
        .replace("{target}", toolchain_path.to_string_lossy().as_ref());
    write(&script_path, script_content)
        .map_err(|e| anyhow!("Error:Failed to write to '{}' : {e}", &script_path))?;

    // 执行脚本
    Command::new("cmd")
        .args(vec!["/c", "start", script_path.as_str()])
        .current_dir(temp_dir)
        .spawn()
        .map_err(|e| anyhow!("Error:Failed to execute command : {e}"))?;

    // 立即退出当前进程
    let tip = "Info:Waiting for external script to finish upgrading, current process exiting...";
    if need_exit_process {
        log!("{tip}");
        process::exit(0);
    } else {
        Ok(tip.to_string())
    }
}

#[test]
fn test_upgrade() {
    use crate::signature::blake3::compute_hash_blake3;
    use crate::utils::envmnt;
    use crate::utils::test::_run_mirror_mock_server;
    use std::fs::{copy, remove_dir_all, rename};
    use std::{thread::sleep, time::Duration};

    envmnt::set("CONFIRM", "true");
    envmnt::set("DEBUG", "true");
    crate::utils::test::_ensure_clear_test_dir();

    // 使用 mock 的镜像数据
    let mock_ctx = crate::utils::test::_use_mock_mirror_data();

    // 备份原工具链
    let toolchain_path = get_path_toolchain().unwrap();
    let bak_toolchain_path = toolchain_path.parent().unwrap().join("toolchain_bak");
    let has_origin_toolchain = toolchain_path.exists();
    if has_origin_toolchain {
        if bak_toolchain_path.exists() {
            remove_dir_all(&toolchain_path).unwrap();
        } else {
            rename(&toolchain_path, &bak_toolchain_path).unwrap();
        }
    }
    assert!(!toolchain_path.join("ept.exe").exists());

    // 启动 mock 服务器
    let _ = _run_mirror_mock_server();

    // 准备下载包并启动静态文件服务器
    copy(
        "examples/old_ept_toolchain/ept_9999.9999.9999.zip",
        "test/ept_9999.9999.9999.zip",
    )
    .unwrap();
    let (_, mut handler) = crate::utils::test::_run_static_file_server();

    // 运行 upgrade
    upgrade(true, false).unwrap();
    upgrade(false, false).unwrap();

    // 等待 3s 后断言程序被更新
    sleep(Duration::from_secs(3));
    assert!(toolchain_path.join("what can i say.man").exists());
    assert_eq!(
        compute_hash_blake3(&p2s!(toolchain_path.join("ept.exe"))).unwrap(),
        "47902cfe5ef75cae1b7cd0497b9b36f98847e55f1afb20d1799f99daf6c40ee4".to_string()
    );

    // 还原原有的镜像文件夹
    crate::utils::test::_restore_mirror_data(mock_ctx);
    // 还原原工具链
    if has_origin_toolchain {
        remove_dir_all(&toolchain_path).unwrap();
        rename(&bak_toolchain_path, &toolchain_path).unwrap();
    }
    handler.kill().unwrap();
}
