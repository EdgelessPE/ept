use crate::{
    p2s,
    utils::cfg::{get_config, set_config, Cfg},
};
use anyhow::{anyhow, Error, Result};
use toml::Value;

// 返回（key 指向的 value，整个 Cfg）
fn get_toml_value(table: &String, key: &String) -> Result<(Value, Value)> {
    let cfg = get_config();
    // 序列化为 toml 对象
    let toml = toml::Value::try_from(cfg)?;
    // 读 table
    let tab = toml
        .get(table)
        .ok_or(anyhow!("Error:Failed to find table '{table}'"))?;
    // 读 key
    let val = tab.get(key).ok_or(anyhow!(
        "Error:Failed to find key '{key}' in table '{table}'"
    ))?;

    Ok((val.to_owned(), toml))
}

pub fn config_set(table: &String, key: &String, value: &String) -> Result<()> {
    // 错误处理闭包
    let err_wrapper =
        |e: Error| anyhow!("Error:Failed to set value of '${key}' as '${value}' : ${e}");
    // 拿到这个值研究一下类型
    let (val, mut cfg) = get_toml_value(table, key).map_err(err_wrapper)?;
    let table = cfg.get_mut(table).unwrap();
    match val {
        Value::String(_) => {
            table[key] = Value::String(value.to_owned());
        }
        Value::Boolean(_) => {
            let bool_value = value
                .parse()
                .map_err(|_| anyhow!("Can't parse '${value}' as valid bool value"))
                .map_err(err_wrapper)?;
            table[key] = Value::Boolean(bool_value);
        }
        Value::Integer(_) => {
            let int_value = value
                .parse()
                .map_err(|_| anyhow!("Can't parse '${value}' as valid integer value"))
                .map_err(err_wrapper)?;
            table[key] = Value::Integer(int_value);
        }
        Value::Float(_) => {
            let float_value = value
                .parse()
                .map_err(|_| anyhow!("Can't parse '${value}' as valid float value"))
                .map_err(err_wrapper)?;
            table[key] = Value::Float(float_value);
        }
        _ => {
            return Err(err_wrapper(anyhow!("This type is not supported for cli configuration, modify the configuration file manually")));
        }
    }

    // 写回
    let updated_cfg = cfg.try_into().map_err(|e| {
        anyhow!("Error:Failed to convert modified config to valid config struct : {e}")
    })?;
    set_config(updated_cfg)?;

    Ok(())
}

pub fn config_get(table: &String, key: &String) -> Result<String> {
    let (val, _) = get_toml_value(table, key)?;

    let str = val
        .as_str()
        .ok_or(anyhow!(
            "Error:Failed to convert value of key '{key}' in table '{table}' to string"
        ))?
        .to_string();

    Ok(str)
}

pub fn config_list() -> Result<String> {
    let cfg = get_config();
    Ok(format!("{cfg:#?}"))
}

pub fn config_init() -> Result<String> {
    let init_cfg = Cfg::default();
    set_config(init_cfg)?;
    config_which()
}

pub fn config_which() -> Result<String> {
    let which = Cfg::use_which()?;
    Ok(p2s!(which))
}

#[test]
fn test_config() {
    use std::{fs, path::Path};
    // 校对函数，同时检查 API 返回和本地文件
    fn checker(answer: Cfg) {
        let cfg = get_config();
        assert_eq!(answer, cfg);
        let toml = fs::read_to_string("config.toml").unwrap();
        let file_cfg: Cfg = toml::from_str(&toml).unwrap();
        assert_eq!(file_cfg, answer);
    }

    // 先保存当前目录下 config.toml 的现场
    let scene_opt = if Path::new("config.toml").exists() {
        Some(fs::read_to_string("config.toml").unwrap())
    } else {
        // 如果没有必须新建一个，不然默认会在用户目录里面新建配置文件
        let mut default_cfg = Cfg::default();
        default_cfg.local.base = "C:/Users/Public/Videos".to_string();
        let text = toml::to_string_pretty(&default_cfg).unwrap();
        std::fs::write("config.toml", text).unwrap();
        None
    };

    // 拿到答案
    let answer_cfg_init = Cfg::default();

    // 测试初始化
    config_init().unwrap();
    checker(answer_cfg_init.clone());

    // 测试 set
    let mut new_cfg = answer_cfg_init.clone();
    let new_base = "C:/Users/Public/Music".to_string();
    new_cfg.local.base.clone_from(&new_base);
    assert!(config_set(
        &"local".to_string(),
        &"base".to_string(),
        &"114514".to_string()
    )
    .is_err());
    config_set(&"local".to_string(), &"base".to_string(), &new_base).unwrap();
    checker(new_cfg.clone());

    // 测试 get
    let get_base = config_get(&"local".to_string(), &"base".to_string()).unwrap();
    assert_eq!(get_base, new_base);

    // 测试 list
    assert_eq!(config_list().unwrap(), format!("{new_cfg:#?}"));

    // 测试 which
    assert_eq!(config_which().unwrap(), "config.toml".to_string());

    // 还原现场
    if let Some(text) = scene_opt {
        // 需要手动重置一次全局 Cfg，否则之后的测试无法正确进行
        let cfg: Cfg = toml::from_str(&text).unwrap();
        set_config(cfg).unwrap();
    } else {
        fs::remove_file("config.toml").unwrap();
    }
}
