use mslnk::ShellLink;

use crate::types::StepLink;
use mslnk::ShellLink;
use anyhow::{anyhow,Result};

pub fn link(step:StepLink)->Result<i32>{
    let sl_res=ShellLink::new(&step.source_file);
    if sl_res.is_err() {
        return Err(anyhow!("Error(Link):Can't find source file '{}'",&step.source_file));
    }

    let c_res=sl_res.unwrap().create_lnk(&step.target_name);
    if c_res.is_err() {
        return Err(anyhow!("Error(Link):Can't create link {}->{} : {}",&step.source_file,&step.target_name,c_res.unwrap_err().to_string()));
    }
    Ok(0)
}