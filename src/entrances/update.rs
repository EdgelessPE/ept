use anyhow::{Result,anyhow};

use crate::parsers::parse_package;


pub fn update_using_package(source_file: String, verify_signature: bool)->Result<()>{
    log(format!("Info:Preparing to update with package '{}'", &source_file));

    // 解包
    let (temp_dir_inner_path,global_package)=unpack_nep(source_file, verify_signature)?;

    // 确认包是否可以升级


    Ok(())
}