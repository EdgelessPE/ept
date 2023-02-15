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

impl Step {
    pub fn run<F>(self, located: &String, interpreter: F) -> Result<i32>
    where
        F: Fn(String) -> String{
        match self {
            Step::StepLink(step) => step.interpret(interpreter).run(&located),
            Step::StepExecute(step) => step.interpret(interpreter).run(&located),
            Step::StepPath(step) => step.interpret(interpreter).run(&located),
            Step::StepLog(step) => step.interpret(interpreter).run(&located),
        }
    }
    pub fn reverse_run<F>(self, located: &String, interpreter: F) -> Result<()>
    where
    F: Fn(String) -> String{
        match self {
            Step::StepLink(step) => step.interpret(interpreter).reverse_run(&located),
            Step::StepExecute(step) => step.interpret(interpreter).reverse_run(&located),
            Step::StepPath(step) => step.interpret(interpreter).reverse_run(&located),
            Step::StepLog(step) => step.interpret(interpreter).reverse_run(&located),
        }
    }
    pub fn get_manifest(&self) -> Vec<String> {
        match self {
            Step::StepLink(step) => step.get_manifest(),
            Step::StepExecute(step) => step.get_manifest(),
            Step::StepPath(step) => step.get_manifest(),
            Step::StepLog(step) => step.get_manifest(),
        }
    }
}

pub use self::execute::StepExecute;
pub use self::link::StepLink;
pub use self::log::StepLog;
pub use self::path::StepPath;
