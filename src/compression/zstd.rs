use std::{cmp::max, fs::File};

use anyhow::{anyhow, Result};
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

pub fn fast_decompress_zstd(raw: &Vec<u8>) -> Result<Vec<u8>> {
    zstd::bulk::decompress(raw, max(raw.capacity() * 5, 1024 * 1024))
        .map_err(|e| anyhow!("Error:Failed to fast decompress : {}", e.to_string()))
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
