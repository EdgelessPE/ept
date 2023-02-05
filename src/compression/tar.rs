use anyhow::Result;
use std::fs::{create_dir_all, remove_dir_all, File};
use std::path::Path;
use tar::{Archive, Builder};

pub fn release_tar(source: String, into: String) -> Result<()> {
    let file = File::open(&source)?;
    let mut archive = Archive::new(file);

    // 覆盖解压
    let p = Path::new(&into);
    if p.exists() {
        remove_dir_all(&into)?;
    }
    create_dir_all(&into)?;

    archive.unpack(&into)?;
    Ok(())
}

pub fn pack_tar(source: String, store_at: String) -> Result<()> {
    let file = File::create(&store_at)?;
    let mut archive = Builder::new(file);
    archive.append_dir_all(".", &source)?;
    archive.finish()?;
    Ok(())
}

#[test]
fn test_release_tar() {
    let res = release_tar(
        r"D:\Desktop\Projects\EdgelessPE\ept\examples\VSCode\VSCode.tar".to_string(),
        "./temp/VSCode_1.0.0.0_Cno/Inner".to_string(),
    );
    println!("{:?}", res);
}

#[test]
fn test_pack_tar() {
    let res = pack_tar(
        "./temp/VSCode_1.0.0.0_Cno/Inner".to_string(),
        r"D:\Desktop\Projects\EdgelessPE\ept\examples\VSCode\Pack.tar".to_string(),
    );
    println!("{:?}", res);
}
