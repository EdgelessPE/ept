use anyhow::Result;

mod link;
mod log;
mod execute;
mod path;

pub trait TStep {
    /// Run this step
    fn run(self,located:&String)->Result<i32>;
    /// Run reversed step
    fn reverse_run(self,located:&String)->Result<()>;
    /// Get manifest
    fn get_manifest(&self)->Vec<String>;
    /// Get interpreted step
    fn interpret<F>(self,interpreter:F)->Self
    where F:Fn(String)->String;
}

pub use self::link::StepLink;
pub use self::path::StepPath;
pub use self::log::StepLog;
pub use self::execute::StepExecute;