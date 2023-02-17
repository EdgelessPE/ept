use anyhow::Result;
use serde::{Deserialize, Serialize};

mod execute;
mod link;
mod log;
mod path;


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

macro_rules! def_enum_step {
    ($($x:ident),*) => {

        #[derive(Serialize, Deserialize, Clone, Debug)]
        pub enum Step {
            $( $x($x) ),*
        }
        
        impl Step {
            pub fn run<F>(self, located: &String, interpreter: F) -> Result<i32>
            where
                F: Fn(String) -> String,
            {
                match self {
                    $( Step::$x(step) => step.interpret(interpreter).run(&located) ),*
                }
            }
            pub fn reverse_run<F>(self, located: &String, interpreter: F) -> Result<()>
            where
                F: Fn(String) -> String,
            {
                match self {
                    $( Step::$x(step) => step.interpret(interpreter).reverse_run(&located) ),*
                }
            }
            pub fn get_manifest(&self) -> Vec<String> {
                match self {
                    $( Step::$x(step) => step.get_manifest() ),*
                }
            }
        }

    };
}

def_enum_step!(StepLink,StepExecute,StepPath,StepLog);

pub use self::execute::StepExecute;
pub use self::link::StepLink;
pub use self::log::StepLog;
pub use self::path::StepPath;
