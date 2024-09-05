use crate::types::permissions::{Generalizable, Permission};
use anyhow::{anyhow, Result};
use serde::de;
use serde::{Deserialize, Serialize};
use toml::Value;

mod copy;
mod delete;
mod download;
mod execute;
mod kill;
mod link;
mod log;
mod mv;
mod new;
mod path;
mod rename;
mod toast;
mod wait;

pub struct VerifyStepCtx {
    pub located: String,
    pub is_expand_flow: bool,
}

impl VerifyStepCtx {
    pub fn _demo() -> Self {
        Self {
            located: "".to_string(),
            is_expand_flow: false,
        }
    }
}

pub trait TStep: Generalizable + Interpretable {
    /// Run this step, return 0 by default
    fn run(self, cx: &mut WorkflowContext) -> Result<i32>;
    /// Run reversed step
    fn reverse_run(self, cx: &mut WorkflowContext) -> Result<()>;
    /// Get manifest
    fn get_manifest(&self, fs: &mut MixedFS) -> Vec<String>;
    /// Verify step
    fn verify_step(&self, ctx: &VerifyStepCtx) -> Result<()>;
}

fn toml_try_into<'de, T>(key: String, val: Value) -> Result<T>
where
    T: de::Deserialize<'de>,
{
    val.to_owned().try_into().map_err(|err| {
        let name_brw = val["name"].to_owned();
        let name = name_brw.as_str().unwrap_or("unknown name");
        let step = val["step"].as_str().unwrap_or("unknown step");
        anyhow!("Error:Can't parse workflow node '{name}'({key}) into step '{step}' : {err}")
    })
}

macro_rules! def_enum_step {
    ($($x:ident),*) => {

        #[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
        #[allow(clippy::enum_variant_names)]
        pub enum Step {
            $( $x($x) ),*
        }

        impl Step {
            pub fn try_from_kv(key:String,val:Value)->Result<Step>{
                // 读取步骤名称
                let step=String::from("Step")+val["step"].as_str().unwrap();

                // 根据步骤名称解析步骤体
                let res=match step.as_str() {
                    $( stringify!($x) => Step::$x(toml_try_into(key,val)?) ),* ,
                    _ => {
                        return Err(anyhow!("Error:Unknown step '{step}'"));
                    },
                };
                Ok(res)
            }

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
            pub fn verify_step(&self,ctx:&VerifyStepCtx) -> Result<()> {
                match self {
                    $( Step::$x(step) => step.verify_step(ctx) ),*
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
    StepWait,
    StepToast,
    StepDelete,
    StepKill,
    StepDownload
);

pub use self::copy::StepCopy;
pub use self::delete::StepDelete;
pub use self::download::StepDownload;
pub use self::execute::StepExecute;
pub use self::kill::StepKill;
pub use self::link::StepLink;
pub use self::log::StepLog;
pub use self::mv::StepMove;
pub use self::new::StepNew;
pub use self::path::StepPath;
pub use self::rename::StepRename;
pub use self::toast::StepToast;
pub use self::wait::StepWait;

use super::interpretable::Interpretable;
use super::mixed_fs::MixedFS;
use super::workflow::WorkflowContext;
