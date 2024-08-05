use clap::Subcommand;

#[derive(Subcommand, Debug)]
pub enum ActionConfig {
    /// Set the config 'key' to a certain 'value' in 'table'
    Set {
        table: String,
        key: String,
        value: String,
    },
    /// Print the value for a given 'key' in 'table'
    Get { table: String, key: String },
    /// Displays the current configuration [alias 'ls']
    #[clap(alias = "ls")]
    List,
    /// Initialize config file
    Init,
    /// Print which config file is selected
    Which,
}
