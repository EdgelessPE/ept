use crate::types::GlobalPackage;
use crate::utils::get_exe_version;
use anyhow::{anyhow, Result};
use std::path::Path;
use std::{fs::File, io::Read};

pub fn parse_package(p: String) -> Result<GlobalPackage> {
    let package_path = Path::new(&p);
    if !package_path.exists() {
        return Err(anyhow!("Error:Fatal:Can't find package.toml path : {}", p));
    }

    let mut text = String::new();
    File::open(&p)?.read_to_string(&mut text)?;
    let pkg_res = toml::from_str(&text);
    if pkg_res.is_err() {
        return Err(anyhow!(
            "Error:Can't validate package.toml at {} : {}",
            p,
            pkg_res.err().unwrap()
        ));
    }

    // 根据主程序更新版本号
    // let mp_version=get_exe_version(file_path)

    Ok(pkg_res.unwrap())
}
