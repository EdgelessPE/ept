use crate::{
    executor::{judge_perm_level, values_validator_path},
    types::permissions::{Permission, PermissionKey},
    utils::{conditions::ensure_arg, path::parse_relative_path_with_located},
};
use anyhow::Result;
use evalexpr::{Function, Value};

use super::EvalFunction;

pub struct Exist {
    //- 检查某个路径指向的文件或目录是否存在
    //@ 需要输入合法的路径
    //# `if = 'Exist("${SystemDrive}/Windows")'`
}

impl EvalFunction for Exist {
    fn get_closure(located: String) -> Function {
        Function::new(move |val| {
            let arg = ensure_arg(val)?;
            let p = parse_relative_path_with_located(&arg, &located);

            Ok(Value::Boolean(p.exists()))
        })
    }
    fn get_permission(arg: String) -> Result<Permission> {
        Ok(Permission {
            key: PermissionKey::fs_read,
            level: judge_perm_level(&arg)?,
            targets: vec![arg],
        })
    }
    fn verify_arg(arg: String) -> Result<()> {
        values_validator_path(&arg)
    }
}
