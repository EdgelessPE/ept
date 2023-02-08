use anyhow::{anyhow, Result};
use pelite::pe64::{Pe, PeFile};
use pelite::FileMap;
use std::path::Path;

pub fn get_exe_version(file_path: String) -> Result<String> {
    let path = Path::new(&file_path);
    if let Ok(map) = FileMap::open(path) {
        let file = PeFile::from_bytes(&map).map_err(|e| {
            anyhow!(
                "Error:Can't read pe information of '{}' : {}",
                &file_path,
                e.to_string()
            )
        })?;

        let resources = file.resources().map_err(|e| {
            anyhow!(
                "Error:Can't read resources of '{}' : {}",
                &file_path,
                e.to_string()
            )
        })?;
        let version_info = resources.version_info().map_err(|e| {
            anyhow!(
                "Error:Can't read version information of '{}' : {}",
                &file_path,
                e.to_string()
            )
        })?;
        let fix_opt = version_info.fixed();
        if fix_opt.is_none() {
            return Err(anyhow!("Error:Can't get version info of '{}'", &file_path));
        }
        let file_version = fix_opt.unwrap().dwFileVersion;

        Ok(format!(
            "{}.{}.{}.{}",
            file_version.Major, file_version.Minor, file_version.Patch, file_version.Build
        ))
    } else {
        Err(anyhow!("Error:Can't read version of '{}'", &file_path))
    }
}

#[test]
fn test_get_exe_version() {
    let res = get_exe_version(r"D:\CnoRPS\Dism++10.1.1002.1\Dism++x64.exe".to_string()).unwrap();
    assert_eq!(res, String::from("10.1.1002.1"));

    let res = get_exe_version(r"D:\CnoRPS\aria2\aria2c.exe".to_string());
    println!("{:?}", res);
}
