use std::sync::RwLock;

use anyhow::Result;

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
