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

use clap::{Parser,Subcommand};

/// Edgeless Package Tool (ept) for Edgeless Next-Generation Packages (nep) [Alpha test]
#[derive(Parser,Debug)]
#[command(version)]
struct Args {
   #[command(subcommand)]
   action: Action,
}

#[derive(Subcommand,Debug)]
enum Action {
    /// Install a package with path (locally in current version)
   Install {
    /// Package path (or package name in future versions)
    package: String,
   },
   /// Uninstall a package with package name
   Uninstall {
    /// Package name
    package_name:String,
   },
   /// Query package information (locally in current version)
   Info {
    /// Package name
    package_name:String,
   },
   /// List informations of installed packages
   List,
   /// Pack a directory content into nep
   Pack {
    /// Source directory ready to be packed
    source_dir:String
   },
}

fn main() {
    let args = Args::parse();
    println!("{:?}",args);
}
