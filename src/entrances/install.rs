use anyhow::{anyhow, Result};
use std::fs::remove_dir_all;
use std::path::Path;

use super::{
    info_local,
    utils::{
        package::{clean_temp, unpack_nep},
        validator::installed_validator,
    },
};
use crate::{
    entrances::{info, update_using_package, update_using_package_matcher},
    utils::{mirror::get_url_with_version_req, path::find_scope_with_name},
};
use crate::{executor::workflow_executor, parsers::parse_workflow, utils::get_path_apps};
use crate::{log, log_ok_last, p2s};
use crate::{
    types::matcher::PackageMatcher,
    utils::{
        download::download_nep, fs::move_or_copy, is_qa_mode,
        path::parse_relative_path_with_located, term::ask_yn,
    },
};

pub fn install_using_package(
    source_file: &String,
    verify_signature: bool,
) -> Result<(String, String)> {
    log!("Info:Preparing to install with package '{source_file}'");

    // 解包
    let (temp_dir_inner_path, package_struct) = unpack_nep(source_file, verify_signature)?;
    log!(
        "Info:If installation fails, use 'ept uninstall \"{name}\"' to roll back",
        name = &package_struct.package.name
    );

    // 读入安装工作流
    log!("Info:Resolving package...");
    let setup_file_path = temp_dir_inner_path.join("workflows/setup.toml");
    let setup_workflow = parse_workflow(&p2s!(setup_file_path))?;
    let package = package_struct.package.clone();
    let software = package_struct.software.clone().unwrap();
    log_ok_last!("Info:Resolving package...");

    // 使用绝对路径的 main_program 字段，检查是否已经全局安装过该软件
    if let Some(installed) = &software.main_program {
        let p = Path::new(installed);
        if p.is_absolute()
            && p.exists()
            && !ask_yn(
                format!(
                    "Package '{name}' has been installed at '{installed}', continue?",
                    name = package.name
                ),
                false,
            )
        {
            return Err(anyhow!("Error:Operation canceled by user"));
        }
    }

    log!("Info:Deploying files...");
    // 检查对应包名有没有被安装过
    if let Ok((_, diff)) = info_local(&software.scope, &package.name) {
        log!(
            "Warning:Package '{name}' has been installed({ver}), switch to update entrance",
            name = package.name,
            ver = diff.version,
        );
        let res = update_using_package(source_file, verify_signature)?;
        return Ok((res.scope, res.name));
    }

    // 解析最终安装位置
    let into_dir = get_path_apps(&software.scope, &package.name, true)?;
    if into_dir.exists() {
        remove_dir_all(into_dir.clone()).map_err(|_| {
            anyhow!(
                "Error:Can't keep target directory '{dir}' clear, manually delete it then try again",
                dir = p2s!(into_dir.as_os_str())
            )
        })?;
    }

    // 移动程序至 apps 目录
    let app_path = temp_dir_inner_path.join(&package.name);
    if !app_path.exists() {
        return Err(anyhow!(
            "Error:App folder not found : {dir}",
            dir = p2s!(app_path)
        ));
    }
    move_or_copy(app_path.clone(), into_dir.clone())?;
    log_ok_last!("Info:Deploying files...");

    // 执行安装工作流
    let into_dir = p2s!(into_dir);
    log!("Info:Running setup workflow...");
    workflow_executor(setup_workflow, into_dir.clone(), package_struct)?;
    log_ok_last!("Info:Running setup workflow...");

    // 保存上下文
    let ctx_path = Path::new(&into_dir).join(".nep_context");
    move_or_copy(temp_dir_inner_path, ctx_path)?;

    // 检查安装是否完整
    log!("Info:Validating setup...");
    installed_validator(&into_dir)?;
    // 如果提供了主程序检查是否存在
    if let Some(installed) = &software.main_program {
        let p = parse_relative_path_with_located(installed, &into_dir);
        if !p.exists() {
            if is_qa_mode() {
                log!("Warning:Validating failed : field 'main_program' provided in table 'software' not exist : '{installed}'")
            } else {
                return Err(anyhow!("Error:Validating failed : field 'main_program' provided in table 'software' not exist : '{installed}'"));
            }
        }
    }
    // 执行一次 info
    info(Some(software.scope.clone()), &package.name).map_err(|e| {
        anyhow!(
            "Error:Validating failed : failed to get info of '{scope}/{name}' : {e}",
            scope = software.scope,
            name = package.name
        )
    })?;
    log_ok_last!("Info:Validating setup...");

    // 清理临时文件夹
    clean_temp(source_file)?;

    Ok((software.scope, package.name))
}

pub fn install_using_url(url: &str, verify_signature: bool) -> Result<(String, String)> {
    // 下载文件到临时目录
    let p = download_nep(url)?;

    // 安装
    install_using_package(&p2s!(p), verify_signature)
}

pub fn install_using_package_matcher(
    matcher: PackageMatcher,
    verify_signature: bool,
) -> Result<(String, String)> {
    // 查找 scope 并使用 scope 更新纠正大小写
    let (scope, package_name) = find_scope_with_name(&matcher.name, matcher.scope.clone())?;
    // 检查对应包名有没有被安装过
    if let Ok((_, diff)) = info_local(&scope, &package_name) {
        log!(
            "Warning:Package '{scope}/{package_name}' has been installed({ver}), will be switched to update entrance",
            ver = diff.version,
        );
        let res = update_using_package_matcher(matcher, verify_signature)?;
        return Ok((res.scope, res.name));
    }
    // 解析 url
    let (url, target_release) = get_url_with_version_req(matcher)?;
    // 执行安装
    if ask_yn(
        format!(
            "Ready to install '{scope}/{package_name}' ({v}), continue?",
            v = target_release.version
        ),
        true,
    ) {
        install_using_url(&url, verify_signature)
    } else {
        Err(anyhow!("Error:Operation canceled by user"))
    }
}

// #[test]
// fn test_install_using_url() {
//     install_using_url(
//         &"http:/localhost:3000/api/redirect?path=/nep/Google/Chrome/Chrome_120.0.6099.200_Cno.nep"
//             .to_string(),
//         false,
//     )
//     .unwrap();
// }

#[test]
fn test_install() {
    use crate::utils::fs::copy_dir;
    envmnt::set("DEBUG", "true");
    envmnt::set("CONFIRM", "true");
    crate::utils::test::_ensure_clear_test_dir();

    // 校验路径
    let shortcut_path = dirs::desktop_dir().unwrap().join("Visual Studio Code.lnk");
    let entry1_path = crate::utils::get_path_bin().unwrap().join("Code.cmd");
    let entry2_path = crate::utils::get_path_bin()
        .unwrap()
        .join("Microsoft-Code.cmd");
    let app_path = get_path_apps(&"Microsoft".to_string(), &"VSCode".to_string(), false).unwrap();
    let mp_path = app_path.join("Code.exe");
    let cx_path = app_path.join(".nep_context").join("package.toml");

    use std::fs::remove_file;
    if shortcut_path.exists() {
        remove_file(&shortcut_path).unwrap();
    }
    if entry1_path.exists() {
        remove_file(&entry1_path).unwrap();
    }
    if entry2_path.exists() {
        remove_file(&entry2_path).unwrap();
    }

    // 卸载
    if info_local(&"Microsoft".to_string(), &"VSCode".to_string()).is_ok() {
        crate::uninstall(Some("Microsoft".to_string()), &"VSCode".to_string()).unwrap();
    }

    // 打包并安装
    crate::pack(
        &"./examples/VSCode".to_string(),
        Some("./test/VSCode_1.75.0.0_Cno (1).nep".to_string()),
        true,
    )
    .unwrap();
    install_using_package(&"./test/VSCode_1.75.0.0_Cno (1).nep".to_string(), true).unwrap();

    assert!(shortcut_path.exists());
    assert!(entry1_path.exists() || entry2_path.exists());
    assert!(mp_path.exists());
    assert!(cx_path.exists());

    // 重复安装，会被要求使用升级，但是会由于同版本导致升级失败
    assert!(
        install_using_package(&"./test/VSCode_1.75.0.0_Cno (1).nep".to_string(), true).is_err()
    );

    crate::uninstall(None, &"VSCode".to_string()).unwrap();

    assert!(!shortcut_path.exists());
    assert!(!entry1_path.exists() || entry2_path.exists());
    assert!(!mp_path.exists());
    assert!(!cx_path.exists());

    // 准备测试 main_program 校验
    crate::utils::test::_ensure_testing_uninstalled("Microsoft", "CallInstaller");
    let binding = crate::utils::env::env_desktop() + "/Call.exe";
    let desktop_call_path = Path::new(&binding);
    if desktop_call_path.exists() {
        remove_file(desktop_call_path).unwrap();
    }

    // 安装 CallInstaller，预期会因为不存在主程序 ${Desktop}/Call.exe 而安装失败
    copy_dir("examples/CallInstaller", "test/CallInstaller1").unwrap();

    assert!(install_using_package(&"test/CallInstaller1".to_string(), false).is_err());
    crate::clean().unwrap();

    // 提供指定的主程序后安装成功
    std::fs::write(desktop_call_path, "114514").unwrap();
    crate::uninstall(None, &"CallInstaller".to_string()).unwrap();
    copy_dir("examples/CallInstaller", "test/CallInstaller2").unwrap();
    install_using_package(&"test/CallInstaller2".to_string(), false).unwrap();

    // 清理
    remove_file(desktop_call_path).unwrap();
    crate::uninstall(None, &"CallInstaller".to_string()).unwrap();
}

#[test]
fn test_install_dism() {
    use crate::utils::arch::{get_arch, SysArch};
    use crate::utils::test::_ensure_testing_uninstalled;
    _ensure_testing_uninstalled("Chuyu", "Dism++");

    crate::utils::fs::copy_dir("examples/Dism++", "test/Dism++").unwrap();

    install_using_package(&"test/Dism++".to_string(), false).unwrap();
    let stem_name = match get_arch().unwrap() {
        SysArch::X64 => "Dism++x64",
        SysArch::X86 => "Dism++x86",
        SysArch::ARM64 => "Dism++ARM64",
    };
    let p = format!("{d}/{stem_name}.lnk", d = crate::utils::env::env_desktop());
    println!("{p}");
    assert!(Path::new(&p).exists());
    std::fs::remove_file(&p).unwrap();
}

#[test]
fn test_reg_entry() {
    use crate::types::{steps::TStep, workflow::WorkflowContext};
    use winreg::enums::HKEY_CURRENT_USER;
    envmnt::set("DEBUG", "true");
    let cur_dir_pb = std::env::current_dir().unwrap();
    let cur_dir = p2s!(cur_dir_pb);
    let flag_path = Path::new("_reg_entry_success.log");
    if flag_path.exists() {
        std::fs::remove_file(flag_path).unwrap();
    }

    // 替换并写注册表
    let raw_reg_text = std::fs::read_to_string("examples/RegEntry/RegEntry/add_reg.reg").unwrap();
    let replaced = raw_reg_text.replace("${CUR_DIR}", &cur_dir.replace('\\', r"\\"));
    let write_path = cur_dir_pb.join("test/_add_reg_entry.reg");
    std::fs::write(&write_path, replaced).unwrap();

    let mut cx = WorkflowContext::_demo();
    crate::types::steps::StepExecute {
        command: format!("reg import \"{r}\"", r = p2s!(write_path)),
        pwd: None,
        call_installer: None,
        wait: None,
    }
    .run(&mut cx)
    .unwrap();

    // 替换卸载命令
    let scene_text = std::fs::read_to_string("examples/RegEntry/RegEntry/uninstall.cmd").unwrap();
    let replaced_uninstall_text = scene_text.replace("${CUR_DIR}", &cur_dir);
    std::fs::write(
        "examples/RegEntry/RegEntry/uninstall.cmd",
        replaced_uninstall_text,
    )
    .unwrap();

    // 校验
    assert!(crate::entrances::verify::verify(&"examples/RegEntry".to_string()).is_ok());

    // 安装
    use crate::utils::test::{_ensure_testing, _ensure_testing_uninstalled};
    _ensure_testing_uninstalled("Cno", "RegEntry");
    _ensure_testing("Cno", "RegEntry");

    // 确认版本号已经更新
    assert_eq!(
        crate::entrances::info(Some("Cno".to_string()), &"RegEntry".to_string())
            .unwrap()
            .local
            .unwrap()
            .version,
        "1.1.4.0".to_string()
    );

    // 执行卸载
    crate::entrances::uninstall(None, &"RegEntry".to_string()).unwrap();

    // 断言 flag 的存在
    assert!(flag_path.exists());

    // 清理和恢复现场
    std::fs::remove_file(flag_path).unwrap();
    std::fs::write("examples/RegEntry/RegEntry/uninstall.cmd", scene_text).unwrap();
    let reg_root = winreg::RegKey::predef(HKEY_CURRENT_USER);
    let node = reg_root
        .open_subkey(r"Software\Microsoft\Windows\CurrentVersion\Uninstall")
        .unwrap();
    node.delete_subkey("_RegEntry").unwrap();
}

#[test]
fn test_install_with_matcher() {
    use semver::VersionReq;
    envmnt::set("CONFIRM", "true");
    // 替换测试镜像源
    let custom_mirror_ctx = crate::utils::test::_mount_custom_mirror();

    // 启动文件服务器
    let (_, mut handler) = crate::utils::test::_run_static_file_server();

    // 打包出一个 VSCode_1.75.4.2_Cno
    let static_path = Path::new("test/static");
    if !static_path.exists() {
        std::fs::create_dir_all(static_path).unwrap();
    }
    crate::pack(
        &"./examples/VSCode".to_string(),
        Some(
            static_path
                .join("VSCode_1.75.4.2_Cno.nep")
                .to_string_lossy()
                .to_string(),
        ),
        true,
    )
    .unwrap();

    // 执行安装
    crate::utils::test::_ensure_testing_vscode_uninstalled();
    install_using_package_matcher(
        PackageMatcher {
            name: "vscode".to_string(),
            scope: None,
            mirror: None,
            version_req: None,
        },
        false,
    )
    .unwrap();
    assert!(
        info_local(&"Microsoft".to_string(), &"VSCode".to_string())
            .unwrap()
            .1
            .version
            == *"1.75.4.0"
    );

    // 手动升版本号
    let source_dir = crate::utils::test::_fork_example_with_version("examples/VSCode", "1.75.4.1");

    // 重新打包一个更高版本的
    crate::pack(
        &source_dir,
        Some(
            static_path
                .join("VSCode_1.75.4.2_Cno.nep")
                .to_string_lossy()
                .to_string(),
        ),
        true,
    )
    .unwrap();

    // 执行更新
    crate::entrances::update_using_package_matcher(
        PackageMatcher {
            name: "vscode".to_string(),
            scope: Some("microsoFT".to_string()),
            mirror: Some("mock-server".to_string()),
            version_req: Some(VersionReq::parse("^1.75").unwrap()),
        },
        false,
    )
    .unwrap();

    crate::utils::test::_ensure_testing_vscode_uninstalled();

    // 关闭文件服务器
    handler.kill().unwrap();

    // 换回原镜像源
    crate::utils::test::_unmount_custom_mirror(custom_mirror_ctx);
}
