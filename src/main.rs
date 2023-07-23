#[macro_use]
extern crate lazy_static;
extern crate tar;

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
use clap::{Parser, Subcommand};
use entrances::meta;
use std::fs::write;
use std::process::exit;

use crate::entrances::{
    clean, info, install_using_package, list, pack, uninstall, update_using_package,
};

/// [Alpha] Edgeless Package Tool (ept) for Edgeless Next-Generation Packages (nep)
#[derive(Parser, Debug)]
#[command(version)]
struct Args {
    #[command(subcommand)]
    action: Action,
    /// Confirm each "Yes or No" question
    #[arg(short, long)]
    yes: bool,
    /// Strict mode, throw immediately when a workflow step goes wrong
    #[arg(short, long)]
    strict: bool,
    /// (Dangerous) Disable online Edgeless CA to skip signature signing or verifying
    #[arg(long)]
    offline: bool,
    /// Run commands in debug mode
    #[arg(short, long)]
    debug: bool,
}

#[derive(Subcommand, Debug)]
enum Action {
    /// Install a package with path (locally in current version)
    Install {
        /// Package path (or package name in future versions)
        package: String,
    },
    /// Update a package with path (locally in current version)
    Update {
        /// Package path (or package name in future versions)
        package: String,
    },
    /// Uninstall a package with package name
    Uninstall {
        /// Package name
        package_name: String,
    },
    /// Pack a directory content into nep
    Pack {
        /// Source directory ready to be packed
        source_dir: String,
        /// (Optional) Store packed nep at
        into_file: Option<String>,
    },
    /// Query package information (locally in current version)
    Info {
        /// Package name
        package_name: String,
    },
    /// List information of installed packages
    List,
    /// Get meta data of given package
    Meta {
        /// Source package
        source_package: String,
        /// (Optional) Save meta report at
        save_at: Option<String>,
    },
    /// Clean temporary or illegal files
    Clean,
}

fn router(action: Action) -> Result<String> {
    // 环境变量读取
    let verify_signature = envmnt::get_or("OFFLINE", "false") == String::from("false");

    // 匹配入口
    match action {
        Action::Install { package } => install_using_package(&package, verify_signature)
            .map(|_| format!("Success:Package '{package}' installed successfully")),
        Action::Update { package } => update_using_package(&package, verify_signature)
            .map(|_| format!("Success:Package '{package}' updated successfully")),
        Action::Uninstall { package_name } => uninstall(&package_name)
            .map(|_| format!("Success:Package '{package_name}' uninstalled successfully")),
        Action::Info { package_name } => info(None, &package_name).map(|res| format!("{res:#?}")),
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
    }
}

fn main() {
    let args = Args::parse();

    // 配置环境变量
    if args.debug || cfg!(debug_assertions) {
        log!("Warning:Debug mode enabled");
        envmnt::set("DEBUG", "true");
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
