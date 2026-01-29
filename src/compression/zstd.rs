use std::{cmp::max, fs::File};

use anyhow::{anyhow, Result};
use zstd::stream::{copy_decode, copy_encode};

pub fn compress_zstd(source: &str, into: &str) -> Result<()> {
    let source_file = File::open(source)?;
    let into_file = File::create(into)?;
    copy_encode(source_file, into_file, 0)?;
    Ok(())
}

pub fn decompress_zstd(source: &str, into: &str) -> Result<()> {
    let source_file = File::open(source)?;
    let into_file = File::create(into)?;
    copy_decode(source_file, into_file)?;
    Ok(())
}

pub fn fast_decompress_zstd(raw: &[u8]) -> Result<Vec<u8>> {
    // 首先尝试将目标缓存配置为 *6，对应 16% 的压缩率，理论上能应付 95% 的包
    let initial_capacity = max(raw.len() * 6, 1024 * 1024);
    if let Ok(res) = zstd::bulk::decompress(raw, initial_capacity) {
        return Ok(res);
    }

    // 10 倍兜底
    let fallback_capacity = max(raw.len() * 10, 1024 * 1024);
    zstd::bulk::decompress(raw, fallback_capacity)
        .map_err(|e| anyhow!("Error:Failed to fast decompress : {e}"))
}

#[test]
fn test_compress_zstd() {
    crate::utils::test::_ensure_clear_test_dir();
    use std::path::Path;
    let p = Path::new("./test/package.toml.zst");
    if p.exists() {
        std::fs::remove_file(p).unwrap();
    }
    compress_zstd("examples/VSCode/package.toml", "./test/package.toml.zst").unwrap();
    assert!(p.exists());
}

#[test]
fn test_decompress_zstd() {
    crate::utils::test::_ensure_clear_test_dir();
    use std::path::Path;
    if !Path::new("./test/package.toml.zst").exists() {
        test_compress_zstd();
    }
    let target = Path::new("test/package.toml");
    if target.exists() {
        std::fs::remove_file(target).unwrap();
    }

    decompress_zstd("./test/package.toml.zst", "test/package.toml").unwrap();

    assert!(target.exists());
}
