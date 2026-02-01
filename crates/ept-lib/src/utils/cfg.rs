use anyhow::{anyhow, Result};

use crate::types::cfg::{Cfg, PreferenceEnum};

use super::arch::SysArch;

pub fn get_flags_score(flags: &str, cfg: &Cfg) -> Result<i32> {
    let mut score = 0;
    for c in flags.chars() {
        let e = match c {
            //- ARM64
            'A' => {
                if SysArch::get_current_arch()? == SysArch::ARM64 {
                    &PreferenceEnum::HighPriority
                } else {
                    &PreferenceEnum::Forbidden
                }
            }
            //- Expandable
            'E' => &cfg.preference.expandable,
            //- Installer
            'I' => &cfg.preference.installer,
            //- Portable
            'P' => &cfg.preference.portable,
            _ => {
                return Err(anyhow!("Error:Invalid flag : '{c}'"));
            }
        };
        let s: i32 = e.to_owned().into();
        score += s;
    }

    Ok(score)
}

#[test]
fn test_get_flags_score() {
    use crate::types::cfg::PreferenceEnum;
    use crate::utils::test::_default_test_cfg;

    let cfg_bak = _default_test_cfg();

    let getter = |i: PreferenceEnum, p: PreferenceEnum, e: PreferenceEnum| {
        let mut cfg = cfg_bak.clone();
        cfg.preference.installer = i;
        cfg.preference.portable = p;
        cfg.preference.expandable = e;
        cfg
    };

    // 初始偏好
    let cfg = getter(
        PreferenceEnum::LowPriority,
        PreferenceEnum::HighPriority,
        PreferenceEnum::HighPriority,
    );
    assert_eq!(get_flags_score("I", &cfg).unwrap(), 2);
    assert_eq!(get_flags_score("IE", &cfg).unwrap(), 18);
    assert_eq!(get_flags_score("P", &cfg).unwrap(), 16);
    assert_eq!(get_flags_score("EP", &cfg).unwrap(), 32);

    // scope 型偏好
    let cfg = getter(
        PreferenceEnum::Forbidden,
        PreferenceEnum::HighPriority,
        PreferenceEnum::HighPriority,
    );
    assert_eq!(get_flags_score("I", &cfg).unwrap(), -1024);
    assert_eq!(get_flags_score("IE", &cfg).unwrap(), -1008);
    assert_eq!(get_flags_score("P", &cfg).unwrap(), 16);
    assert_eq!(get_flags_score("EP", &cfg).unwrap(), 32);

    // 仅完整安装偏好
    let cfg = getter(
        PreferenceEnum::HighPriority,
        PreferenceEnum::Forbidden,
        PreferenceEnum::LowPriority,
    );
    assert_eq!(get_flags_score("I", &cfg).unwrap(), 16);
    assert_eq!(get_flags_score("IE", &cfg).unwrap(), 18);
    assert_eq!(get_flags_score("P", &cfg).unwrap(), -1024);
    assert_eq!(get_flags_score("EP", &cfg).unwrap(), -1022);
}
