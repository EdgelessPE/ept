use anyhow::{anyhow, Result};
use std::{
    fs::{copy, create_dir_all},
    path::PathBuf,
};

use crate::p2s;

// （是否启用缓存，源文件，Option<(缓存目录, 缓存 key)>）
pub struct CacheCtx(pub bool, pub PathBuf, pub Option<(PathBuf, String)>);

pub fn spawn_cache(ctx: CacheCtx) -> Result<()> {
    let CacheCtx(enabled_cache, at, cached) = ctx;
    if enabled_cache {
        if let Some((cache_path, cache_key)) = cached {
            if !cache_path.exists() {
                create_dir_all(&cache_path).map_err(|e| {
                    anyhow!(
                        "Error:Failed to create cache directory at '{}' : {e}",
                        p2s!(cache_path)
                    )
                })?;
            }
            let target = cache_path.join(cache_key);
            copy(&at, &target).map_err(|e| {
                anyhow!(
                    "Error:Failed to store cache file from '{}' to '{}' : {e}",
                    p2s!(at),
                    p2s!(target)
                )
            })?;
            log!("Info:Cache stored at '{}'", p2s!(target))
        }
    }
    Ok(())
}
