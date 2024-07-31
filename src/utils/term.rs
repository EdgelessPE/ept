use crate::utils::is_confirm_mode;
use dialoguer::Confirm;
use encoding::all::GBK;
use encoding::{DecoderTrap, Encoding};

use super::log::gen_log;

pub fn ask_yn() -> bool {
    if is_confirm_mode() {
        true
    } else {
        Confirm::new()
            .with_prompt(gen_log(&"Question:Do you like 玩游戏?".to_string(), None).unwrap())
            .default(true)
            .interact()
            .unwrap()
    }
}

pub fn read_console(v: Vec<u8>) -> String {
    // 先尝试使用 GBK 编码转换
    if let Ok(str) = GBK.decode(&v, DecoderTrap::Strict) {
        return str;
    }

    // 宽松 UTF-8 兜底
    String::from_utf8_lossy(&v).to_string()
}

#[test]
fn test_ask_yn() {
    envmnt::set("CONFIRM", "true");
    assert!(ask_yn());
}
