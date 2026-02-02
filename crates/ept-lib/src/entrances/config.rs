use std::path::Path;

use crate::types::context::RuntimeContext;
use crate::{p2s, types::cfg::Cfg};
use anyhow::{anyhow, Error, Result};
use toml::Value;

// 返回（key 指向的 value，整个 Cfg）
fn get_toml_value(ctx: &RuntimeContext, table: &str, key: &str) -> Result<(Value, Value)> {
    // 序列化为 toml 对象
    let toml = Value::try_from(ctx.cfg.clone())?;
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

pub fn config_set(ctx: &RuntimeContext, table: &str, key: &str, value: &str) -> Result<()> {
    // 错误处理闭包
    let err_wrapper =
        |e: Error| anyhow!("Error:Failed to set value of '${key}' as '${value}' : ${e}");
    // 拿到这个值研究一下类型
    let (val, mut cfg) = get_toml_value(ctx, table, key).map_err(err_wrapper)?;
    let table = cfg.get_mut(table).unwrap();
    match val {
        Value::String(_) => {
            table[key] = Value::String(value.to_owned());
        }
        // Value::Boolean(_) => {
        //     let bool_value = value
        //         .parse()
        //         .map_err(|_| anyhow!("Can't parse '${value}' as valid bool value"))
        //         .map_err(err_wrapper)?;
        //     table[key] = Value::Boolean(bool_value);
        // }
        // Value::Integer(_) => {
        //     let int_value = value
        //         .parse()
        //         .map_err(|_| anyhow!("Can't parse '${value}' as valid integer value"))
        //         .map_err(err_wrapper)?;
        //     table[key] = Value::Integer(int_value);
        // }
        // Value::Float(_) => {
        //     let float_value = value
        //         .parse()
        //         .map_err(|_| anyhow!("Can't parse '${value}' as valid float value"))
        //         .map_err(err_wrapper)?;
        //     table[key] = Value::Float(float_value);
        // }
        _ => {
            return Err(err_wrapper(anyhow!("This type is not supported for cli configuration, modify the configuration file manually")));
        }
    }

    // 写回
    // 从 toml Value 反序列化回 Cfg
    let updated_cfg_ser: Cfg = cfg.try_into().map_err(|e| {
        anyhow!("Error:Failed to convert modified config to valid config struct : {e}")
    })?;
    let updated_cfg = updated_cfg_ser;
    Cfg::overwrite(updated_cfg)?;

    Ok(())
}

pub fn config_get(ctx: &RuntimeContext, table: &str, key: &str) -> Result<String> {
    let (val, _) = get_toml_value(ctx, table, key)?;

    let str = val
        .as_str()
        .ok_or(anyhow!(
            "Error:Failed to convert value of key '{key}' in table '{table}' to string"
        ))?
        .to_string();

    Ok(str)
}

pub fn config_list(runtime_ctx: &RuntimeContext) -> Result<String> {
    let cfg = &runtime_ctx.cfg;
    Ok(format!("{cfg:#?}"))
}

pub fn config_init(ctx: &RuntimeContext) -> Result<String> {
    let file_path = config_which()?;
    if Path::new(&file_path).exists()
        && !ctx.interaction().ask_yn(
            &format!("Config file already exists at '{file_path}', overwrite it?"),
            false,
        )
    {
        return Err(anyhow!("Error:Operation cancelled by user"));
    }
    Cfg::overwrite(ctx.cfg.clone())?;
    Ok(file_path)
}

pub fn config_which() -> Result<String> {
    let which = Cfg::use_which(false)?;
    Ok(p2s!(which))
}

#[test]
fn test_config() {
    use crate::types::cfg::FILE_NAME;
    use crate::utils::test::_default_test_cfg;
    use std::{fs, path::Path};

    let config = _default_test_cfg();

    // 校对函数，同时检查 API 返回和本地文件
    fn checker(answer: Cfg) {
        let toml = fs::read_to_string(FILE_NAME).unwrap();
        let file_cfg_ser: Cfg = toml::from_str(&toml).unwrap();
        let answer_ser: Cfg = answer.into();
        assert_eq!(file_cfg_ser, answer_ser);
    }

    // 先保存当前目录下 eptrc.toml 的现场
    let scene_opt = if Path::new(FILE_NAME).exists() {
        Some(fs::read_to_string(FILE_NAME).unwrap())
    } else {
        // 如果没有必须新建一个，不然默认会在用户目录里面新建配置文件
        let mut default_cfg = _default_test_cfg();
        default_cfg.cfg.local.base = "C:/Users/Public/Videos".to_string();
        let text = toml::to_string_pretty(&default_cfg.cfg).unwrap();
        fs::write(FILE_NAME, text).unwrap();
        None
    };

    // 拿到答案
    let answer_cfg_init = _default_test_cfg();

    // 测试初始化
    config_init(&config).unwrap();
    checker(answer_cfg_init.cfg.clone());

    // 测试 set
    let mut new_cfg = answer_cfg_init.clone();
    let new_base = "C:/Users/Public/Music".to_string();
    new_cfg.cfg.local.base.clone_from(&new_base);
    assert!(config_set(&config, "local", "base", "114514").is_err());
    config_set(&config, "local", "base", &new_base).unwrap();

    // 测试 get
    let get_base = config_get(&config, "local", "base").unwrap();
    assert_eq!(get_base, new_base);

    // 测试 list
    assert_eq!(config_list(&config).unwrap(), format!("{:#?}", new_cfg.cfg));

    // 测试 which
    assert_eq!(config_which().unwrap(), FILE_NAME.to_string());

    // 还原现场
    if let Some(text) = scene_opt {
        // 需要手动重置一次全局 Cfg，否则之后的测试无法正确进行
        let cfg: Cfg = toml::from_str(&text).unwrap();
        Cfg::overwrite(cfg).unwrap();
    } else {
        fs::remove_file(FILE_NAME).unwrap();
    }
}
