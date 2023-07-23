use crate::{
    types::permissions::{Permission, PermissionLevel},
    utils::{ensure_arg, is_alive_with_name},
};
use anyhow::{anyhow, Result};
use evalexpr::{Function, Value};

use super::EvalFunction;

pub struct IsAlive {}

impl EvalFunction for IsAlive {
    fn get_closure(_: String) -> Function {
        Function::new(move |val| {
            let arg = ensure_arg(val)?;
            Ok(Value::Boolean(is_alive_with_name(&arg)))
        })
    }
    fn get_permission(arg: String) -> Result<Permission> {
        Ok(Permission {
            key: "process_query".to_string(),
            level: PermissionLevel::Normal,
            targets: vec![arg],
        })
    }
    fn verify_arg(arg: String) -> Result<()> {
        if !arg.to_ascii_lowercase().ends_with(".exe") {
            return Err(anyhow!(
                "Error:Argument of 'IsAlive' should ends with '.exe', got '{arg}'"
            ));
        }
        Ok(())
    }
}