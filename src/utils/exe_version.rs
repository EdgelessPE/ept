use anyhow::{anyhow, Result};
use pelite::FileMap;
use pelite::{pe32, pe64};
use std::path::Path;

fn get_64_version<P: AsRef<Path>>(file_path: P) -> Result<String> {
    use pelite::pe::Pe;
    let path = file_path.as_ref();
    if let Ok(map) = FileMap::open(path) {
        let file = pe64::PeFile::from_bytes(&map)
            .map_err(|e| anyhow!("Error:Can't read pe information of '{path:?}' : {e}"))?;

        let resources = file
            .resources()
            .map_err(|e| anyhow!("Error:Can't read resources of '{path:?}' : {e}"))?;
        let version_info = resources
            .version_info()
            .map_err(|e| anyhow!("Error:Can't read version information of '{path:?}' : {e}"))?;
        let fix_opt = version_info.fixed();
        if fix_opt.is_none() {
            return Err(anyhow!("Error:Can't get version info of '{path:?}'"));
        }
        let file_version = fix_opt.unwrap().dwFileVersion;

        Ok(format!(
            "{}.{}.{}.{}",
            file_version.Major, file_version.Minor, file_version.Patch, file_version.Build
        ))
    } else {
        Err(anyhow!("Error:Can't read version of '{path:?}'"))
    }
}

fn get_32_version<P: AsRef<Path>>(file_path: P) -> Result<String> {
    use pelite::pe32::Pe;
    let path = file_path.as_ref();
    if let Ok(map) = FileMap::open(path) {
        let file = pe32::PeFile::from_bytes(&map)
            .map_err(|e| anyhow!("Error:Can't read pe information of '{path:?}' : {e}"))?;

        let resources = file
            .resources()
            .map_err(|e| anyhow!("Error:Can't read resources of '{path:?}' : {e}"))?;
        let version_info = resources
            .version_info()
            .map_err(|e| anyhow!("Error:Can't read version information of '{path:?}' : {e}"))?;
        let fix_opt = version_info.fixed();
        if fix_opt.is_none() {
            return Err(anyhow!("Error:Can't get version info of '{path:?}'"));
        }
        let file_version = fix_opt.unwrap().dwFileVersion;

        Ok(format!(
            "{}.{}.{}.{}",
            file_version.Major, file_version.Minor, file_version.Patch, file_version.Build
        ))
    } else {
        Err(anyhow!("Error:Can't read version of '{path:?}'"))
    }
}

pub fn get_exe_version<P: AsRef<Path>>(file_path: P) -> Result<String> {
    let path = file_path.as_ref();
    if !path.exists() {
        return Err(anyhow!(
            "Error:Failed to get version : file '{path:?}' not exist"
        ));
    }
    if let Ok(res) = get_32_version(&file_path) {
        return Ok(res);
    }
    if let Ok(res) = get_64_version(&file_path) {
        return Ok(res);
    }

    Err(anyhow!("Error:Can't read version of '{path:?}'"))
}

#[test]
fn test_get_exe_version() {
    let res = get_exe_version("./examples/Dism++/Dism++/Dism++x64.exe").unwrap();
    assert_eq!(res, String::from("10.1.1002.1"));

    let res = get_exe_version("./examples/Dism++/Dism++/Dism++x86.exe").unwrap();
    assert_eq!(res, String::from("10.1.1002.1"));

    let res = get_exe_version("./examples/Dism++/Dism++/Dism++ARM64.exe").unwrap();
    assert_eq!(res, String::from("10.1.1002.1"));
}
