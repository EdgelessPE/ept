use std::path::Path;

use eval::Expr;

pub fn functions_decorator(expr:Expr)->Expr{
    expr
    .function("Exist", |val| {
        // 参数校验
        if val.len() > 1 {
            return Err(eval::Error::ArgumentsGreater(1));
        }
        if val.len() == 0 {
            return Err(eval::Error::ArgumentsLess(1));
        }
        let str_opt = val[0].as_str();
        if str_opt.is_none() {
            return Err(eval::Error::Custom(
                "Error:Internal function 'Exist' should accept a string".to_string(),
            ));
        }
        let p = Path::new(str_opt.unwrap());

        Ok(eval::Value::Bool(p.exists()))
    })
    .function("IsDirectory", |val| {
        // 参数校验
        if val.len() > 1 {
            return Err(eval::Error::ArgumentsGreater(1));
        }
        if val.len() == 0 {
            return Err(eval::Error::ArgumentsLess(1));
        }
        let str_opt = val[0].as_str();
        if str_opt.is_none() {
            return Err(eval::Error::Custom(
                "Error:Internal function 'IsDirectory' should accept a string".to_string(),
            ));
        }
        let p = Path::new(str_opt.unwrap());

        Ok(eval::Value::Bool(p.is_dir()))
    })
}