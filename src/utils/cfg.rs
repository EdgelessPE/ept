use std::sync::RwLock;

use anyhow::{anyhow, Result};

use crate::types::cfg::Cfg;

lazy_static! {
    static ref CFG: RwLock<Cfg> = RwLock::new(Cfg::init().unwrap());
}

pub fn get_config() -> Cfg {
    CFG.read().unwrap().clone()
}

pub fn set_config(next: Cfg) -> Result<()> {
    CFG.write().unwrap().overwrite(next)
}

#[test]
fn test_config() {
    let mut cfg = get_config();
    println!("{cfg:#?}");
    cfg.local.base = "2333".to_string();
    println!("{cfg:#?}");
    assert!(set_config(cfg).is_err());
}

pub fn get_flags_score(flags: &str) -> Result<i32> {
    let cfg = get_config();
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
