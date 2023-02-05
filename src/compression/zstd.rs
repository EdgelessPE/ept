use std::fs::File;

use anyhow::Result;
use zstd::stream::{copy_decode, copy_encode};

pub fn compress_zstd(source: String, into: String) -> Result<()> {
    let source_file = File::open(&source)?;
    let into_file = File::create(&into)?;
    copy_encode(source_file, into_file, 0)?;
    Ok(())
}

pub fn decompress_zstd(source: String, into: String) -> Result<()> {
    let source_file = File::open(&source)?;
    let into_file = File::create(&into)?;
    copy_decode(source_file, into_file)?;
    Ok(())
}

#[test]
fn test_compress_zstd() {
    let res = compress_zstd(
        r"D:\Desktop\Projects\EdgelessPE\ept\examples\VSCode\Pack.tar".to_string(),
        r"D:\Desktop\Projects\EdgelessPE\ept\examples\VSCode\Pack.tar.zst".to_string(),
    );
    println!("{:?}", res);
}

#[test]
fn test_decompress_zstd() {
    let res = decompress_zstd(
        r"D:\Desktop\Projects\EdgelessPE\ept\examples\VSCode\Pack.tar.zst".to_string(),
        r"D:\Desktop\Projects\EdgelessPE\ept\examples\VSCode\Pack.tar".to_string(),
    );
    println!("{:?}", res);
}
