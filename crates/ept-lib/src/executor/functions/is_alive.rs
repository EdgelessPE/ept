use crate::{
    types::permissions::{Permission, PermissionKey, PermissionLevel},
    utils::{conditions::ensure_arg, process::is_alive_with_name},
};
use anyhow::{anyhow, Result};
use evalexpr::{DefaultNumericTypes, Function, Value};

use super::EvalFunction;

pub struct IsAlive {
    //- 检查某个进程是否正在运行
    //@ 需要以 `.exe` 结尾
    //# `if = 'IsAlive("code.exe")'`
}

impl EvalFunction for IsAlive {
    fn get_closure(
        _cfg: &crate::types::context::RuntimeContext,
        _: String,
    ) -> Function<DefaultNumericTypes> {
        Function::new(move |val| {
            let arg = ensure_arg(val)?;
            Ok(Value::Boolean(is_alive_with_name(&arg)))
        })
    }
    fn get_permission(arg: &str) -> Result<Permission> {
        Ok(Permission {
            key: PermissionKey::process_query,
            level: PermissionLevel::Normal,
            targets: vec![arg.to_string()],
        })
    }
    fn verify_arg(arg: &str) -> Result<()> {
        if !arg.to_ascii_lowercase().ends_with(".exe") {
            return Err(anyhow!(
                "Error:Argument of 'IsAlive' should ends with '.exe', got '{arg}'"
            ));
        }
        Ok(())
    }
}
