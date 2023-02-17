mod author;
mod extended_semver;
mod info;
mod package;
mod signature;
mod steps;
mod workflow;
use toml::Value;

pub use self::author::Author;
pub use self::extended_semver::ExSemVer;
pub use self::info::{Info, InfoDiff};
pub use self::package::{GlobalPackage, Package, Software};
pub use self::signature::{Signature, SignatureNode};
pub use self::steps::{Step, StepExecute, StepLink, StepLog, StepPath, TStep};
pub use self::workflow::{WorkflowHeader, WorkflowNode};

#[derive(Clone, Debug)]
pub struct KV {
    pub key: String,
    pub value: Value,
}
