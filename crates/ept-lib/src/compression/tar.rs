use crate::log;
use anyhow::{anyhow, Result};
use std::fs::{create_dir_all, remove_dir_all, File};
use std::path::Path;
use tar::{Archive, Builder};

pub fn release_tar(source: &str, into: &str) -> Result<()> {
    log!("Debug:Extracting tar '{source}' to '{into}'");
    let file = File::open(source)?;
    let mut archive = Archive::new(file);

    // 覆盖解压
    let p = Path::new(into);
    if p.exists() {
        log!("Debug:Removing existing directory '{into}'");
        remove_dir_all(into)?;
    }
    create_dir_all(into)?;

    archive.unpack(into)?;
    log!("Debug:Successfully extracted tar to '{into}'");
    Ok(())
}

pub fn pack_tar(source: &str, store_at: &str) -> Result<()> {
    log!("Debug:Packing tar from '{source}' to '{store_at}'");
    let file = File::create(store_at)
        .map_err(|e| anyhow!("Error:Failed to create file at '{store_at}' : {e}"))?;
    let mut archive = Builder::new(file);
    archive.append_dir_all(".", source)?;
    archive.finish()?;
    log!("Debug:Successfully packed tar to '{store_at}'");
    Ok(())
}

#[test]
fn test_pack_tar() {
    crate::utils::test::_ensure_clear_test_dir();
    let p = Path::new("./test/VSCode_1.0.0.0_Cno.tar");
    if p.exists() {
        std::fs::remove_file(p).unwrap();
    }
    pack_tar("examples/VSCode", "./test/VSCode_1.0.0.0_Cno.tar").unwrap();
    assert!(p.exists());
}

#[test]
fn test_release_tar() {
    crate::utils::test::_ensure_clear_test_dir();
    if !Path::new("./test/VSCode_1.0.0.0_Cno.tar").exists() {
        test_pack_tar();
    }

    release_tar("./test/VSCode_1.0.0.0_Cno.tar", "./test/VSCode_1.0.0.0_Cno").unwrap();

    assert!(Path::new("test/VSCode_1.0.0.0_Cno/package.toml").exists());

    // 测试覆盖
    use crate::utils::fs::try_recycle;
    try_recycle("test/VSCode_1.0.0.0_Cno/package.toml").unwrap();
    release_tar("./test/VSCode_1.0.0.0_Cno.tar", "./test/VSCode_1.0.0.0_Cno").unwrap();

    assert!(Path::new("test/VSCode_1.0.0.0_Cno/package.toml").exists());
}
