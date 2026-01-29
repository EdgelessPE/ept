mod exist;
mod is_alive;
mod is_directory;
mod is_installed;

use self::{exist::Exist, is_alive::IsAlive, is_directory::IsDirectory, is_installed::IsInstalled};
use crate::types::permissions::Permission;
use anyhow::{anyhow, Result};
use evalexpr::*;

macro_rules! def_eval_functions {
    ($($x:ident),*) => {
        pub fn set_context_with_function(context: &mut HashMapContext,located: &str) {
            $(
                context.set_function(
                    stringify!($x).to_string(),
                    $x::get_closure(located.to_string()),
                ).unwrap();
             )*
        }

        pub fn get_eval_function_names()->Vec<&'static str> {
            vec![$( stringify!($x) ),*]
        }

        pub fn get_eval_function_permission(name:&str,arg:&str)->Result<Permission>{
            match name {
                $( stringify!($x) => $x::get_permission(arg) ),* ,
                _=>Err(anyhow!("Error:Unknown eval function name '{name}'"))
            }
        }

        pub fn verify_eval_function_arg(name:&str,arg:&str)->Result<()> {
            match name {
                $( stringify!($x) => $x::verify_arg(arg) ),* ,
                _=>Err(anyhow!("Error:Unknown eval function name '{name}'"))
            }
        }
    };
}

trait EvalFunction {
    fn get_closure(located: String) -> Function<DefaultNumericTypes>;
    fn get_permission(arg: &str) -> Result<Permission>;
    fn verify_arg(arg: &str) -> Result<()>;
}

def_eval_functions!(Exist, IsDirectory, IsAlive, IsInstalled);
