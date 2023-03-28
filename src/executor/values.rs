use eval::Expr;

use crate::utils::env::{env_system_drive, env_home, env_appdata, env_program_files_x64, env_desktop, env_program_files_x86};

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