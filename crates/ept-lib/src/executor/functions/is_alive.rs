use crate::{
    types::permissions::{Permission, PermissionKey, PermissionLevel},
    utils::{conditions::ensure_arg, process::is_alive_with_name},
};
use anyhow::{anyhow, Result};
use evalexpr::{DefaultNumericTypes, Function, Value};

use super::EvalFunction;
use crate::types::context::RuntimeContext;

pub struct IsAlive {
    //- 检查某个进程是否正在运行
    //@ 需要以 `.exe` 结尾
    //# `if = 'IsAlive("code.exe")'`
}

impl EvalFunction for IsAlive {
    fn get_closure(_ctx: &RuntimeContext, _: String) -> Function<DefaultNumericTypes> {
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

#[test]
fn test_is_alive_verify_arg() {
    assert!(IsAlive::verify_arg("code.exe").is_ok());
    assert!(IsAlive::verify_arg("CODE.EXE").is_ok());
    assert!(IsAlive::verify_arg("code").is_err());
    assert!(IsAlive::verify_arg("code.txt").is_err());
    assert!(IsAlive::verify_arg("").is_err());
}

#[test]
fn test_is_alive_get_permission() {
    let perm = IsAlive::get_permission("code.exe").unwrap();
    assert_eq!(perm.key, PermissionKey::process_query);
}
