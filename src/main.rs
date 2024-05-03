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

use colored::Colorize;
use entrances::meta;
use std::fs::write;
use std::process::exit;

use self::types::cli::{Action, ActionConfig, Args};
use crate::entrances::config::{config_get, config_init, config_list, config_set, config_which};
use crate::entrances::{
    clean, info, install_using_package, list, pack, uninstall, update_using_package,
};

#[cfg(not(tarpaulin_include))]
fn router(action: Action) -> Result<String> {
    // 环境变量读取

    use types::cli::ActionMirror;

    use crate::{
        entrances::{mirror_add, mirror_remove, mirror_update, mirror_update_all, search},
        utils::term::parse_package_matcher,
    };
    let verify_signature = envmnt::get_or("OFFLINE", "false") == String::from("false");

    // 匹配入口
    match action {
        Action::Install { package } => install_using_package(&package, verify_signature)
            .map(|_| format!("Success:Package '{package}' installed successfully")),
        Action::Update { package } => update_using_package(&package, verify_signature)
            .map(|_| format!("Success:Package '{package}' updated successfully")),
        Action::Uninstall { package_matcher } => {
            let parse_res = parse_package_matcher(&package_matcher, true, true)?;
            uninstall(parse_res.scope, &parse_res.name).map(|_| {
                format!(
                    "Success:Package '{name}' uninstalled successfully",
                    name = parse_res.name
                )
            })
        }
        Action::Search { keyword } => search(&keyword).map(|results| {
            let len = results.len();
            let res: String =
                results
                    .into_iter()
                    .fold(format!("\nFound {len} results:\n"), |acc, node| {
                        return acc
                            + &format!(
                                "  {scope}/{name} ({version})   {mirror}\n",
                                name = node.name,
                                version = node.version,
                                scope = node.scope,
                                mirror = node
                                    .from_mirror
                                    .unwrap_or("".to_string())
                                    .as_str()
                                    .truecolor(100, 100, 100)
                            );
                    });
            res
        }),
        Action::Info { package_matcher } => {
            let parse_res = parse_package_matcher(&package_matcher, true, true)?;
            info(parse_res.scope, &parse_res.name).map(|res| format!("{res:#?}"))
        }
        Action::List => list().map(|list| {
            if list.len() == 0 {
                return "No installed package".to_string();
            }
            let res: String =
                list.into_iter()
                    .fold(String::from("\nInstalled packages:\n"), |acc, node| {
                        return acc
                            + &format!(
                                "  {name}    {version}\n",
                                name = node.name,
                                version = node.local.unwrap().version
                            );
                    });
            res
        }),
        Action::Pack {
            source_dir,
            into_file,
        } => pack(&source_dir, into_file, verify_signature)
            .map(|location| format!("Success:Package has been stored at '{location}'")),
        Action::Meta {
            source_package,
            save_at,
        } => {
            let res = meta(&source_package, verify_signature)?;
            let text = serde_json::to_string_pretty(&res)
                .map_err(|e| anyhow!("Error:Failed to deserialize result : {e}"))?;
            if let Some(into) = save_at {
                write(&into, text)
                    .map_err(|e| anyhow!("Error:Failed to write to '{into}' : {e}"))?;
                return Ok(format!("Success:Meta report saved at '{into}'"));
            } else {
                return Ok(text);
            }
        }

        Action::Clean => clean().map(|_| format!("Success:Cleaned")),
        Action::Config { operation } => match operation {
            ActionConfig::Set { table, key, value } => config_set(&table, &key, &value)
                .map(|_| format!("Success:Config value of '{key}' has been set to '{value}'")),
            ActionConfig::Get { table, key } => config_get(&table, &key),
            ActionConfig::List => config_list(),
            ActionConfig::Init => config_init()
                .map(|location| format!("Success:Initial config stored at '{location}'")),
            ActionConfig::Which => config_which(),
        },
        Action::Mirror { operation } => match operation {
            ActionMirror::Add { url } => {
                mirror_add(&url, None).map(|name| format!("Success:Mirror '{name}' has been added"))
            }
            ActionMirror::Update { name } => {
                if let Some(n) = name {
                    mirror_update(&n)
                        .map(|name| format!("Success:Index of mirror '{name}' has been updated"))
                } else {
                    mirror_update_all().map(|names| {
                        format!(
                            "Success:Index of mirrors '({name})' has been updated",
                            name = names.join(", ")
                        )
                    })
                }
            }
            ActionMirror::Remove { name } => {
                mirror_remove(&name).map(|_| format!("Success:Mirror '{name}' has been removed"))
            }
        },
    }
}

#[cfg(not(tarpaulin_include))]
fn main() {
    use clap::Parser;

    use crate::utils::launch_clean;

    let args = Args::parse();

    // 配置环境变量
    if args.qa {
        envmnt::set("QA", "true");
    }
    if args.debug || args.qa || cfg!(debug_assertions) {
        log!("Warning:Debug mode enabled");
        envmnt::set("DEBUG", "true");
    } else {
        launch_clean().unwrap();
    }
    if args.offline {
        log!("Warning:Offline mode enabled, ept couldn't guarantee security or integrality of packages");
        envmnt::set("OFFLINE", "true");
    }
    if args.yes {
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
