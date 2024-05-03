use semver::VersionReq;

pub mod author;
pub mod cli;
pub mod extended_semver;
pub mod info;
pub mod interpretable;
pub mod meta;
pub mod mirror;
pub mod mixed_fs;
pub mod package;
pub mod permissions;
pub mod signature;
pub mod software;
pub mod steps;
pub mod uninstall_reg_entry;
pub mod verifiable;
pub mod workflow;

#[derive(Clone, Debug, PartialEq)]
pub struct PackageMatcher {
    pub name: String,
    pub scope: Option<String>,
    pub mirror: Option<String>,
    pub version_req: Option<VersionReq>,
}
