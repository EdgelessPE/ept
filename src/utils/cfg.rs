use std::{
    fs::{create_dir_all, write},
    path::{Path, PathBuf},
    sync::RwLock,
};

use anyhow::{anyhow, Result};
use config::Config;
use dirs::home_dir;
use humantime::parse_duration;
use serde::{Deserialize, Serialize};
use toml::{to_string_pretty, Value};

use crate::{p2s, types::verifiable::Verifiable};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Local {
    pub base: String,
}
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Online {
    pub mirror_update_interval: String,
}
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Cfg {
    pub local: Local,
    pub online: Online,
}

const FILE_NAME: &str = "config.toml";

lazy_static! {
    static ref CUR_DIR: PathBuf = Path::new("./").to_path_buf();
    static ref USER_DIR: PathBuf = home_dir().unwrap().join("ept");
    static ref CFG: RwLock<Cfg> = RwLock::new(Cfg::init().unwrap());
}

impl Cfg {
    pub fn default() -> Self {
        Self {
            local: Local {
                base: p2s!(USER_DIR),
            },
            online: Online {
                mirror_update_interval: "1d".to_string(),
            },
        }
    }
    pub fn use_which() -> Result<PathBuf> {
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
        log!("Debug:Use config at '{f}'", f = p2s!(from));
        Ok(from)
    }
    pub fn init() -> Result<Self> {
        let from = Self::use_which()?;
        let f = p2s!(from);
        let default_val = Value::try_from(Self::default()).unwrap();
        let settings = Config::builder()
            .add_source(config::File::from_str(
                &to_string_pretty(&default_val).unwrap(),
                config::FileFormat::Toml,
            ))
            .add_source(config::File::with_name(&f))
            .add_source(config::Environment::with_prefix("EPT"))
            .build()
            .map_err(|e| anyhow!("Error:Failed to build config with located config '{f}' : {e}"))?;
        let cfg: Self = settings.try_deserialize().map_err(|e| {
            anyhow!(
                "Error:Invalid config content, try delete '{f}' : {e}",
                f = p2s!(from),
            )
        })?;

        // 校验
        cfg.verify_self(&"".to_string())
            .map_err(|e| anyhow!("Error:Invalid config '{f}' : {e}", f = p2s!(from)))?;

        Ok(cfg)
    }
    pub fn overwrite(&mut self, other: Self) -> Result<()> {
        // 校验
        other
            .verify_self(&"".to_string())
            .map_err(|e| anyhow!("Error:Invalid overwrite config : {e}"))?;

        // 赋值
        self.local = other.local.clone();

        let from = Self::use_which()?;
        let value = Value::try_from(other)?;
        let text = to_string_pretty(&value)?;
        write(from, text)?;
        Ok(())
    }
}

impl Verifiable for Cfg {
    fn verify_self(&self, _: &String) -> Result<()> {
        // base 必须为存在的绝对路径
        let base_path = Path::new(&self.local.base);
        if !base_path.is_absolute() {
            return Err(anyhow!(
                "Error:Field 'local.base' should be absolute path, got '{base}'",
                base = self.local.base
            ));
        }
        if !base_path.exists() {
            return Err(anyhow!(
                "Error:Field 'local.base' doesn't exist : '{base}'",
                base = self.local.base
            ));
        }

        // mirror_update_interval 可解析
        parse_duration(&self.online.mirror_update_interval).map_err(|e| anyhow!("Error:Failed to parse field 'online.mirror_update_interval' as valid time span : '{e}', e.g. '5d' '14m54s'"))?;

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
