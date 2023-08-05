mod config;

pub use self::config::ActionConfig;
use clap::{Parser, Subcommand};

/// [Alpha] Edgeless Package Tool (ept) for Edgeless Next-Generation Packages (nep)
#[derive(Parser, Debug)]
#[command(version)]
pub struct Args {
    #[command(subcommand)]
    pub action: Action,
    /// Confirm each "Yes or No" question
    #[arg(short, long)]
    pub yes: bool,

    /// Strict mode, throw immediately when a workflow step goes wrong
    #[arg(short, long)]
    pub strict: bool,

    /// (Dangerous) Disable online Edgeless CA to skip signature signing or verifying
    #[arg(long)]
    pub offline: bool,

    /// Run commands in debug mode
    #[arg(short, long)]
    pub debug: bool,
    // TODO:支持日志写入文件，并检查 println!
}

#[derive(Subcommand, Debug)]
pub enum Action {
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

    /// Manage ept config files
    Config {
        #[command(subcommand)]
        operation: ActionConfig,
    },
}
