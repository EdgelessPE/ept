use clap::Subcommand;

#[derive(Subcommand, Debug)]
pub enum ActionMirror {
    /// Add mirror
    Add {
        /// Mirror url
        url: String,
    },
    /// Update mirror index
    Update {
        /// (Optional) Mirror name, update all if not provided
        name: Option<String>,
    },
    /// Remove mirror
    Remove {
        /// Mirror name
        name: String,
    },
    /// List added mirrors
    List,
}
