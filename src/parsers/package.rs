use std::{fs::File, io::Read};
use std::path::Path;
use anyhow::{anyhow,Result};
use crate::types::{GlobalPackage};


pub fn parse_package(p:String)->Result<GlobalPackage>{
    let package_path=Path::new(&p);
    if !package_path.exists() {
        return Err(anyhow!("Error:Fatal:Can't find package path : {}",p));
    }

    let mut text=String::new();
    File::open(&p)?.read_to_string(&mut text)?;
    let pkg=toml::from_str(&text)?;

    //TODO:校验 schema 

    Ok(pkg)
}