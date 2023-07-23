use crate::{
    entrances::info,
    utils::{ensure_arg, is_alive_with_name, parse_relative_path_with_located},
};
use evalexpr::*;

pub fn get_context_with_function(located: &String) -> HashMapContext {
    let l1 = located.to_owned();
    let l2 = located.to_owned();
    context_map! {
        "Exist"=>Function::new(move |val| {
        let arg = ensure_arg(val)?;
        let p = parse_relative_path_with_located(&arg, &l1);

        Ok(Value::Boolean(p.exists()))
    }),
    "IsDirectory"=>Function::new(move |val| {
        let arg = ensure_arg(val)?;
        let p = parse_relative_path_with_located(&arg, &l2);

        Ok(Value::Boolean(p.is_dir()))
    }),
    "IsAlive"=>Function::new(move |val|{
        let arg = ensure_arg(val)?;
        Ok(Value::Boolean(is_alive_with_name(&arg)))
    }),
    "IsInstalled"=>Function::new(move |val| {
        let arg = ensure_arg(val)?;
        let sp: Vec<&str> = arg.split("/").collect();
        if sp.len() != 2 {
            return Err(error::EvalexprError::CustomMessage(format!("Invalid argument '{arg}' : expect 'SCOPE/NAME', e.g. 'Microsoft/VSCode'")));
        }
        let info = info(Some(sp[0].to_string()), &sp[1].to_string());

        Ok(Value::Boolean(info.is_ok()))
    })
    }
    .unwrap()
}
