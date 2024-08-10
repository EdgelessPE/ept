use clap::Subcommand;

#[derive(Subcommand, Debug)]
pub enum ActionMirror {
    /// Add mirror
    Add {
        /// Mirror url
        url: String,
    },
    /// Update mirror index [alias 'up']
    #[clap(alias = "up")]
    Update {
        /// (Optional) Mirror name, update all if not provided
        name: Option<String>,
    },
    /// Remove mirror [alias 'rm']
    #[clap(alias = "rm")]
    Remove {
        /// Mirror name
        name: String,
    },
    /// List added mirrors [alias 'ls']
    #[clap(alias = "ls")]
    List,
}
