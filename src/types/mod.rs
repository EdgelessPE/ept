mod author;
mod extended_semver;
mod info;
mod package;
mod signature;
mod software;
mod steps;
mod verifiable;
mod workflow;

use toml::Value;

pub use self::author::Author;
pub use self::extended_semver::ExSemVer;
pub use self::info::{Info, InfoDiff};
pub use self::package::{GlobalPackage, Package};
pub use self::signature::{Signature, SignatureNode};
pub use self::software::Software;
pub use self::steps::{Step, StepExecute, StepLink, StepLog, StepPath, TStep};
pub use self::verifiable::Verifiable;
pub use self::workflow::{WorkflowHeader, WorkflowNode};

#[derive(Clone, Debug)]
pub struct KV {
    pub key: String,
    pub value: Value,
}
