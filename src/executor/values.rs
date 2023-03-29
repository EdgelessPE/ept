use eval::Expr;
use anyhow::{anyhow,Result};
use crate::utils::env::{env_system_drive, env_home, env_appdata, env_program_files_x64, env_desktop, env_program_files_x86};

// TODO:使用正则表达式实现内置变量收集器，支持过滤 "${ExitCode}", "${DefaultLocation}"

macro_rules! define_values {
    ($({$key:expr,$val:expr}),*) => {
        pub fn values_decorator(expr:Expr, exit_code: i32, located: &String)->Expr{
            expr
            .value("${ExitCode}",exit_code)
            .value("${DefaultLocation}",located.to_owned())
            $(.value($key,$val))*
        }
        
        pub fn values_replacer(raw:String, exit_code: i32, located: &String)->String{
            raw
            .replace("${ExitCode}",&exit_code.to_string())
            .replace("${DefaultLocation}",located)
            $(.replace($key,&$val))*
        }

        /// 仅适用于路径的内置变量校验器
        pub fn values_validator_path(raw:&String)->Result<()>{
            // "${ExitCode}", "${DefaultLocation}" 不是合法的路径开头内置变量，对于 "${DefaultLocation}" 应该使用相对路径
            let valid_start=HashSet::from_iter(vec![$($key),*]);
            if raw.starts_with("${DefaultLocation}"){
                return Err(anyhow!("Error:'${DefaultLocation}' is not allowed in '{}', use './' instead",raw));
            }
            // 内置变量开头
            if raw.starts_with("${"){
                //TODO:正则表达式匹配开头，阻止非法的开头
                return Err(anyhow!("Error:Unknown inner value '{}' in '{}'",&start,raw));
            }
            // 阻止绝对路径
            if Path::new(raw).is_absolute(){
                return Err(anyhow!("Error:Absolute path is not allowed in '{}'",raw));
            }
            // 阻止 ..
            if raw.contains(".."){
                return Err(anyhow!("Error:Double dot '..' is not allowed in '{}'",raw));
            }
            
            Ok(())
        }
    };
}

define_values!{
    {"${SystemDrive}",env_system_drive()},
    {"${Home}",env_home()},
    {"${AppData}",env_appdata()},
    {"${ProgramFiles_X64}",env_program_files_x64()},
    {"${ProgramFiles_X86}",env_program_files_x86()},
    {"${Desktop}",env_desktop()}
}