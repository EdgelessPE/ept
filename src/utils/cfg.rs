use std::sync::{Arc, RwLock};

use anyhow::{anyhow, Result};

use crate::types::cfg::Cfg;

lazy_static! {
    static ref CFG: Arc<RwLock<Cfg>> = Arc::new(RwLock::new(Cfg::init().unwrap()));
}

pub fn get_config() -> Cfg {
    CFG.read().unwrap().clone()
}

pub fn set_config(next: Cfg) -> Result<()> {
    Cfg::overwrite(next.clone())?;
    let mut lock = CFG.write().unwrap();
    *lock = next;

    Ok(())
}

#[test]
fn test_config() {
    let mut cfg = get_config();
    println!("{cfg:#?}");
    cfg.local.base = "2333".to_string();
    println!("{cfg:#?}");
    assert!(set_config(cfg).is_err());
}

pub fn get_flags_score(flags: &str, cfg: &Cfg) -> Result<i32> {
    let mut score = 0;
    for c in flags.chars() {
        let e = match c {
            'E' => &cfg.preference.expandable,
            'I' => &cfg.preference.installer,
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
    let cfg_bak = get_config();

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
