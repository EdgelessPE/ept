use crate::types::permissions::{Generalizable, Permission};
use crate::types::verifiable::Verifiable;
use crate::types::KV;
use anyhow::{anyhow, Result};
use serde::de;
use serde::{Deserialize, Serialize};

mod copy;
mod execute;
mod link;
mod log;
mod mv;
mod new;
mod path;
mod rename;
mod wait;
mod toast;

pub trait TStep: Verifiable + Generalizable {
    /// Run this step
    fn run(self, cx: &mut WorkflowContext) -> Result<i32>;
    /// Run reversed step
    fn reverse_run(self, cx: &mut WorkflowContext) -> Result<()>;
    /// Get manifest
    fn get_manifest(&self, fs: &mut MixedFS) -> Vec<String>;
    /// Get interpreted step
    fn interpret<F>(self, interpreter: F) -> Self
    where
        F: Fn(String) -> String;
}

fn toml_try_into<'de, T>(kv: KV) -> Result<T>
where
    T: de::Deserialize<'de>,
{
    let val = kv.value;
    val.to_owned().try_into().map_err(|err| {
        let key = kv.key;
        let name_brw = val["name"].to_owned();
        let name = name_brw.as_str().unwrap_or("unknown name");
        let step = val["step"].as_str().unwrap_or("unknown step");
        anyhow!("Error:Can't parse workflow node '{name}'({key}) into step '{step}' : {err}")
    })
}

macro_rules! def_enum_step {
    ($($x:ident),*) => {

        #[derive(Serialize, Deserialize, Clone, Debug)]
        pub enum Step {
            $( $x($x) ),*
        }

        impl Step {
            pub fn run<F>(self, cx: &mut WorkflowContext, interpreter: F) -> Result<i32>
            where
                F: Fn(String) -> String,
            {
                match self {
                    $( Step::$x(step) => step.interpret(interpreter).run(cx) ),*
                }
            }
            pub fn reverse_run<F>(self, cx: &mut WorkflowContext, interpreter: F) -> Result<()>
            where
                F: Fn(String) -> String,
            {
                match self {
                    $( Step::$x(step) => step.interpret(interpreter).reverse_run(cx) ),*
                }
            }
            pub fn get_manifest(&self, fs: &mut MixedFS) -> Vec<String> {
                match self {
                    $( Step::$x(step) => step.get_manifest(fs) ),*
                }
            }
        }

        impl TryFrom<KV> for Step {
            type Error=anyhow::Error;
            fn try_from(kv:KV)->Result<Step>{
                // 读取步骤名称
                let val=kv.value.clone();
                let step=String::from("Step")+val["step"].as_str().unwrap();

                // 根据步骤名称解析步骤体
                let res=match step.as_str() {
                    $( stringify!($x) => Step::$x(toml_try_into(kv)?) ),* ,
                    _ => {
                        return Err(anyhow!("Error:Unknown step '{step}'"));
                    },
                };
                Ok(res)
            }
        }

        impl Verifiable for Step {
            fn verify_self(&self,located:&String) -> Result<()> {
                match self {
                    $( Step::$x(step) => step.verify_self(located) ),*
                }
            }
        }

        impl Generalizable for Step {
            fn generalize_permissions(&self)->Result<Vec<Permission>> {
                match self {
                    $( Step::$x(step) => step.generalize_permissions() ),*
                }
            }
        }
    };
}

// 注册步骤
def_enum_step!(
    StepLink,
    StepExecute,
    StepPath,
    StepLog,
    StepCopy,
    StepMove,
    StepRename,
    StepNew,
    StepWait
);

pub use self::copy::StepCopy;
pub use self::execute::StepExecute;
pub use self::link::StepLink;
pub use self::log::StepLog;
pub use self::mv::StepMove;
pub use self::new::StepNew;
pub use self::path::StepPath;
pub use self::rename::StepRename;
pub use self::wait::StepWait;

use super::mixed_fs::MixedFS;
use super::workflow::WorkflowContext;
