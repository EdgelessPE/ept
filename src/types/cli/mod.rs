mod config;
mod mirror;
pub use self::config::ActionConfig;
pub use self::mirror::ActionMirror;
use clap::{Parser, Subcommand};

/// Edgeless Package Tool (ept) for Next-Generation Edgeless Packages (nep)
#[derive(Parser, Debug)]
#[command(version)]
pub struct Args {
    #[command(subcommand)]
    pub action: Action,
    /// Confirm each "Yes or No" question
    #[arg(short, long)]
    pub yes: bool,

    /// (Dangerous) Disable online Edgeless CA to skip signature signing or verifying
    #[arg(long)]
    pub offline: bool,

    /// Tweaking certain behavior when running in Edgeless QA
    #[arg(long)]
    pub qa: bool,

    /// Run commands in debug mode
    #[arg(short, long)]
    pub debug: bool,
}

#[derive(Subcommand, Debug)]
pub enum Action {
    /// Install a package [alias 'i' 'add']
    #[clap(aliases = &["i", "add"])]
    Install {
        /// Package matchers（expect pattern ((MIRROR/)SCOPE/)NAME(@SEMVER)）or Nep package url or Nep package local path
        packages: Vec<String>,
    },

    /// Update all updatable packages or a specified package [alias 'up']
    #[clap(alias = "up")]
    Update {
        /// Package matchers（expect pattern ((MIRROR/)SCOPE/)NAME(@SEMVER)）or Nep package url or Nep package local path
        packages: Option<Vec<String>>,
    },

    /// Uninstall packages [alias 'remove' 'rm']
    #[clap(aliases = &["remove","rm"])]
    Uninstall {
        /// Package matcher, expect pattern (SCOPE/)NAME
        package_matchers: Vec<String>,
    },

    /// Search a package
    Search {
        /// Keyword
        keyword: String,
        /// Use keyword as a regular expression, e.g. ept search -r 'vsc\w+'
        #[arg(short, long)]
        regex: bool,
    },

    /// Query a package
    Info {
        /// Package matcher, expect pattern (SCOPE/)NAME
        package_matcher: String,
    },

    /// List information of installed packages [alias 'ls']
    #[clap(alias = "ls")]
    List,

    /// Get meta data of given package
    Meta {
        /// Package matcher, expect pattern (SCOPE/)NAME or Nep package local path
        package: String,
        /// (Optional) Save meta report at
        save_at: Option<String>,
    },

    /// Pack a directory content into nep
    Pack {
        /// Source directory ready to be packed
        source_dir: String,
        /// (Optional) Store packed nep at
        into_file: Option<String>,
    },

    /// Manage ept config
    Config {
        #[command(subcommand)]
        operation: ActionConfig,
    },

    /// Manage nep mirrors
    Mirror {
        #[command(subcommand)]
        operation: ActionMirror,
    },

    /// Upgrade ept toolchain
    Upgrade {
        /// Check for upgrade without performing the update process
        #[arg(short, long)]
        check: bool,
    },

    /// Clean temporary or illegal files
    Clean,
}
