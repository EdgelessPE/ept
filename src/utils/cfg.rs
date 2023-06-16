use std::{
    fs::{create_dir_all, read_to_string, write},
    path::{Path, PathBuf},
    sync::RwLock,
};

use anyhow::{anyhow, Result};
use dirs::home_dir;
use serde::{Deserialize, Serialize};
use toml::{from_str, to_string_pretty, Value};

use crate::{p2s, types::verifiable::Verifiable};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Local {
    pub base: String,
}
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Cfg {
    pub local: Local,
}

const FILE_NAME: &str = "config.toml";

lazy_static! {
    static ref CUR_DIR: PathBuf = Path::new("./").to_path_buf();
    static ref USER_DIR: PathBuf = home_dir().unwrap().join("ept");
    static ref CFG: RwLock<Cfg> = RwLock::new(Cfg::init().unwrap());
}

impl Cfg {
    fn default() -> Self {
        Self {
            local: Local {
                base: p2s!(USER_DIR),
            },
        }
    }
    fn use_which() -> Result<PathBuf> {
        let from = if CUR_DIR.join(FILE_NAME).exists() {
            CUR_DIR.join(FILE_NAME)
        } else {
            let from = USER_DIR.join(FILE_NAME);
            if !from.exists() {
                create_dir_all(USER_DIR.to_str().unwrap()).map_err(|e| {
                    anyhow!("Error:Can't create '{dir}' : {e}", dir = p2s!(USER_DIR),)
                })?;
                let default = Value::try_from(Self::default())?;
                write(from.clone(), to_string_pretty(&default)?).map_err(|e| {
                    anyhow!(
                        "Error:Can't write default config to '{f}' : {e}",
                        f = p2s!(from),
                    )
                })?;
            }
            from
        };
        Ok(from)
    }
    pub fn init() -> Result<Self> {
        let from = Self::use_which()?;
        let text = read_to_string(from.clone())?;
        let cfg: Self = from_str(&text).map_err(|e| {
            anyhow!(
                "Error:Invalid config content, try delete '{f}' : {e}",
                f = p2s!(from),
            )
        })?;

        // 校验
        cfg.verify_self(&"".to_string()).map_err(|e| {
            anyhow!(
                "Error:Invalid config '{f}' : {e}",
                f = p2s!(from)
            )
        })?;

        Ok(cfg)
    }
    pub fn overwrite(&mut self, other: Self) -> Result<()> {
        // 校验
        other.verify_self(&"".to_string()).map_err(|e| {
            anyhow!(
                "Error:Invalid overwrite config : {e}"
            )
        })?;

        // 赋值
        self.local = other.local.clone();

        let from = Self::use_which()?;
        let value = Value::try_from(other)?;
        let text = to_string_pretty(&value)?;
        write(from, &text)?;
        Ok(())
    }
}

impl Verifiable for Cfg {
    fn verify_self(&self, _: &String) -> Result<()> {
        // base 必须为存在的绝对路径
        let base_path = Path::new(&self.local.base);
        if !base_path.is_absolute() {
            return Err(anyhow!("Error:Field 'local.base' should be absolute"));
        }
        if !base_path.exists() {
            return Err(anyhow!("Error:Field 'local.base' doesn't exist"));
        }

        Ok(())
    }
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
