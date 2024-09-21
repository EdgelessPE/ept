use crate::utils::fmt_print::{fmt_log, fmt_log_in_step};
use crate::utils::is_confirm_mode;
use anyhow::anyhow;
use colored::{ColoredString, Colorize};
use dialoguer::Confirm;
use encoding::all::GBK;
use encoding::{DecoderTrap, Encoding};

fn get_question_head(default_value: bool) -> ColoredString {
    if default_value {
        "Question".truecolor(103, 58, 183)
    } else {
        "Question".truecolor(255, 87, 34)
    }
}

fn ask_yn_impl(prompt: String, default_value: bool) -> bool {
    if is_confirm_mode() {
        log!("{prompt} (confirmed)");
        true
    } else {
        Confirm::new()
            .with_prompt(&prompt)
            .default(default_value)
            .interact()
            .map_err(|e| anyhow!("Error:Failed to ask yn question '{prompt}' : {e}"))
            .unwrap()
    }
}

pub fn ask_yn(prompt: String, default_value: bool) -> bool {
    debug_assert!(prompt.as_bytes().first().unwrap().is_ascii_uppercase() && prompt.ends_with('?'));
    ask_yn_impl(
        fmt_log(get_question_head(default_value), &prompt),
        default_value,
    )
}

pub fn ask_yn_in_step(step_name: &str, prompt: String, default_value: bool) -> bool {
    debug_assert!(prompt.as_bytes().first().unwrap().is_ascii_uppercase() && prompt.ends_with('?'));
    ask_yn_impl(
        fmt_log_in_step(step_name, get_question_head(default_value), &prompt),
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
    use crate::utils::flags::{set_flag, Flag};
    set_flag(Flag::Confirm, true);
    assert!(ask_yn("Do you like what you see😘?".to_string(), true));
    assert!(ask_yn_in_step(
        "Step",
        "Do you like 玩游戏♂?".to_string(),
        false
    ));
}
