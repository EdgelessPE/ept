use anyhow::Result;
use std::fs::{create_dir_all, remove_dir_all, File};
use std::path::Path;
use tar::{Archive, Builder};

pub fn release_tar(source: &String, into: &String) -> Result<()> {
    let file = File::open(source)?;
    let mut archive = Archive::new(file);

    // 覆盖解压
    let p = Path::new(into);
    if p.exists() {
        remove_dir_all(into)?;
    }
    create_dir_all(into)?;

    archive.unpack(into)?;
    Ok(())
}

pub fn pack_tar(source: &String, store_at: &String) -> Result<()> {
    let file = File::create(store_at)?;
    let mut archive = Builder::new(file);
    archive.append_dir_all(".", source)?;
    archive.finish()?;
    Ok(())
}

#[test]
fn test_pack_tar() {
    let p = Path::new("./test/VSCode_1.0.0.0_Cno.tar");
    if p.exists() {
        crate::compression::remove_file(p).unwrap();
    }
    pack_tar(
        &"examples/VSCode".to_string(),
        &"./test/VSCode_1.0.0.0_Cno.tar".to_string(),
    )
    .unwrap();
    assert!(p.exists());
}

#[test]
fn test_release_tar() {
    if !Path::new("./test/VSCode_1.0.0.0_Cno.tar").exists() {
        test_pack_tar();
    }

    release_tar(
        &"./test/VSCode_1.0.0.0_Cno.tar".to_string(),
        &"./test/VSCode_1.0.0.0_Cno".to_string(),
    )
    .unwrap();

    assert!(Path::new("test/VSCode_1.0.0.0_Cno/package.toml").exists());

    // 测试覆盖
    use crate::utils::try_recycle;
    try_recycle("test/VSCode_1.0.0.0_Cno/package.toml").unwrap();
    release_tar(
        &"./test/VSCode_1.0.0.0_Cno.tar".to_string(),
        &"./test/VSCode_1.0.0.0_Cno".to_string(),
    )
    .unwrap();

    assert!(Path::new("test/VSCode_1.0.0.0_Cno/package.toml").exists());
}
