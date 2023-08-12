use crate::log;
use crate::types::permissions::PermissionLevel;
use crate::utils::env::{
    env_appdata, env_desktop, env_home, env_program_files_x64, env_program_files_x86,
    env_public_desktop, env_system_drive,
};
use crate::utils::{get_arch, is_starts_with_inner_value};
use anyhow::{anyhow, Result};
use evalexpr::{ContextWithMutableVariables, HashMapContext, Value};
use lazy_static::lazy_static;
use regex::Regex;
use std::collections::HashSet;
use std::path::Path;

lazy_static! {
    static ref RE: Regex = Regex::new(r"\$\{(\w+)\}").unwrap();
}

macro_rules! define_values {
    ($({$key:expr, $val:expr, $perm:expr}),*) => {
        fn get_arr(extra:bool)->Vec<String>{
            let mut arr=vec![$($key.to_string()),*];
            if extra{
                arr.push("${ExitCode}".to_string());
                arr.push("${DefaultLocation}".to_string());
            }
            arr
        }

        // 支持虚假的模板字符串
        pub fn values_replacer(raw:String, exit_code: i32, located: &String)->String{
            raw
            .replace("${ExitCode}",&exit_code.to_string())
            .replace("${DefaultLocation}",located)
            $(.replace($key,&$val))*
        }

        pub fn set_context_with_constant_values(context: &mut HashMapContext){
            $(
                context.set_value(
                    $key[2..$key.len()-1].to_string(),
                    Value::String($val)
                ).unwrap();
            )*
        }

        pub fn set_context_with_mutable_values(context: &mut HashMapContext, exit_code: i32, located: &String){
            context.set_value("ExitCode".to_string(),Value::Int(exit_code.into())).unwrap();
            context.set_value("DefaultLocation".to_string(),Value::String(located.to_owned())).unwrap();
        }

        pub fn match_value_permission(value:&String)->Result<PermissionLevel>{
            let perm=match value.as_str() {
                $($key=>$perm,)*
                _=>PermissionLevel::Normal,
            };
            Ok(perm)
        }
    };
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

/// 适用于路径入参的内置变量使用规范校验器
pub fn values_validator_path(raw: &String) -> Result<()> {
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
    if is_starts_with_inner_value(raw)
        && (collection.len() == 0 || !raw.starts_with(&collection[0]))
    {
        return Err(anyhow!(
            "Error:Shouldn't start with an unknown inner value in '{raw}'"
        ));
    }

    // 阻止使用一个以上的 env 变量
    let first_elem = collection
        .get(0)
        .map(|s| s.to_owned())
        .unwrap_or("".to_string());
    let env_hash_set: HashSet<String> = HashSet::from_iter(get_arr(false));
    let env_count = collection.into_iter().fold(0, |acc, x| {
        if env_hash_set.contains(&x) {
            acc + 1
        } else {
            acc
        }
    });
    if env_count > 1 {
        return Err(anyhow!(
            "Error:Illegal usage of env inner values in '{raw}' : 1 at most, got {env_count}"
        ));
    }

    // 只能在开头使用
    if env_count == 1 && !raw.starts_with(&first_elem) {
        return Err(anyhow!(
            "Error:Illegal usage of '{first_elem}' in '{raw}' : can only appear at the beginning"
        ));
    }

    // 检查 env 变量的使用，后面必须加 "/"
    for env_val in get_arr(false) {
        let str = &env_val[2..env_val.len() - 1];
        let reg = Regex::new(&format!(r"\${}{}{}[^/]", r"\{", str, r"\}"))?;
        if reg.is_match(raw) {
            return Err(anyhow!("Error:Path value '{env_val}' must be followed by a slash in '{raw}' (e.g. ${}/Windows/system32)","{SystemDrive}"));
        }
    }

    Ok(())
}

/// 给定内置函数访问的 fs 目标（包含内置变量），需要的权限级别
pub fn judge_perm_level(fs_target: &String) -> Result<PermissionLevel> {
    // 收集使用到的内置变量
    let values = collect_values(fs_target)?;

    let mut final_perm = PermissionLevel::Normal;
    for val in values {
        let cur = match_value_permission(&val)?;
        if cur > final_perm {
            final_perm = cur;
        }
    }

    Ok(final_perm)
}

// {变量名称，值计算表达式，对应权限等级}
define_values! {
    {"${SystemDrive}",env_system_drive(),PermissionLevel::Sensitive},
    {"${Home}",env_home(),PermissionLevel::Important},
    {"${AppData}",env_appdata(),PermissionLevel::Sensitive},
    {"${ProgramFiles_X64}",env_program_files_x64(),PermissionLevel::Sensitive},
    {"${ProgramFiles_X86}",env_program_files_x86(),PermissionLevel::Sensitive},
    {"${Desktop}",env_desktop(),PermissionLevel::Important},
    {"${PublicDesktop}",env_public_desktop(),PermissionLevel::Important},
    {"${Arch}",get_arch().unwrap().to_string(),PermissionLevel::Normal}
}

#[test]
fn test_collect_values() {
    values_validator_path(&"${AppData}/${ExitCode}.${SystemData}/".to_string()).unwrap();

    let err_res = values_validator_path(&"${SystemData}${AppData}${ExitCode}./".to_string());
    assert!(err_res.is_err());
    log!("{e}", e = err_res.unwrap_err());

    let err_res = values_validator_path(&"C:/system".to_string());
    assert!(err_res.is_err());
    log!("{e}", e = err_res.unwrap_err());

    let err_res = values_validator_path(&"${AppData}/../nep".to_string());
    assert!(err_res.is_err());
    log!("{e}", e = err_res.unwrap_err());

    let err_res = values_validator_path(&"114${DefaultLocation}/vscode".to_string());
    assert!(err_res.is_err());
    log!("{e}", e = err_res.unwrap_err());

    let err_res = values_validator_path(&"${AppData}/./${ExitCode}${Home}/nep".to_string());
    assert!(err_res.is_err());
    log!("{e}", e = err_res.unwrap_err());

    let err_res = values_validator_path(&"$/{${Desktop}/vscode".to_string());
    assert!(err_res.is_err());
    log!("{e}", e = err_res.unwrap_err());

    let err_res = values_validator_path(&"${Desktop}vscode".to_string());
    assert!(err_res.is_err());
    log!("{e}", e = err_res.unwrap_err());
}
