use crate::{
    executor::{judge_perm_level, values_validator_path},
    types::permissions::{Permission, PermissionKey},
    utils::{conditions::ensure_arg, path::parse_relative_path_with_located},
};
use anyhow::Result;
use evalexpr::{DefaultNumericTypes, Function, Value};

use super::EvalFunction;
use crate::types::context::RuntimeContext;

pub struct IsDirectory {
    //- 检查某个路径是否指向一个目录
    //@ 需要输入合法的路径
    //# `if = 'IsDirectory("${SystemDrive}/Windows")'`
}

impl EvalFunction for IsDirectory {
    fn get_closure(_ctx: &RuntimeContext, located: String) -> Function<DefaultNumericTypes> {
        Function::new(move |val| {
            let arg = ensure_arg(val)?;
            let p = parse_relative_path_with_located(&arg, &located);

            Ok(Value::Boolean(p.is_dir()))
        })
    }
    fn get_permission(arg: &str) -> Result<Permission> {
        Ok(Permission {
            key: PermissionKey::fs_read,
            level: judge_perm_level(arg)?,
            targets: vec![arg.to_string()],
        })
    }
    fn verify_arg(arg: &str) -> Result<()> {
        values_validator_path(arg)
    }
}

#[test]
fn test_is_directory_verify_arg() {
    assert!(IsDirectory::verify_arg("./test").is_ok());
    assert!(IsDirectory::verify_arg("${AppData}/test").is_ok());
    assert!(IsDirectory::verify_arg("C:\\Windows").is_err());
    assert!(IsDirectory::verify_arg("../test").is_err());
    assert!(IsDirectory::verify_arg("${DefaultLocation}/test").is_err());
}
