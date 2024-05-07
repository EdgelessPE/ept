use crate::{
    entrances::info,
    types::permissions::{Permission, PermissionKey, PermissionLevel},
    utils::conditions::ensure_arg,
};
use anyhow::{anyhow, Result};
use evalexpr::{error, Function, Value};
use regex::Regex;

use super::EvalFunction;

lazy_static! {
    static ref RESOURCE_REGEX: Regex = Regex::new(r"^[^/]+/[^/]+$").unwrap();
}

pub struct IsInstalled {
    //- 检查某个包是否已被 ept 安装
    //@ 需要匹配模式 'SCOPE/NAME'
    //# `if = 'IsInstalled("Microsoft/VSCode")'`
}

impl EvalFunction for IsInstalled {
    fn get_closure(_: String) -> Function {
        Function::new(move |val| {
            let arg = ensure_arg(val)?;
            let sp: Vec<&str> = arg.split('/').collect();
            if sp.len() != 2 {
                return Err(error::EvalexprError::CustomMessage(format!(
                    "Invalid argument '{arg}' : expect 'SCOPE/NAME', e.g. 'Microsoft/VSCode'"
                )));
            }
            let info = info(Some(sp[0].to_string()), &sp[1].to_string());

            Ok(Value::Boolean(info.is_ok()))
        })
    }
    fn get_permission(arg: String) -> Result<Permission> {
        Ok(Permission {
            key: PermissionKey::nep_installed,
            level: PermissionLevel::Normal,
            targets: vec![arg],
        })
    }
    fn verify_arg(arg: String) -> Result<()> {
        if !RESOURCE_REGEX.is_match(&arg) {
            return Err(anyhow!("Error:Argument of 'IsAlive' should match pattern 'SCOPE/NAME' (e.g. Microsoft/VSCode)"));
        }
        Ok(())
    }
}
