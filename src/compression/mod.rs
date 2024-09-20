mod tar;
mod zstd;

use crate::{log, p2s};

pub use self::tar::{pack_tar, release_tar};
pub use self::zstd::fast_decompress_zstd;
use self::zstd::{compress_zstd, decompress_zstd};
use anyhow::{anyhow, Result};
use std::fs::remove_file;
use std::path::Path;

fn get_temp_tar(zstd_file: &Path) -> String {
    let stem = p2s!(zstd_file.file_stem().unwrap());
    p2s!(zstd_file.with_file_name(stem))
}

#[test]
fn test_get_temp_tar() {
    let res = get_temp_tar(Path::new(
        r"D:\Desktop\Projects\EdgelessPE\ept\examples\VSCode\Pack.tar.zst",
    ));
    assert_eq!(
        res,
        "D:/Desktop/Projects/EdgelessPE/ept/examples/VSCode/Pack.tar".to_string()
    );
}

pub fn compress(source_dir: &String, into_file: &String) -> Result<()> {
    let temp_tar = get_temp_tar(Path::new(into_file));
    pack_tar(source_dir, &temp_tar)
        .map_err(|res| anyhow!("Error:Can't archive '{source_dir}' into '{temp_tar}' : {res}"))?;

    compress_zstd(&temp_tar, into_file)
        .map_err(|res| anyhow!("Error:Can't compress '{temp_tar}' into '{into_file}' : {res}"))?;

    if let Err(e) = remove_file(&temp_tar) {
        log!("Warning:Can't remove temp tar '{temp_tar}' : {e}");
    }

    Ok(())
}

pub fn decompress(source_file: &String, into_dir: &String) -> Result<()> {
    let temp_tar = get_temp_tar(Path::new(source_file));
    decompress_zstd(source_file, &temp_tar).map_err(|res| {
        anyhow!("Error:Can't decompress '{source_file}' into '{temp_tar}' : {res}")
    })?;

    release_tar(&temp_tar, into_dir)
        .map_err(|res| anyhow!("Error:Can't release '{temp_tar}' into '{into_dir}' : {res}"))?;

    if let Err(e) = remove_file(&temp_tar) {
        log!("Warning:Can't remove temp tar '{temp_tar}' : {e}");
    }

    Ok(())
}

#[test]
fn test_compress() {
    crate::utils::test::_ensure_clear_test_dir();
    let p = Path::new("./test/VSCode_1.0.0.0_Cno.tar.zst");
    if p.exists() {
        remove_file(p).unwrap();
    }
    compress(
        &"examples/VSCode".to_string(),
        &"./test/VSCode_1.0.0.0_Cno.tar.zst".to_string(),
    )
    .unwrap();
    assert!(p.exists());
}

#[test]
fn test_decompress() {
    crate::utils::test::_ensure_clear_test_dir();
    if !Path::new("./test/VSCode_1.0.0.0_Cno.tar.zst").exists() {
        test_compress();
    }
    let target = Path::new("test/VSCode_1.0.0.0_Cno");
    if target.exists() {
        crate::utils::fs::try_recycle(target).unwrap();
    }

    decompress(
        &"./test/VSCode_1.0.0.0_Cno.tar.zst".to_string(),
        &"./test/VSCode_1.0.0.0_Cno".to_string(),
    )
    .unwrap();

    assert!(Path::new("test/VSCode_1.0.0.0_Cno/package.toml").exists());
}
