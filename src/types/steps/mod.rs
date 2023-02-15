use anyhow::Result;
use serde::{Deserialize, Serialize};

mod execute;
mod link;
mod log;
mod path;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum Step {
    StepExecute(StepExecute),
    StepLink(StepLink),
    StepLog(StepLog),
    StepPath(StepPath),
}

pub trait TStep {
    /// Run this step
    fn run(self, located: &String) -> Result<i32>;
    /// Run reversed step
    fn reverse_run(self, located: &String) -> Result<()>;
    /// Get manifest
    fn get_manifest(&self) -> Vec<String>;
    /// Get interpreted step
    fn interpret<F>(self, interpreter: F) -> Self
    where
        F: Fn(String) -> String;
}

pub use self::execute::StepExecute;
pub use self::link::StepLink;
pub use self::log::StepLog;
pub use self::path::StepPath;
