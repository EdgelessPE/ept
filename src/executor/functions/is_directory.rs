use crate::{
    executor::{judge_perm_level, values_validator_path},
    types::permissions::Permission,
    utils::{ensure_arg, parse_relative_path_with_located},
};
use anyhow::Result;
use evalexpr::{Function, Value};

use super::EvalFunction;

pub struct IsDirectory {}

impl EvalFunction for IsDirectory {
    fn get_closure(located: String) -> Function {
        Function::new(move |val| {
            let arg = ensure_arg(val)?;
            let p = parse_relative_path_with_located(&arg, &located);

            Ok(Value::Boolean(p.is_dir()))
        })
    }
    fn get_permission(arg: String) -> Result<Permission> {
        Ok(Permission {
            key: "fs_read".to_string(),
            level: judge_perm_level(&arg)?,
            targets: vec![arg],
        })
    }
    fn verify_arg(arg: String) -> Result<()> {
        values_validator_path(&arg)
    }
}
