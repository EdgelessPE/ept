use anyhow::{anyhow, Result};
use humantime::parse_duration;
use std::{
    fs::{copy, create_dir_all, read_dir},
    time::SystemTime,
};

use crate::{
    p2s,
    types::context::CacheCtx,
    utils::{fs::try_recycle, get_path_cache},
};

pub fn spawn_cache(cx: CacheCtx) -> Result<()> {
    log!("Debug:Spawning cache with cx: {cx:?}");
    let CacheCtx(enabled_cache, at, cached) = cx;
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
    } else {
        log!("Debug:Cache disabled, skip spawning cache");
    }
    Ok(())
}

pub fn restore_cache(cx: CacheCtx, source: &str) -> Result<bool> {
    log!("Debug:Restoring cache with cx: {cx:?}");
    let CacheCtx(enabled_cache, to, cached) = cx;
    if enabled_cache {
        if let Some((cache_path, cache_key)) = cached.clone() {
            let cache_file_path = cache_path.join(&cache_key);
            if cache_file_path.exists() {
                copy(&cache_file_path, &to).map_err(|e: std::io::Error| {
                    anyhow!(
                        "Error:Failed to restore cache from '{}' to '{}' : {e}",
                        p2s!(cache_file_path),
                        p2s!(to)
                    )
                })?;
                log!(
                    "Info:Restored cache form '{}' to '{}'",
                    p2s!(cache_file_path),
                    p2s!(to)
                );
                return Ok(true);
            } else {
                log!(
                    "Debug:Cache not found for '{source}' at '{}'",
                    p2s!(cache_file_path)
                );
            }
        }
    } else {
        log!("Debug:Cache disabled, skip restoring cache");
    }

    Ok(false)
}

pub fn clean_cache(cfg: &crate::types::context::RuntimeContext) -> Result<()> {
    let duration_cfg = parse_duration(&cfg.cfg.local.cache_valid_duration).map_err(|e| anyhow!("Error:Failed to parse config field 'local.cache_valid_duration' as valid time span : '{e}', e.g. '5d' '14m54s'"))?;
    let now = SystemTime::now();
    log!(
        "Debug:Cache valid duration : '{i}'",
        i = &cfg.cfg.local.cache_valid_duration
    );

    let cache_dir = get_path_cache(cfg)?;
    let mut cache_files = Vec::new();
    for entry in read_dir(cache_dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() {
            let metadata = entry.metadata()?;
            let modified = metadata.modified()?;
            if now.duration_since(modified).unwrap() > duration_cfg {
                cache_files.push(path);
            }
        }
    }

    log!(
        "Debug:Found {} cache files to clean : {cache_files:?}",
        cache_files.len()
    );

    for file in cache_files {
        if let Err(e) = try_recycle(file.clone()) {
            log!(
                "Warning:Failed to clean cache file '{}' : {}",
                p2s!(file),
                e
            );
        }
    }

    Ok(())
}
