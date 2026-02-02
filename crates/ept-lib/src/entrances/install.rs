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
use crate::types::constants::{DIR_NEP_CONTEXT, DIR_WORKFLOWS, WORKFLOW_SETUP};
use crate::{entrances::update_using_package, utils::parse_inputs::ParseInputResEnum};
use crate::{
    entrances::{expand_workshop, is_workshop_expandable},
    signature::blake3::compute_hash_blake3_from_string,
    types::package::GlobalPackage,
    utils::{
        cache::spawn_cache, download::download_nep, fs::move_or_copy, get_path_cache,
        path::parse_relative_path_with_located,
    },
};
use crate::{executor::workflow_executor, parsers::parse_workflow, utils::get_path_apps};
use crate::{log, log_ok_last, p2s};

// 检查软件是否已通过绝对路径的 main_program 字段全局安装
fn check_global_installation(
    cfg: &crate::types::context::RuntimeContext,
    package: &GlobalPackage,
) -> Result<bool> {
    if let Some(ref software) = package.software {
        if let Some(ref installed) = software.main_program {
            let p = Path::new(installed);
            if p.is_absolute() && p.exists() {
                return Ok(cfg.interaction().ask_yn(
                    &format!(
                        "Package '{name}' has been installed at '{installed}', continue?",
                        name = package.package.name
                    ),
                    false,
                ));
            }
        }
    }
    Ok(true)
}

// 检查包是否已安装，如果是则重定向到更新流程
fn check_existing_installation(
    cfg: &crate::types::context::RuntimeContext,
    source_file: &str,
    package: &GlobalPackage,
    verify_signature: bool,
) -> Result<Option<(String, String)>> {
    if let Ok((_, diff)) = info_local(cfg, &package.package.scope, &package.package.name) {
        log!(
            "Warning:Package '{name}' has been installed({ver}), switch to update entrance",
            name = package.package.name,
            ver = diff.version,
        );
        let res = update_using_package(cfg, source_file, verify_signature)?;
        return Ok(Some((res.scope, res.name)));
    }
    Ok(None)
}

// 将应用文件从临时目录部署到 apps 目录
fn deploy_app_files(
    cfg: &crate::types::context::RuntimeContext,
    temp_dir: &Path,
    package: &GlobalPackage,
) -> Result<String> {
    let into_dir = get_path_apps(cfg, &package.package.scope, &package.package.name, true)?;
    if into_dir.exists() {
        remove_dir_all(into_dir.clone()).map_err(|_| {
            anyhow!(
                "Error:Can't keep target directory '{dir}' clear, manually delete it then try again",
                dir = p2s!(into_dir.as_os_str())
            )
        })?;
    }

    let app_path = temp_dir.join(&package.package.name);
    if !app_path.exists() {
        return Err(anyhow!(
            "Error:App folder not found : {dir}",
            dir = p2s!(app_path)
        ));
    }
    move_or_copy(app_path, into_dir.clone())?;

    Ok(p2s!(into_dir))
}

// 验证指定的 main_program 是否存在
fn validate_main_program(
    cfg: &crate::types::context::RuntimeContext,
    into_dir: &str,
    package: &GlobalPackage,
) -> Result<()> {
    if let Some(ref software) = package.software {
        if let Some(ref installed) = software.main_program {
            let p = parse_relative_path_with_located(installed, into_dir);
            log!("Debug:Checking main program at '{}'", p2s!(p));
            if !p.exists() {
                if cfg.cfg.mode.qa {
                    log!("Warning:Validating failed : field 'main_program' provided in table 'software' not exist : '{installed}'")
                } else {
                    return Err(anyhow!("Error:Validating failed : field 'main_program' provided in table 'software' not exist : '{installed}'"));
                }
            }
        }
    }
    Ok(())
}

// 安装完成后的最终验证
fn finalize_installation(
    cfg: &crate::types::context::RuntimeContext,
    into_dir: &str,
    package: &GlobalPackage,
) -> Result<()> {
    installed_validator(into_dir)?;
    validate_main_program(cfg, into_dir, package)?;

    log!(
        "Debug:Try to get info of '{scope}/{name}'",
        scope = package.package.scope,
        name = package.package.name
    );
    info_local(cfg, &package.package.scope, &package.package.name).map_err(|e| {
        anyhow!(
            "Error:Validating failed : failed to get info of '{scope}/{name}' : {e}",
            scope = package.package.scope,
            name = package.package.name
        )
    })?;

    Ok(())
}

pub fn install_using_package(
    cfg: &crate::types::context::RuntimeContext,
    source_file: &str,
    verify_signature: bool,
) -> Result<(String, String)> {
    log!("Info:Preparing to install with package '{source_file}'");

    // 解包
    let (temp_dir_inner_path, package_struct) = unpack_nep(cfg, source_file, verify_signature)?;
    log!(
        "Info:If installation fails, use 'ept uninstall \"{name}\"' to roll back",
        name = package_struct.package.name
    );

    // 加载安装工作流
    log!("Info:Resolving package...");
    let setup_file_path = temp_dir_inner_path.join(DIR_WORKFLOWS).join(WORKFLOW_SETUP);
    let setup_workflow = parse_workflow(&p2s!(setup_file_path))?;

    // 检查是否已全局安装
    if !check_global_installation(cfg, &package_struct)? {
        return Err(anyhow!("Error:Operation canceled by user"));
    }

    // 检查是否已安装并重定向到更新
    if let Some(result) =
        check_existing_installation(cfg, source_file, &package_struct, verify_signature)?
    {
        return Ok(result);
    }
    log_ok_last!("Info:Resolving package...");

    // 如有展开工作流则执行
    let temp_dir_inner = p2s!(temp_dir_inner_path);
    if is_workshop_expandable(&temp_dir_inner) {
        expand_workshop(cfg, &temp_dir_inner)?;
    }

    // 部署文件
    log!("Info:Deploying files...");
    let into_dir = deploy_app_files(cfg, &temp_dir_inner_path, &package_struct)?;
    log_ok_last!("Info:Deploying files...");

    // 运行安装工作流
    log!("Info:Running setup workflow...");
    workflow_executor(
        cfg,
        setup_workflow,
        into_dir.clone(),
        package_struct.clone(),
    )?;
    log_ok_last!("Info:Running setup workflow...");

    // 保存 nep 上下文
    let ctx_path = Path::new(&into_dir).join(DIR_NEP_CONTEXT);
    move_or_copy(temp_dir_inner_path, ctx_path)?;

    // 验证安装
    log!("Info:Validating setup...");
    finalize_installation(cfg, &into_dir, &package_struct)?;
    log_ok_last!("Info:Validating setup...");

    // 清理
    clean_temp(cfg, source_file)?;

    Ok((
        package_struct.package.scope.clone(),
        package_struct.package.name.clone(),
    ))
}

pub fn install_using_url(
    cfg: &crate::types::context::RuntimeContext,
    url: &str,
    verify_signature: bool,
) -> Result<(String, String)> {
    // 下载文件到临时目录
    let cache_path = get_path_cache(cfg)?;
    let url_hash = compute_hash_blake3_from_string(url)?;
    let (p, cache_ctx) = download_nep(cfg, url, Some((cache_path, url_hash)))?;

    // 安装
    let info = install_using_package(cfg, &p2s!(p), verify_signature)?;

    // 缓存下载的包
    spawn_cache(cache_ctx)?;

    Ok(info)
}

pub fn install_using_parsed(
    cfg: &crate::types::context::RuntimeContext,
    parsed: Vec<ParseInputResEnum>,
    verify_signature: bool,
) -> Result<Vec<(String, String)>> {
    let mut arr = Vec::new();
    for parsed in parsed {
        log!("Info:Start installing {}", parsed.preview());
        let (scope, name) = match parsed {
            ParseInputResEnum::LocalPath(p, temp_dir) => {
                if let Some(temp_dir) = temp_dir {
                    install_using_package(cfg, &p2s!(temp_dir), false)?
                } else {
                    install_using_package(cfg, &p, verify_signature)?
                }
            }
            ParseInputResEnum::Url(u, temp_dir) => {
                if let Some(temp_dir) = temp_dir {
                    install_using_package(cfg, &p2s!(temp_dir), false)?
                } else {
                    install_using_url(cfg, &u, verify_signature)?
                }
            }
            ParseInputResEnum::PackageMatcher(p) => {
                install_using_url(cfg, &p.download_url, verify_signature)?
            }
        };
        log!("Success:Package '{scope}/{name}' installed successfully");
        arr.push((scope, name));
    }
    Ok(arr)
}

// #[test]
// fn test_install_using_url() {
//     install_using_url(
//         "http:/localhost:3000/api/redirect?path=/nep/Google/Chrome/Chrome_120.0.6099.200_Cno.nep"
//         false,
//     )
//     .unwrap();
// }

#[test]
fn test_install() {
    use crate::types::constants::FILE_PACKAGE;
    use crate::utils::fs::copy_dir;
    use crate::utils::test::_default_test_cfg;
    crate::utils::test::_ensure_clear_test_dir();

    let cfg = _default_test_cfg();

    // 校验路径
    let shortcut_path = dirs::desktop_dir().unwrap().join("Visual Studio Code.lnk");
    let entry1_path = crate::utils::get_path_bin(&cfg).unwrap().join("Code.cmd");
    let entry2_path = crate::utils::get_path_bin(&cfg)
        .unwrap()
        .join("Microsoft-Code.cmd");
    let app_path = get_path_apps(&cfg, "Microsoft", "VSCode", false).unwrap();
    let mp_path = app_path.join("Code.exe");
    let cx_path = app_path.join(DIR_NEP_CONTEXT).join(FILE_PACKAGE);

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
    if info_local(&cfg, "Microsoft", "VSCode").is_ok() {
        crate::uninstall(&cfg, Some("Microsoft".to_string()), "VSCode").unwrap();
    }

    // 打包并安装
    crate::pack(
        &cfg,
        "./examples/VSCode",
        Some("./test/VSCode_1.75.0.0_Cno (1).nep".to_string()),
        true,
    )
    .unwrap();
    install_using_package(&cfg, "./test/VSCode_1.75.0.0_Cno (1).nep", true).unwrap();

    assert!(shortcut_path.exists());
    assert!(entry1_path.exists() || entry2_path.exists());
    assert!(mp_path.exists());
    assert!(cx_path.exists());

    // 重复安装，会被要求使用升级，但是会由于同版本导致升级失败
    assert!(install_using_package(&cfg, "./test/VSCode_1.75.0.0_Cno (1).nep", true).is_err());

    crate::uninstall(&cfg, None, "VSCode").unwrap();

    assert!(!shortcut_path.exists());
    assert!(!entry1_path.exists() || entry2_path.exists());
    assert!(!mp_path.exists());
    assert!(!cx_path.exists());

    // 准备测试 main_program 校验
    crate::utils::test::_ensure_testing_uninstalled("Microsoft", "CallInstaller");
    let binding = crate::utils::env::env_desktop().unwrap() + "/Call.exe";
    let desktop_call_path = Path::new(&binding);
    if desktop_call_path.exists() {
        remove_file(desktop_call_path).unwrap();
    }

    // 安装 CallInstaller，预期会因为不存在主程序 ${Desktop}/Call.exe 而安装失败
    copy_dir("examples/CallInstaller", "test/CallInstaller1").unwrap();

    assert!(install_using_package(&cfg, "test/CallInstaller1", false).is_err());
    crate::clean(&cfg).unwrap();

    // 提供指定的主程序后安装成功
    std::fs::write(desktop_call_path, "114514").unwrap();
    crate::uninstall(&cfg, None, "CallInstaller").unwrap();
    copy_dir("examples/CallInstaller", "test/CallInstaller2").unwrap();
    install_using_package(&cfg, "test/CallInstaller2", false).unwrap();

    // 清理
    remove_file(desktop_call_path).unwrap();
    crate::uninstall(&cfg, None, "CallInstaller").unwrap();
}

#[test]
fn test_install_dism() {
    use crate::utils::arch::SysArch;
    use crate::utils::test::_default_test_cfg;
    use crate::utils::test::_ensure_testing_uninstalled;
    let cfg = _default_test_cfg();
    _ensure_testing_uninstalled("Chuyu", "Dism++");

    crate::utils::fs::copy_dir("examples/Dism++", "test/Dism++").unwrap();

    install_using_package(&cfg, "test/Dism++", false).unwrap();
    let stem_name = match SysArch::get_current_arch().unwrap() {
        SysArch::X64 => "Dism++x64",
        SysArch::X86 => "Dism++x86",
        SysArch::ARM64 => "Dism++ARM64",
    };
    let p = format!(
        "{d}/{stem_name}.lnk",
        d = crate::utils::env::env_desktop().unwrap()
    );
    println!("{p}");
    assert!(Path::new(&p).exists());
    std::fs::remove_file(&p).unwrap();
    crate::uninstall(&cfg, None, "Dism++").unwrap();
}

#[test]
fn test_reg_entry() {
    use crate::types::{context::WorkflowContext, steps::TStep};
    use crate::utils::flags::{set_flag, Flag};
    use crate::utils::test::_default_test_cfg;
    use winreg::enums::HKEY_CURRENT_USER;
    set_flag(Flag::Debug, true);
    let cur_dir_pb = std::env::current_dir().unwrap();
    let cur_dir = p2s!(cur_dir_pb);
    let flag_path = Path::new("_reg_entry_success.log");
    if flag_path.exists() {
        std::fs::remove_file(flag_path).unwrap();
    }
    let cfg = _default_test_cfg();

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
        ignore_exit_code: None,
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
    assert!(crate::entrances::verify::verify(&cfg, "examples/RegEntry").is_ok());

    // 安装
    use crate::utils::test::{_ensure_testing, _ensure_testing_uninstalled};
    _ensure_testing_uninstalled("Cno", "RegEntry");
    _ensure_testing("Cno", "RegEntry");

    // 确认版本号已经更新
    assert_eq!(
        crate::entrances::info(
            &cfg,
            crate::types::matcher::PackageInputEnum::PackageMatcher(
                crate::types::matcher::PackageMatcher {
                    scope: Some("Cno".to_string()),
                    name: "RegEntry".to_string(),
                    mirror: None,
                    version_req: None,
                }
            ),
            false
        )
        .unwrap()
        .0
        .local
        .unwrap()
        .version,
        "1.1.4.0".to_string()
    );

    // 执行卸载
    crate::entrances::uninstall(&cfg, None, "RegEntry").unwrap();

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
    use crate::utils::test::_default_test_cfg;
    // 替换测试镜像源
    let custom_mirror_ctx = crate::utils::test::_mount_custom_mirror();

    let cfg = _default_test_cfg();

    // 启动文件服务器
    let (_, mut handler) = crate::utils::test::_run_static_file_server();

    // 打包出一个 VSCode_1.75.4.2_Cno
    let static_path = Path::new("test/static");
    if !static_path.exists() {
        std::fs::create_dir_all(static_path).unwrap();
    }
    crate::pack(
        &cfg,
        "./examples/VSCode",
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
    let parsed =
        crate::utils::parse_inputs::parse_install_inputs(&cfg, vec!["vscode".to_string()], false)
            .unwrap();
    install_using_parsed(&cfg, parsed.into_iter().map(|p| p.0).collect(), false).unwrap();
    assert!(info_local(&cfg, "Microsoft", "VSCode").unwrap().1.version == *"1.75.4.0");

    // 使用大小写不敏感的别名直接安装
    crate::utils::test::_ensure_testing_vscode_uninstalled();
    let parsed =
        crate::utils::parse_inputs::parse_install_inputs(&cfg, vec!["CODE".to_string()], false)
            .unwrap();
    install_using_parsed(&cfg, parsed.into_iter().map(|p| p.0).collect(), false).unwrap();
    assert!(info_local(&cfg, "Microsoft", "VSCode").unwrap().1.version == *"1.75.4.0");

    // 手动升版本号
    let source_dir = crate::utils::test::_fork_example_with_version("examples/VSCode", "1.75.4.1");

    // 重新打包一个更高版本的
    crate::pack(
        &cfg,
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

    // 无法安装，会报错
    assert!(crate::entrances::update::update_using_package_matcher(
        &cfg,
        "microsoFT/vscode".to_string(),
        false
    )
    .is_err());

    crate::utils::test::_ensure_testing_vscode_uninstalled();

    // 关闭文件服务器
    handler.kill().unwrap();

    // 换回原镜像源
    crate::utils::test::_unmount_custom_mirror(custom_mirror_ctx);
}

#[test]
fn test_install_expandable() {
    use crate::utils::test::_default_test_cfg;
    crate::utils::test::_ensure_clear_test_dir();
    let cfg = _default_test_cfg();
    crate::utils::test::_ensure_testing_uninstalled("Microsoft", "VSCodeE");

    // 断言原来的包中不包含这个二进制文件
    assert!(!Path::new("examples/VSCodeE/VSCodeE/Code.exe").exists());

    // 启动文件服务器
    let (_addr, mut handler) = crate::utils::test::_run_static_file_server();
    std::fs::copy("examples/VSCode/VSCode/Code.exe", "test/Code.exe").unwrap();

    // 安装
    crate::utils::fs::copy_dir("examples/VSCodeE", "test/VSCodeE").unwrap();
    install_using_package(&cfg, "test/VSCodeE", false).unwrap();

    // 断言安装成功
    assert!(info_local(&cfg, "Microsoft", "VSCodeE").is_ok());
    assert!(get_path_apps(&cfg, "Microsoft", "VSCodeE", false)
        .unwrap()
        .join("Code.exe")
        .exists());

    crate::utils::test::_ensure_testing_uninstalled("Microsoft", "VSCodeE");
    handler.kill().unwrap();
}

#[test]
fn test_install_offline() {
    use crate::utils::test::_default_test_cfg;
    let cfg = _default_test_cfg();
    crate::utils::test::_ensure_testing_vscode_uninstalled();
    assert!(install_using_package(&cfg, "examples/vscode", true).is_err());
}
