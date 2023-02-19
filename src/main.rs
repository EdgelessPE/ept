#[macro_use]
extern crate lazy_static;
extern crate tar;

mod ca;
mod compression;
mod entrances;
mod executor;
mod parsers;
mod signature;
mod types;
#[macro_use]
mod utils;

pub use self::utils::{fn_log, fn_log_ok_last};
use anyhow::Result;
use clap::{Parser, Subcommand};
use entrances::update_using_package;

use crate::entrances::{info, install_using_package, list, pack, uninstall};

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
}

fn router(action: Action) -> Result<String> {
    // 环境变量读取
    let verify_signature = envmnt::get_or("OFFLINE", "false") == String::from("false");

    // 匹配入口
    match action {
        Action::Install { package } => install_using_package(package.clone(), verify_signature)
            .map(|_| format!("Success:Package '{}' installed successfully", &package)),
        Action::Update { package } => update_using_package(package.clone(), verify_signature)
            .map(|_| format!("Success:Package '{}' updated successfully", &package)),
        Action::Uninstall { package_name } => uninstall(package_name.clone()).map(|_| {
            format!(
                "Success:Package '{}' uninstalled successfully",
                &package_name
            )
        }),
        Action::Info { package_name } => info(package_name).map(|res| format!("{:#?}", res)),
        Action::List => list().map(|list| {
            if list.len() == 0 {
                return "No installed package".to_string();
            }
            let mut res_str = String::from("Installed packages:");
            for node in list {
                res_str += &format!("  {}    {}", &node.name, &node.local.unwrap().version);
            }
            res_str
        }),
        Action::Pack {
            source_dir,
            into_file,
        } => pack(source_dir, into_file, verify_signature)
            .map(|location| format!("Success:Package has been stored at '{}'", &location)),
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
        envmnt::set("OFFLINE", "true")
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
        log!("{}", res.unwrap());
    } else {
        log!("{}", res.unwrap_err().to_string())
    }
}
