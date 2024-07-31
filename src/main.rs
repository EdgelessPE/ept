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

use anyhow::{anyhow, Result};
use clap::Parser;
use colored::Colorize;
use entrances::meta;
use std::fs::write;
use std::process::exit;

use self::types::cli::{Action, ActionConfig, Args};
use crate::entrances::config::{config_get, config_init, config_list, config_set, config_which};
use crate::entrances::{
    auto_mirror_update_all, clean, info, install_using_package, list, pack, uninstall, update_all,
    update_using_package,
};
use crate::utils::cfg::get_config;
use crate::utils::launch_clean;

#[cfg(not(tarpaulin_include))]
fn router(action: Action) -> Result<String> {
    // 环境变量读取

    use chrono::DateTime;
    use types::{cli::ActionMirror, extended_semver::ExSemVer};

    use crate::{
        entrances::{
            install_using_package_matcher, install_using_url, mirror_add, mirror_list,
            mirror_remove, mirror_update, mirror_update_all, search, update_using_package_matcher,
            update_using_url,
        },
        types::matcher::{PackageInputEnum, PackageMatcher},
    };
    let cfg = get_config();
    let verify_signature = envmnt::get_or("OFFLINE", "false") == *"false";

    // 匹配入口
    match action {
        Action::Install { package } => {
            let res = match PackageInputEnum::parse(package.clone(), false, false)? {
                PackageInputEnum::PackageMatcher(matcher) => {
                    auto_mirror_update_all(&cfg)?;
                    install_using_package_matcher(matcher, verify_signature)
                }
                PackageInputEnum::Url(url) => install_using_url(&url, verify_signature),
                PackageInputEnum::LocalPath(source_file) => {
                    install_using_package(&source_file, verify_signature)
                }
            };
            res.map(|_| format!("Success:Package '{package}' installed successfully"))
        }
        Action::Update { package } => {
            if let Some(package) = package {
                let res = match PackageInputEnum::parse(package.clone(), false, false)? {
                    PackageInputEnum::PackageMatcher(matcher) => {
                        auto_mirror_update_all(&cfg)?;
                        update_using_package_matcher(matcher, verify_signature)
                    }
                    PackageInputEnum::Url(url) => update_using_url(&url, verify_signature),
                    PackageInputEnum::LocalPath(source_file) => {
                        update_using_package(&source_file, verify_signature)
                    }
                };
                res.map(|info| info.format_success())
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
        Action::Uninstall { package_matcher } => {
            let parse_res = PackageMatcher::parse(&package_matcher, true, true)?;
            uninstall(parse_res.scope, &parse_res.name).map(|_| {
                format!(
                    "Success:Package '{name}' uninstalled successfully",
                    name = parse_res.name
                )
            })
        }
        Action::Search { keyword } => {
            auto_mirror_update_all(&cfg)?;
            search(&keyword).map(|results| {
                let len = results.len();
                let res: String =
                    results
                        .into_iter()
                        .fold(format!("\nFound {len} results:\n"), |acc, node| {
                            acc + &format!(
                                "  {scope}/{name}    ({version})   {mirror}\n",
                                name = node.name.cyan().bold(),
                                version = node.version,
                                scope = node.scope.cyan().italic(),
                                mirror = node
                                    .from_mirror
                                    .unwrap_or("".to_string())
                                    .as_str()
                                    .truecolor(100, 100, 100)
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
                        acc + &format!(
                            "  {scope}/{name}    ({local_ver}{update_tip})\n",
                            scope = node.software.unwrap().scope.cyan().italic(),
                            name = node.name.cyan().bold(),
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
                format!("Success:{count} trash found and cleaned")
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
                    let str: String = res.into_iter().fold(
                        String::from("\nAdded mirrors:\n"),
                        |acc, (name, time)| {
                            let date_time: DateTime<chrono::Local> = time.into();
                            let time_str = date_time.format("%Y-%m-%d %H:%M:%S").to_string();
                            let update_str = format!("updated at {time_str}");
                            acc + &format!(
                                "  {name}    {str}\n",
                                str = update_str.as_str().truecolor(100, 100, 100)
                            )
                        },
                    );
                    Ok(str)
                } else {
                    Ok("Info:No mirror added".to_string())
                }
            }
            ActionMirror::Remove { name } => {
                mirror_remove(&name).map(|_| format!("Success:Mirror '{name}' removed"))
            }
        },
    }
}

#[cfg(not(tarpaulin_include))]
fn main() {
    // 清理缓存
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
    if args.strict {
        log!("Warning:Strict mode enabled");
        envmnt::set("STRICT", "true");
    }

    // 使用路由器匹配入口
    let res = router(args.action);
    if res.is_ok() {
        log!("{msg}", msg = res.unwrap());
        exit(0);
    } else {
        log!("{msg}", msg = res.unwrap_err());
        exit(1);
    }
}
