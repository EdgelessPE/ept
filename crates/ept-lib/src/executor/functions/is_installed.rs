use crate::{
    entrances::info_local,
    types::permissions::{Permission, PermissionKey, PermissionLevel},
    utils::conditions::ensure_arg,
};
use anyhow::{anyhow, Result};
use evalexpr::{error, DefaultNumericTypes, Function, Value};
use regex::Regex;

use super::EvalFunction;
use crate::types::context::RuntimeContext;

lazy_static! {
    static ref RESOURCE_REGEX: Regex = Regex::new(r"^[^/]+/[^/]+$").unwrap();
}

pub struct IsInstalled {
    //- 检查某个包是否已被 ept 安装
    //@ 需要匹配模式 'SCOPE/NAME'
    //# `if = 'IsInstalled("Microsoft/VSCode")'`
}

impl EvalFunction for IsInstalled {
    fn get_closure(ctx: &RuntimeContext, _: String) -> Function<DefaultNumericTypes> {
        let cfg = ctx.clone();
        Function::new(move |val| {
            let arg = ensure_arg(val)?;
            let sp: Vec<&str> = arg.split('/').collect();
            if sp.len() != 2 {
                return Err(error::EvalexprError::CustomMessage(format!(
                    "Invalid argument '{arg}' : expect 'SCOPE/NAME', e.g. 'Microsoft/VSCode'"
                )));
            }
            let info = info_local(&cfg, sp[0], sp[1]);

            Ok(Value::Boolean(info.is_ok()))
        })
    }
    fn get_permission(arg: &str) -> Result<Permission> {
        Ok(Permission {
            key: PermissionKey::nep_installed,
            level: PermissionLevel::Normal,
            targets: vec![arg.to_string()],
        })
    }
    fn verify_arg(arg: &str) -> Result<()> {
        if !RESOURCE_REGEX.is_match(arg) {
            return Err(anyhow!("Error:Argument of 'IsAlive' should match pattern 'SCOPE/NAME' (e.g. Microsoft/VSCode)"));
        }
        Ok(())
    }
}

#[test]
fn test_is_installed_verify_arg() {
    assert!(IsInstalled::verify_arg("Microsoft/VSCode").is_ok());
    assert!(IsInstalled::verify_arg("github/VSCode").is_ok());
    assert!(IsInstalled::verify_arg("Microsoft").is_err());
    assert!(IsInstalled::verify_arg("a/b/c").is_err());
    assert!(IsInstalled::verify_arg("").is_err());
}

#[test]
fn test_is_installed_get_permission() {
    let perm = IsInstalled::get_permission("Microsoft/VSCode").unwrap();
    assert_eq!(perm.key, PermissionKey::nep_installed);
    assert_eq!(perm.targets, vec!["Microsoft/VSCode"]);
}
