#[macro_use]
extern crate lazy_static;
extern crate tar;
#[macro_use]
extern crate tantivy;
mod ca;
mod compression;
mod entrances;
mod executor;
mod parsers;
mod signature;
#[macro_use]
mod types;
#[macro_use]
mod utils;

use self::types::cfg::Cfg;
use self::types::cli::{Action, ActionConfig, Args};
use crate::entrances::config::{config_get, config_init, config_list, config_set, config_which};
use crate::entrances::{
    auto_mirror_update_all, clean, info, install_using_package, list, pack, uninstall, update_all,
};
use crate::utils::cfg::get_config;
use crate::utils::envmnt;
use crate::utils::launch_clean;
use anyhow::{anyhow, Result};
use clap::Parser;
use colored::Colorize;
use entrances::meta;
use std::fs::write;
use std::process::exit;

#[cfg(not(tarpaulin_include))]
fn router(action: Action, cfg: Cfg) -> Result<String> {
    // 环境变量读取
    use entrances::{install_using_parsed, update_using_parsed, upgrade};
    use types::{cli::ActionMirror, extended_semver::ExSemVer};
    use utils::{
        fmt_print::{fmt_mirror_line, fmt_package_line},
        get_path_apps,
        parse_inputs::{parse_install_inputs, parse_uninstall_inputs, parse_update_inputs},
        term::ask_yn,
    };

    use crate::{
        entrances::{
            mirror_add, mirror_list, mirror_remove, mirror_update, mirror_update_all, search,
        },
        types::matcher::{PackageInputEnum, PackageMatcher},
    };
    let verify_signature = envmnt::get_or("OFFLINE", "false") == *"false";

    // 匹配入口
    match action {
        Action::Install { packages } => {
            // 解析输入
            let parsed = parse_install_inputs(packages)?;
            // 询问是否执行
            let tip = &parsed
                .iter()
                .fold("\nTarget packages:\n".to_string(), |acc, node| {
                    acc + &node.to_string()
                });
            println!("{tip}");
            if !ask_yn(
                format!(
                    "Ready to install those {} packages, continue?",
                    parsed.len()
                ),
                true,
            ) {
                return Err(anyhow!("Error:Operation canceled by user"));
            }
            // 执行
            install_using_parsed(parsed, verify_signature).map(|arr| {
                let length = arr.len();
                if length == 1 {
                    String::new()
                } else {
                    format!("Success:{length} packages installed successfully")
                }
            })
        }
        Action::Update { packages } => {
            if let Some(packages) = packages {
                // 解析输入
                let parsed = parse_update_inputs(packages)?;
                // 询问是否执行
                let tip = &parsed
                    .iter()
                    .fold("\nTarget packages:\n".to_string(), |acc, node| {
                        acc + &node.to_string()
                    });
                println!("{tip}");
                if !ask_yn(
                    format!(
                        "Ready to update with those {} packages, continue?",
                        parsed.len()
                    ),
                    true,
                ) {
                    return Err(anyhow!("Error:Operation canceled by user"));
                }
                // 执行
                update_using_parsed(parsed, verify_signature).map(|arr| {
                    let length = arr.len();
                    if length == 1 {
                        String::new()
                    } else {
                        format!("Success:{length} packages updated successfully")
                    }
                })
            } else {
                update_all(verify_signature).map(|(success_count, failure_count)| {
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
            let parsed = parse_uninstall_inputs(package_matchers)?;
            // 询问是否执行
            let tip = &parsed.iter().fold(
                "\nTarget packages:\n".to_string(),
                |acc, (scope, name, version)| acc + &fmt_package_line(scope, name, version, None),
            );
            println!("{tip}");
            if !ask_yn(
                format!(
                    "Ready to uninstall those {} packages, continue?",
                    parsed.len()
                ),
                true,
            ) {
                return Err(anyhow!("Error:Operation canceled by user"));
            }
            let length = parsed.len();
            for (scope, name, _) in parsed {
                let tip = uninstall(Some(scope.clone()), &name).map(|(scope, name)| {
                    format!("Success:Package '{scope}/{name}' uninstalled successfully")
                }).map_err(|e|{
                    // 卸载失败时提示用户如何手动解决坏包
                    let app_path=get_path_apps(&scope, &name, false).unwrap();
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
            auto_mirror_update_all(&cfg)?;
            search(&keyword, regex).map(|results| {
                let len = results.len();
                let res: String =
                    results
                        .into_iter()
                        .fold(format!("\nFound {len} results:\n"), |acc, node| {
                            acc + &fmt_package_line(
                                &node.scope,
                                &node.name,
                                &node.version,
                                node.from_mirror,
                            )
                        });
                res
            })
        }
        Action::Info { package_matcher } => {
            auto_mirror_update_all(&cfg)?;
            let parse_res = PackageMatcher::parse(&package_matcher, true, true)?;
            info(parse_res.scope, &parse_res.name).map(|res| format!("{res:#?}"))
        }
        Action::List => list().map(|list| {
            if list.is_empty() {
                return "Info:No installed package".to_string();
            }
            let res: String =
                list.into_iter()
                    .fold(String::from("\nInstalled packages:\n"), |acc, node| {
                        let local_ver = node.local.unwrap().version;
                        let update_tip = if let Some(online_diff) = node.online {
                            let online_ver = online_diff.version;
                            if ExSemVer::parse(&online_ver).unwrap()
                                > ExSemVer::parse(&local_ver).unwrap()
                            {
                                format!("  ↑ {online_ver}").green().to_string()
                            } else {
                                String::new()
                            }
                        } else {
                            String::new()
                        };
                        acc + &fmt_package_line(
                            &node.software.unwrap().scope,
                            &node.name,
                            &format!("{local_ver}{update_tip}"),
                            None,
                        )
                    });
            res
        }),
        Action::Pack {
            source_dir,
            into_file,
        } => pack(&source_dir, into_file, verify_signature)
            .map(|location| format!("Success:Package stored at '{location}'")),
        Action::Meta { package, save_at } => {
            envmnt::set("NO_WARNING", "true");
            let package_input_enum = PackageInputEnum::parse(package, true, true)?;
            let res = meta(package_input_enum, verify_signature)?;
            let text = serde_json::to_string_pretty(&res)
                .map_err(|e| anyhow!("Error:Failed to deserialize result : {e}"))?;
            if let Some(into) = save_at {
                write(&into, text)
                    .map_err(|e| anyhow!("Error:Failed to write to '{into}' : {e}"))?;
                Ok(format!("Success:Meta report saved at '{into}'"))
            } else {
                Ok(text)
            }
        }

        Action::Clean => clean().map(|count| {
            if count == 0 {
                "Info:No trash found".to_string()
            } else {
                format!("Success:{count} trashes found and cleaned")
            }
        }),

        Action::Config { operation } => match operation {
            ActionConfig::Set { table, key, value } => config_set(&table, &key, &value)
                .map(|_| format!("Success:Config value of '{key}' set to '{value}'")),
            ActionConfig::Get { table, key } => config_get(&table, &key),
            ActionConfig::List => config_list(),
            ActionConfig::Init => config_init()
                .map(|location| format!("Success:Initial config stored at '{location}'")),
            ActionConfig::Which => config_which(),
        },
        Action::Mirror { operation } => match operation {
            ActionMirror::Add { url } => {
                mirror_add(&url, None).map(|name| format!("Success:Mirror '{name}' added"))
            }
            ActionMirror::Update { name } => {
                if let Some(n) = name {
                    mirror_update(&n)
                        .map(|name| format!("Success:Index of mirror '{name}' updated"))
                } else {
                    mirror_update_all().map(|names| {
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
                let res = mirror_list()?;
                if !res.is_empty() {
                    let str: String = res
                        .into_iter()
                        .fold(String::from("\nAdded mirrors:\n"), |acc, (name, time)| {
                            acc + &fmt_mirror_line(&name, time)
                        });
                    Ok(str)
                } else {
                    Ok("Info:No mirror added".to_string())
                }
            }
            ActionMirror::Remove { name } => {
                mirror_remove(&name).map(|_| format!("Success:Mirror '{name}' removed"))
            }
        },
        Action::Upgrade { check } => upgrade(check, true),
    }
}

#[cfg(not(tarpaulin_include))]
fn main() {
    use entrances::mirror_list;
    // 清理缓存
    use utils::upgrade::{check_has_upgrade, fmt_upgradable, fmt_upgradable_cross_wid_gap};
    launch_clean().unwrap();

    // 启用虚拟终端
    colored::control::set_virtual_terminal(true).unwrap();

    // 配置环境变量
    let args = Args::parse();
    if args.qa {
        envmnt::set("QA", "true");
    }
    if args.debug || args.qa || cfg!(debug_assertions) {
        log!("Warning:Debug mode enabled");
        envmnt::set("DEBUG", "true");
    }
    if args.offline {
        log!("Warning:Offline mode enabled, ept couldn't guarantee security or integrality of packages");
        envmnt::set("OFFLINE", "true");
    }
    if args.qa || args.yes {
        log!("Warning:Confirmation mode enabled");
        envmnt::set("CONFIRM", "true");
    }

    // 获取配置
    let cfg = get_config();

    // 判断是否需要检查更新
    let need_check_update = cfg.online.auto_check_upgrade
        && !matches!(&args.action, Action::Upgrade { check: _ })
        && !mirror_list().unwrap_or_default().is_empty();

    // 使用路由器匹配入口
    let res = router(args.action, cfg);

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
        let (has_upgrade, is_cross_wid_gap, latest_release)= check_has_upgrade().map_err(|e|anyhow!("Error:Failed to check self upgrade : '{e}'. If this error persists, consider changing 'online.auto_check_upgrade' to 'false' in config")).unwrap();
        if has_upgrade {
            println!();
            log!(
                "{}",
                if is_cross_wid_gap {
                    fmt_upgradable_cross_wid_gap(true, latest_release)
                } else {
                    fmt_upgradable(latest_release)
                }
            )
        }
    }

    // 退出
    exit(exit_code);
}
