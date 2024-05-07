use crate::utils::is_confirm_mode;
use encoding::all::GBK;
use encoding::{DecoderTrap, Encoding};
use std::io::stdin;

pub fn ask_yn() -> bool {
    if is_confirm_mode() {
        true
    } else {
        let mut input = String::new();
        let term_in = stdin();
        term_in.read_line(&mut input).unwrap();
        &input[0..1] == "y"
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
    log!("Warning:Please select? (y/n)");
    let res = ask_yn();
    assert!(res);
}
