pub mod author;
pub mod extended_semver;
pub mod info;
pub mod meta;
pub mod mixed_fs;
pub mod package;
pub mod permissions;
pub mod signature;
pub mod software;
pub mod steps;
pub mod verifiable;
pub mod workflow;

use toml::Value;

#[derive(Clone, Debug)]
pub struct KV {
    pub key: String,
    pub value: Value,
}
