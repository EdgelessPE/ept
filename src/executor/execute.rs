use std::process::Command;
use std::str::{from_utf8};
use anyhow::{Result, anyhow};
use crate::types::{StepExecute};
use crate::utils::{log};

fn read_console(v:Vec<u8>)->String {
    let msg_res=from_utf8(&v);
    if msg_res.is_err() {
        log("Warning(Execute):Console output can't be parsed with utf8".to_string());
        String::new()
    }else{
        msg_res.unwrap().to_string()
    }
}

pub fn execute(step:StepExecute)->Result<i32>{
    // 配置终端
    let launch_terminal=if cfg!(target_os = "windows") {
        ("cmd","/c")
    }else {
        ("sh","-c")
    };

    // 构造执行器
    let mut c=Command::new(launch_terminal.0);
    let cmd=c.args([launch_terminal.1,&step.command]);

    // 指定工作目录
    let pwd_opt=step.pwd;
    if pwd_opt.is_some() {
        cmd.current_dir(pwd_opt.unwrap());
    }

    // 执行并收集结果
    let output_res=cmd.output();
    if output_res.is_err() {
        return Err(anyhow!("Error(Execute):Command '{}' spawned failed : {}",&step.command,output_res.unwrap_err()));
    }
    let output=output_res.unwrap();

    // 处理退出码
    match output.status.code() {
        Some(val)=>{
            if val==0 {
                log(format!("Info(Execute):Command '{}' output : \n{}",&step.command,&read_console(output.stdout)));
            }else{
                log(format!("Error(Execute):Command '{}' failed, output : \n{}",&step.command,&read_console(output.stderr)));
            }
                Ok(val)
        },
        None=>{
            Err(anyhow!("Error(Execute):Command '{}' terminated by signal",&step.command))
        }
    }

}

#[test]
fn test_execute(){
    execute(StepExecute{
        command:"echo hello nep !".to_string(),
        pwd:None
    }).unwrap();
    execute(StepExecute{
        command:"ls".to_string(),
        pwd:Some("./src".to_string())
    }).unwrap();

    let res=execute(StepExecute{
        command:"exit 2".to_string(),
        pwd:None
    }).unwrap();
    assert_eq!(res,2);
}