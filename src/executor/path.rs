use crate::types::{StepPath};
use crate::utils::log;
use winreg::{enums::*,RegKey};
use anyhow::{anyhow,Result};

pub fn step_path(step:StepPath)->Result<i32>{
    // 打开 HKEY_CURRENT_USER\Environment
    let hkcu=RegKey::predef(HKEY_CURRENT_USER);
    let table_res=hkcu.open_subkey("Environment");
    if table_res.is_err() {
        return Err(anyhow!("Error(Path):Can't open user register"));
    }
    let table=table_res.unwrap();

    // 读取 Path 键值
    let p_res=table.get_value("Path");
    if p_res.is_err() {
        return Err(anyhow!("Error(Path):Can't get 'Path' in register"));
    }

    // 拆分 Path 为数组
    let origin_text:String=p_res.unwrap();
    let mut origin_arr:Vec<&str>=origin_text.split(";").collect();

    // 检查给定的值是否已经存在
    let is_exist=origin_arr.contains(&step.record.as_str());
    
    // 增删 Path 变量
    let n=format!("\"{}\"",&step.record);
    let ns=n.as_str();
    match step.operation.as_str() {
        "Add"=>{
            if is_exist {
                log(format!("Warning(Path):Record '{}' already existed in PATH",&step.record));
                return Ok(0);
            }else{
                origin_arr.push(ns);
            }
        },
        "Remove"=>{
            if is_exist {
                origin_arr=origin_arr
                .into_iter()
                .filter(|x| {x!=&step.record})
                .collect();
            }else{
                log(format!("Warning(Path):Record '{}' not exist in PATH",&step.record));
                return Ok(0);
            }
        },
        _=>{
            return Err(anyhow!("Error(Path):Unknown operation : {}",&step.operation));
        }
    }

    // 生成新字符串
    let new_arr:Vec<&str>=origin_arr
    .into_iter()
    .filter(|x|{x.to_owned()!=""})
    .collect();
    let new_text=new_arr.join(";");
    log(format!("Debug(Path):Save register with '{}'",&new_text));

    // 写回注册表
    let (table,_)=hkcu.create_subkey("Environment")?;
    let w_res=table.set_value("Path", &new_text);
    if w_res.is_err() {
        return Err(anyhow!("Error(Path):Can't write to register : {}",w_res.unwrap_err().to_string()));
    }

    Ok(0)
}

#[test]
fn test_path(){
    step_path(StepPath{
        record:String::from(r"D:\CnoRPS\2345Pic"),
        operation:"Add".to_string()
    }).unwrap();
}