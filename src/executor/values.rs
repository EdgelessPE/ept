use crate::log;
use crate::utils::env::{
    env_appdata, env_desktop, env_home, env_program_files_x64, env_program_files_x86,
    env_system_drive,
};
use anyhow::{anyhow, Result};
use eval::Expr;
use lazy_static::lazy_static;
use regex::Regex;
use std::collections::HashSet;
use std::path::Path;

lazy_static! {
    static ref RE: Regex = Regex::new(r"\$\{(\w+)\}").unwrap();
}

macro_rules! define_values {
    ($({$key:expr,$val:expr}),*) => {
        fn get_arr(extra:bool)->Vec<String>{
            let mut arr=vec![$($key.to_string()),*];
            if extra{
                arr.push("${ExitCode}".to_string());
                arr.push( "${DefaultLocation}".to_string());
            }
            arr
        }

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

define_values! {
    {"${SystemDrive}",env_system_drive()},
    {"${Home}",env_home()},
    {"${AppData}",env_appdata()},
    {"${ProgramFiles_X64}",env_program_files_x64()},
    {"${ProgramFiles_X86}",env_program_files_x86()},
    {"${Desktop}",env_desktop()}
}

// 收集合法的内置变量
pub fn collect_values(raw: &String) -> Result<Vec<String>> {
    let valid_values: HashSet<String> = HashSet::from_iter(get_arr(true));

    let collection: Vec<String> = RE
        .captures_iter(raw)
        .filter_map(|cap| {
            let str = cap.get(0).unwrap().as_str();
            if valid_values.contains(str) {
                Some(str.to_string())
            } else {
                log!("Warning:Unknown value '{str}' in '{raw}', check if it's a spelling mistake");
                None
            }
        })
        .collect();

    Ok(collection)
}

/// 仅适用于路径的内置变量校验器
pub fn values_validator_manifest_path(raw: &String) -> Result<()> {
    // "${DefaultLocation}" 不是合法的路径内置变量，应该使用相对路径
    if raw.contains("${DefaultLocation}") {
        return Err(anyhow!(
            "Error:'${}' is not allowed in '{raw}', use './' instead",
            "{DefaultLocation}"
        ));
    }
    // 阻止绝对路径
    if Path::new(raw).is_absolute() {
        return Err(anyhow!(
            "Error:Absolute path '{raw}' is not allowed, use proper inner values instead"
        ));
    }
    // 阻止 ..
    if raw.contains("..") {
        return Err(anyhow!("Error:Double dot '..' is not allowed in '{raw}'"));
    }

    // 收集合法的内置变量
    let collection = collect_values(raw)?;

    // 如果以一个近似内置变量的名称打头，但是又不存在这个内置变量则阻止
    if raw.starts_with("${") && !raw.starts_with(&collection[0]) {
        return Err(anyhow!(
            "Error:Shouldn't start with an unknown inner value in '{raw}'"
        ));
    }

    // 阻止使用一个以上的 env 变量
    let first_elem=collection.get(0).map(|s|s.to_owned()).unwrap_or("".to_string());
    let env_hash_set:HashSet<String>=HashSet::from_iter(get_arr(false));
    let env_count=collection
    .into_iter()
    .fold(0, |acc,x|{
        if env_hash_set.contains(&x){
            acc+1
        }else{
            acc
        }
    });
    if env_count>1{
        return Err(anyhow!("Error:Illegal usage of env inner values in '{raw}' : 1 at most, got {env_count}"));
    }

    // 只能在开头使用
    if env_count==1&&!raw.starts_with(&first_elem){
        return Err(anyhow!("Error:Illegal usage of '{first_elem}' in '{raw}' : can only appear at the beginning"));
    }

    Ok(())
}

#[test]
fn test_collect_values() {
    values_validator_manifest_path(&"${AppData}${ExitCode}.${SystemData}/".to_string()).unwrap();

    let err_res =
        values_validator_manifest_path(&"${SystemData}${AppData}${ExitCode}./".to_string());
    assert!(err_res.is_err());
    log!("{}",err_res.unwrap_err());

    let err_res = values_validator_manifest_path(&"C:/system".to_string());
    assert!(err_res.is_err());
    log!("{}",err_res.unwrap_err());

    let err_res = values_validator_manifest_path(&"${AppData}/../nep".to_string());
    assert!(err_res.is_err());
    log!("{}",err_res.unwrap_err());

    let err_res = values_validator_manifest_path(&"114${DefaultLocation}/vscode".to_string());
    assert!(err_res.is_err());
    log!("{}",err_res.unwrap_err());

    let err_res = values_validator_manifest_path(&"${AppData}/./${ExitCode}${Home}/nep".to_string());
    assert!(err_res.is_err());
    log!("{}",err_res.unwrap_err());

    let err_res = values_validator_manifest_path(&"$/{${Desktop}/vscode".to_string());
    assert!(err_res.is_err());
    log!("{}",err_res.unwrap_err());
}
