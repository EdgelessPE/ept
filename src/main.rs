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
mod utils;

use clap::{Parser, Subcommand};
use entrances::update_using_package;

use crate::{
    entrances::{info, install_using_package, list, pack, uninstall},
    utils::log,
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
}

fn main() {
    let args = Args::parse();

    // 配置环境变量
    if args.debug {
        log("Warning:Debug mode enabled".to_string());
        envmnt::set("DEBUG", "true");
    }
    if args.offline {
        log("Warning:Offline mode enabled, ept couldn't guarantee security or integrality of packages".to_string());
        envmnt::set("OFFLINE", "true")
    }
    if args.yes {
        log("Warning:Confirmation mode enabled".to_string());
        envmnt::set("CONFIRM", "true");
    }
    if args.strict {
        log("Warning:Strict mode enabled".to_string());
        envmnt::set("STRICT", "true");
    }

    // 匹配入口
    let verify_signature = envmnt::get_or("OFFLINE", "false") == String::from("false");
    match args.action {
        Action::Install { package } => {
            let res = install_using_package(package.clone(), verify_signature);
            if res.is_err() {
                log(res.unwrap_err().to_string());
            } else {
                log(format!(
                    "Success:Package '{}' installed successfully",
                    &package
                ));
            }
        }
        Action::Update { package } => {
            let res = update_using_package(package.clone(), verify_signature);
            if res.is_err() {
                log(res.unwrap_err().to_string());
            } else {
                log(format!(
                    "Success:Package '{}' updated successfully",
                    &package
                ));
            }
        }
        Action::Uninstall { package_name } => {
            let res = uninstall(package_name.clone());
            if res.is_err() {
                log(res.unwrap_err().to_string());
            } else {
                log(format!(
                    "Success:Package '{}' uninstalled successfully",
                    &package_name
                ));
            }
        }
        Action::Info { package_name } => {
            let res = info(package_name);
            if res.is_err() {
                log(res.unwrap_err().to_string());
            } else {
                println!("{:#?}", res.unwrap());
            }
        }
        Action::List => {
            let res = list();
            if res.is_err() {
                log(res.unwrap_err().to_string());
            } else {
                println!("Installed packages:");
                for node in res.unwrap() {
                    println!("  {}    {}", &node.name, &node.local.unwrap().version);
                }
            }
        }
        Action::Pack {
            source_dir,
            into_file,
        } => {
            let res = pack(source_dir, into_file, verify_signature);
            if res.is_err() {
                log(res.unwrap_err().to_string());
            } else {
                log(format!(
                    "Success:Package has been stored at '{}'",
                    &res.unwrap()
                ));
            }
        }
    };

    ()
}
