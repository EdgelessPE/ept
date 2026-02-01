use anyhow::{anyhow, Result};
use clap::Parser;
use ept_lib::{
    entrances::{
        auto_mirror_update_all, clean,
        config::{config_get, config_init, config_list, config_set, config_which},
        info, install_using_parsed, list, meta, mirror_add, mirror_list, mirror_remove,
        mirror_update, mirror_update_all, pack, search, uninstall, update_all, update_using_parsed,
        upgrade,
    },
    types::{
        cfg::Cfg,
        cli::{Action, ActionConfig, ActionMirror, Args},
        matcher::PackageInputEnum,
    },
    utils::{
        flags::{set_flag, Flag},
        fmt_print::{fmt_print_mirror_line, FmtPrint, FmtPrintCaller, PackageSource},
        get_path_apps, launch_clean,
        parse_inputs::{parse_install_inputs, parse_uninstall_inputs, parse_update_inputs},
        term::ask_yn,
        term::write_windows_terminal_status,
        upgrade::{check_has_upgrade, print_upgradable, print_upgradable_cross_wid_gap},
    },
};
use ept_lib::{log, p2s};
use std::fs::write;
use std::process::exit;

#[cfg(not(tarpaulin_include))]
fn router(action: Action, cfg: &Cfg) -> Result<String> {
    let verify_signature = !cfg.mode.offline;

    // 匹配入口
    match action {
        Action::Install {
            package_matchers: packages,
        } => {
            // 解析输入
            let parsed = parse_install_inputs(cfg, packages, verify_signature)?;
            log!("Debug:Parsed install packages: {parsed:?}");
            if parsed.is_empty() {
                return Ok(
                    "Warning:All packages have been installed, installation skipped".to_string(),
                );
            }
            // 打印详细元信息
            log!("Info:Check the following information before installation:");
            println!();
            for (input, info) in &parsed {
                println!(
                    "{}\n",
                    info.fmt_print(
                        FmtPrintCaller::Install(PackageSource::from(input.clone())),
                        true
                    )
                    .unwrap()
                );
            }
            // 询问是否执行
            let tip = &parsed
                .iter()
                .fold("\nTarget packages:\n".to_string(), |acc, node| {
                    acc + &node
                        .1
                        .fmt_brief_print(FmtPrintCaller::Install(PackageSource::from(
                            node.0.clone(),
                        )))
                        .unwrap()
                });
            println!("{tip}");
            if !ask_yn(
                cfg,
                format!(
                    "Ready to install those {} packages, continue?",
                    parsed.len()
                ),
                true,
            ) {
                return Err(anyhow!("Error:Operation canceled by user"));
            }
            // 执行
            install_using_parsed(
                cfg,
                parsed.into_iter().map(|p| p.0).collect(),
                verify_signature,
            )
            .map(|arr| {
                let length = arr.len();
                if length == 1 {
                    String::new()
                } else {
                    format!("Success:{length} packages installed successfully")
                }
            })
        }
        Action::Update {
            package_matchers: packages,
        } => {
            if let Some(packages) = packages {
                // 解析输入
                let parsed = parse_update_inputs(cfg, packages, verify_signature)?;
                log!("Debug:Parsed update packages: {parsed:?}");
                // 打印详细元信息
                log!("Info:Check the following information before update:");
                println!();
                for (input, info) in &parsed {
                    println!(
                        "{}\n",
                        info.fmt_print(
                            FmtPrintCaller::Update(PackageSource::from(input.clone(),)),
                            true
                        )
                        .unwrap()
                    );
                }
                // 询问是否执行
                let tip = &parsed
                    .iter()
                    .fold("\nTarget packages:\n".to_string(), |acc, node| {
                        acc + &node
                            .1
                            .fmt_brief_print(FmtPrintCaller::Update(PackageSource::from(
                                node.0.clone(),
                            )))
                            .unwrap()
                    });
                println!("{tip}");
                if !ask_yn(
                    cfg,
                    format!(
                        "Ready to update with those {} packages, continue?",
                        parsed.len()
                    ),
                    true,
                ) {
                    return Err(anyhow!("Error:Operation canceled by user"));
                }
                // 执行
                update_using_parsed(
                    cfg,
                    parsed.into_iter().map(|p| p.0).collect(),
                    verify_signature,
                )
                .map(|arr| {
                    let length = arr.len();
                    if length == 1 {
                        String::new()
                    } else {
                        format!("Success:{length} packages updated successfully")
                    }
                })
            } else {
                update_all(&mut cfg.clone(), verify_signature).map(|(success_count, failure_count)| {
                    if failure_count == 0 {
                        if success_count == 0 {
                            "Info:No updatable packages".to_string()
                        } else {
                            format!("Success:Updated {success_count} packages")
                        }
                    } else {
                        format!("Error:{failure_count} packages failed to be updated and {success_count} packages updated successfully")
                    }
                })
            }
        }
        Action::Uninstall { package_matchers } => {
            // 解析输入
            let parsed = parse_uninstall_inputs(cfg, package_matchers)?;
            log!("Debug:Parsed uninstall packages: {parsed:?}");
            // 询问是否执行
            let tip = &parsed
                .iter()
                .fold("\nTarget packages:\n".to_string(), |acc, info| {
                    acc + &info.fmt_brief_print(FmtPrintCaller::Uninstall).unwrap()
                });
            println!("{tip}");
            if !ask_yn(
                cfg,
                format!(
                    "Ready to uninstall those {} packages, continue?",
                    parsed.len()
                ),
                true,
            ) {
                return Err(anyhow!("Error:Operation canceled by user"));
            }
            let length = parsed.len();
            for info in parsed {
                let scope = info.scope;
                let name = info.name;
                let tip = uninstall(cfg, Some(scope.clone()), &name).map(|(scope, name)| {
                    format!("Success:Package '{scope}/{name}' uninstalled successfully")
                }).map_err(|e|{
                    // 卸载失败时提示用户如何手动解决坏包
                    let app_path=get_path_apps(cfg, &scope, &name, false).unwrap();
                    anyhow!("Error:Failed to uninstall package '{scope}/{name}' : '{e}', try to manually delete '{}' if this package is broken",p2s!(app_path))
                })?;
                log!("{tip}");
            }
            Ok(if length == 1 {
                String::new()
            } else {
                format!("Success:{length} packages uninstalled successfully")
            })
        }
        Action::Search { keyword, regex } => {
            auto_mirror_update_all(cfg)?;
            search(cfg, &keyword, regex).map(|results| {
                let len = results.len();
                let res: String = results
                    .into_iter()
                    .fold(format!("\nFound {len} results:\n"), |acc, node| {
                        acc + &node.fmt_brief_print()
                    });
                res
            })
        }
        Action::Info {
            package_matcher,
            save_at,
        } => {
            auto_mirror_update_all(cfg)?;
            let parse_res = PackageInputEnum::parse(package_matcher, true, true)?;
            let (info, _) = info(cfg, parse_res, verify_signature)?;
            if let Some(into) = save_at {
                let text = toml::to_string_pretty(&info)?;
                write(&into, text)
                    .map_err(|e| anyhow!("Error:Failed to write to '{into}' : {e}"))?;
                Ok(format!("Success:Info report saved at '{into}'"))
            } else {
                let text = info.fmt_print(FmtPrintCaller::Info, true)?;
                Ok(text)
            }
        }
        Action::List => list(cfg).map(|list| {
            if list.is_empty() {
                return "Info:No installed package".to_string();
            }
            let res: String = list
                .into_iter()
                .fold(String::from("\nInstalled packages:\n"), |acc, node| {
                    acc + &node.fmt_brief_print(FmtPrintCaller::Info).unwrap()
                });
            res
        }),
        Action::Pack {
            source_dir,
            into_file,
        } => pack(cfg, &source_dir, into_file, verify_signature)
            .map(|location| format!("Success:Package stored at '{location}'")),
        Action::Meta {
            package_matcher: package,
            save_at,
        } => {
            // 调用 meta
            let package_input_enum = PackageInputEnum::parse(package, true, true)?;
            let res = meta(cfg, package_input_enum, verify_signature)?;

            // 移除 temp_dir
            let mut res_toml = toml::Value::try_from(res)?;
            let _ = res_toml.as_table_mut().unwrap().remove("temp_dir");

            // 序列化
            let text = toml::to_string_pretty(&res_toml)
                .map_err(|e| anyhow!("Error:Failed to deserialize result : {e}"))?;
            if let Some(into) = save_at {
                write(&into, text)
                    .map_err(|e| anyhow!("Error:Failed to write to '{into}' : {e}"))?;
                Ok(format!("Success:Meta report saved at '{into}'"))
            } else {
                Ok(text)
            }
        }

        Action::Clean => clean(cfg).map(|count| {
            if count == 0 {
                "Info:No trash found".to_string()
            } else {
                format!("Success:{count} trashes found and cleaned")
            }
        }),

        Action::Config { operation } => match operation {
            ActionConfig::Set { table, key, value } => config_set(cfg, &table, &key, &value)
                .map(|_| format!("Success:Config value of '{key}' set to '{value}'")),
            ActionConfig::Get { table, key } => config_get(cfg, &table, &key),
            ActionConfig::List => config_list(cfg),
            ActionConfig::Init => config_init(cfg)
                .map(|location| format!("Success:Initial config stored at '{location}'")),
            ActionConfig::Which => config_which(),
        },
        Action::Mirror { operation } => match operation {
            ActionMirror::Add { url } => {
                mirror_add(cfg, &url, None).map(|name| format!("Success:Mirror '{name}' added"))
            }
            ActionMirror::Update { name } => {
                if let Some(n) = name {
                    mirror_update(cfg, &n)
                        .map(|name| format!("Success:Index of mirror '{name}' updated"))
                } else {
                    mirror_update_all(cfg).map(|names| {
                        if names.is_empty() {
                            "Warning:No mirror has been added".to_string()
                        } else {
                            format!(
                                "Success:Index of mirrors '({name})' updated",
                                name = names.join(", ")
                            )
                        }
                    })
                }
            }
            ActionMirror::List => {
                let res = mirror_list(cfg)?;
                if !res.is_empty() {
                    let str: String = res
                        .into_iter()
                        .fold(String::from("\nAdded mirrors:\n"), |acc, info| {
                            acc + &fmt_print_mirror_line(info)
                        });
                    Ok(str)
                } else {
                    Ok("Info:No mirror added".to_string())
                }
            }
            ActionMirror::Remove { name } => {
                mirror_remove(cfg, &name).map(|_| format!("Success:Mirror '{name}' removed"))
            }
        },
        Action::Upgrade { check } => upgrade(cfg, check, true),
    }
}

#[cfg(not(tarpaulin_include))]
fn main() {
    // 启用虚拟终端
    colored::control::set_virtual_terminal(true).unwrap();

    // 初始化配置
    let mut cfg = Cfg::init().unwrap_or_else(|e| {
        log!("Error:Failed to initialize config : {e}");
        exit(1);
    });

    // 配置环境变量
    let args = Args::parse();
    if args.qa {
        cfg.mode.qa = true;
    }
    if args.debug || args.qa || cfg!(debug_assertions) {
        log!("Warning:Debug mode enabled");
        set_flag(Flag::Debug, true);
    }
    if args.offline {
        log!("Warning:Offline mode enabled, ept couldn't guarantee security or integrality of packages");
        cfg.mode.offline = true;
    }
    if args.qa || args.yes {
        log!("Warning:Confirmation mode enabled");
        cfg.interaction.auto_confirm_all = true;
    }

    // 清理缓存
    launch_clean(&cfg).unwrap();

    // 判断是否需要检查更新
    let need_check_update = cfg.online.auto_check_upgrade
        && !matches!(&args.action, Action::Upgrade { check: _ })
        && !mirror_list(&cfg).unwrap_or_default().is_empty();

    // 使用路由器匹配入口
    write_windows_terminal_status(&cfg, 3);
    let res = router(args.action, &cfg);
    write_windows_terminal_status(&cfg, 0);

    // 判断退出码
    let mut exit_code = 0;
    if let Ok(msg) = &res {
        if !msg.is_empty() {
            log!("{msg}");
        }
    }
    if let Err(msg) = res {
        log!("{msg}");
        exit_code = 1;
    }

    // 检查程序更新
    if need_check_update {
        let check_res = check_has_upgrade(&cfg).map_err(|e| anyhow!("Error:Failed to check self upgrade : '{e}'. If this error persists, consider changing 'online.auto_check_upgrade' to 'false' in config"));
        if let Ok((has_upgrade, is_cross_wid_gap, latest_release)) = check_res {
            if has_upgrade {
                println!();
                log!(
                    "{}",
                    if is_cross_wid_gap {
                        print_upgradable_cross_wid_gap(true, latest_release)
                    } else {
                        print_upgradable(latest_release)
                    }
                )
            }
        } else {
            println!();
            log!("{}", check_res.unwrap_err());
            // exit_code = 1;
        }
    }

    // 退出
    exit(exit_code);
}
