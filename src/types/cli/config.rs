use clap::Subcommand;

#[derive(Subcommand, Debug)]
pub enum ActionConfig {
    /// Set the config 'key' to a certain 'value'
    Set { key: String, value: String },
    /// Echoes the value for a given 'key'
    Get { key: String },
    /// Displays the current configuration
    List,
    /// Initialize config file
    Init,
}
