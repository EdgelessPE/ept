use crate::utils::is_confirm_mode;
use colored::Colorize;
use dialoguer::Confirm;
use encoding::all::GBK;
use encoding::{DecoderTrap, Encoding};

fn ask_yn_impl(prompt: String, default_value: bool) -> bool {
    if is_confirm_mode() {
        true
    } else {
        Confirm::new()
            .with_prompt(format!(
                "{:>8} {prompt}",
                if default_value {
                    "Question".truecolor(103, 58, 183)
                } else {
                    "Question".truecolor(255, 87, 34)
                }
            ))
            .default(default_value)
            .interact()
            .unwrap()
    }
}

pub fn ask_yn(prompt: String, default_value: bool) -> bool {
    println!("prompt: {prompt}");
    debug_assert!(prompt.as_bytes().first().unwrap().is_ascii_uppercase() && prompt.ends_with('?'));
    ask_yn_impl(prompt, default_value)
}

pub fn ask_yn_in_step(step_name: &str, prompt: String, default_value: bool) -> bool {
    println!("prompt({step_name}): {prompt}");
    debug_assert!(prompt.as_bytes().first().unwrap().is_ascii_uppercase() && prompt.ends_with('?'));
    ask_yn_impl(
        format!("{:<9} {prompt}", step_name.truecolor(100, 100, 100)),
        default_value,
    )
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
    assert!(ask_yn("Do you like 玩游戏?".to_string(), true));
}
