use std::str::FromStr;

use crate::types::mirror::{MirrorEptToolchain, MirrorEptToolchainRelease};
use anyhow::{anyhow, Result};
use fs_extra::file::read_to_string;
use semver::Version;
use serde_json::from_str;

use super::{constants::MIRROR_FILE_EPT_TOOLCHAIN, fs::read_sub_dir, get_path_mirror};

// 从本地的第一个镜像缓存中读取工具链信息
pub fn read_local_mirror_ept_toolchain() -> Result<MirrorEptToolchain> {
    let mirror_base = get_path_mirror()?;
    let dir_list = read_sub_dir(&mirror_base)?;
    for mirror_name in dir_list {
        let p = mirror_base
            .join(mirror_name)
            .join(MIRROR_FILE_EPT_TOOLCHAIN);
        if !p.exists() {
            continue;
        }
        let text = read_to_string(&p)?;
        let res: MirrorEptToolchain = from_str(&text)
            .map_err(|e| anyhow!("Error:Invalid ept toolchain content at '{p:?}' : {e}"))?;
        return Ok(res);
    }
    Err(anyhow!(
        "Error:No mirror with 'EPT_TOOLCHAIN' service added"
    ))
}

// 检查当前 ept 是否可更新，返回 （是否有更新，是否跨越鸿沟，当前的最新版本）
pub fn check_has_upgrade(
    current_version: &str,
    res: MirrorEptToolchain,
) -> Result<(bool, bool, MirrorEptToolchainRelease)> {
    // 筛选出最新的版本
    let mut latest_version = Version::new(0, 0, 0);
    let mut latest_release = None;
    for release in res.releases {
        if Version::from_str(&release.version)? > latest_version {
            latest_version = Version::from_str(&release.version)?;
            latest_release = Some(release);
        }
    }
    if let Some(latest_release) = latest_release {
        let current_ins = Version::from_str(current_version)?;
        let mut is_cross_wid_gap = false;
        // 判断是否跨域了鸿沟，算法：当前版本 < 鸿沟 <= 最新版本
        for wg_version in res.update.wild_gaps {
            let wg_ins = Version::from_str(&wg_version)?;
            if current_ins < wg_ins && wg_ins <= latest_version {
                is_cross_wid_gap = true;
                break;
            }
        }
        Ok((
            current_ins < latest_version,
            is_cross_wid_gap,
            latest_release,
        ))
    } else {
        Err(anyhow!("Error:No ept toolchain release found"))
    }
}

#[test]
fn test_check_has_upgrade() {
    use crate::types::mirror::{MirrorEptToolchainRelease, MirrorEptToolchainUpdate};
    // 没有更新
    let res = MirrorEptToolchain {
        update: MirrorEptToolchainUpdate {
            wild_gaps: Vec::new(),
        },
        releases: vec![
            MirrorEptToolchainRelease {
                name: "ept_0.2.2.zip".to_string(),
                version: "0.2.2".to_string(),
                url: String::new(),
                timestamp: 0,
                size: 0,
            },
            MirrorEptToolchainRelease {
                name: "ept_0.2.1.zip".to_string(),
                version: "0.2.1".to_string(),
                url: String::new(),
                timestamp: 0,
                size: 0,
            },
        ],
    };
    assert_eq!(
        check_has_upgrade("0.2.2", res.clone()).unwrap(),
        (
            false,
            true,
            MirrorEptToolchainRelease {
                name: "ept_0.2.2.zip".to_string(),
                version: "0.2.2".to_string(),
                url: String::new(),
                timestamp: 0,
                size: 0,
            }
        )
    );

    // 有可用的自更新
    assert_eq!(
        check_has_upgrade("0.2.1", res).unwrap(),
        (
            true,
            true,
            MirrorEptToolchainRelease {
                name: "ept_0.2.2.zip".to_string(),
                version: "0.2.2".to_string(),
                url: String::new(),
                timestamp: 0,
                size: 0,
            }
        )
    );

    // 当前版本是最后一个鸿沟版本，没有更新
    let res = MirrorEptToolchain {
        update: MirrorEptToolchainUpdate {
            wild_gaps: vec!["0.2.1".to_string(), "1.0.0".to_string()],
        },
        releases: vec![
            MirrorEptToolchainRelease {
                name: "ept_0.2.2.zip".to_string(),
                version: "0.2.2".to_string(),
                url: String::new(),
                timestamp: 0,
                size: 0,
            },
            MirrorEptToolchainRelease {
                name: "ept_1.0.0.zip".to_string(),
                version: "1.0.0".to_string(),
                url: String::new(),
                timestamp: 0,
                size: 0,
            },
            MirrorEptToolchainRelease {
                name: "ept_0.2.1.zip".to_string(),
                version: "0.2.1".to_string(),
                url: String::new(),
                timestamp: 0,
                size: 0,
            },
        ],
    };
    assert_eq!(
        check_has_upgrade("1.0.0", res.clone()).unwrap(),
        (
            false,
            true,
            MirrorEptToolchainRelease {
                name: "ept_1.0.0.zip".to_string(),
                version: "1.0.0".to_string(),
                url: String::new(),
                timestamp: 0,
                size: 0,
            }
        )
    );

    // 有更新且更新会跨域鸿沟
    assert_eq!(
        check_has_upgrade("0.2.2", res.clone()).unwrap(),
        (
            true,
            false,
            MirrorEptToolchainRelease {
                name: "ept_1.0.0.zip".to_string(),
                version: "1.0.0".to_string(),
                url: String::new(),
                timestamp: 0,
                size: 0,
            }
        )
    );
}
