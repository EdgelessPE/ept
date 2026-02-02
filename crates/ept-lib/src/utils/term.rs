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

fn ask_yn_impl(
    cfg: &crate::types::context::RuntimeContext,
    prompt: String,
    default_value: bool,
) -> bool {
    if is_confirm_mode(cfg) {
        log!("{prompt} (confirmed)");
        true
    } else {
        write_windows_terminal_status(cfg, 0);
        let res = Confirm::new()
            .with_prompt(&prompt)
            .default(default_value)
            .interact()
            .map_err(|e| anyhow!("Error:Failed to ask yn question '{prompt}' : {e}"))
            .unwrap();
        write_windows_terminal_status(cfg, 3);
        res
    }
}

pub fn ask_yn(
    cfg: &crate::types::context::RuntimeContext,
    prompt: String,
    default_value: bool,
) -> bool {
    debug_assert!(prompt.as_bytes().first().unwrap().is_ascii_uppercase() && prompt.ends_with('?'));
    ask_yn_impl(
        cfg,
        fmt_log(get_question_head(default_value), &prompt),
        default_value,
    )
}

pub fn ask_yn_in_step(
    cfg: &crate::types::context::RuntimeContext,
    step_name: &str,
    prompt: String,
    default_value: bool,
) -> bool {
    debug_assert!(prompt.as_bytes().first().unwrap().is_ascii_uppercase() && prompt.ends_with('?'));
    ask_yn_impl(
        cfg,
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

/*
   0 是默认状态，表示应隐藏进度栏。 在命令完成后使用此状态来清除任何进度状态。
   1：将进度值设置为 <progress>，处于“默认”状态。
   2：将进度值设置为 <progress>，处于“错误”状态。
   3：将任务栏设置为“不确定”状态。 这对于没有进度值但仍在运行的命令非常有用。 此状态忽略 <progress> 值。
   4：将进度值设置为 <progress>，处于“警告”状态。
*/
pub fn write_windows_terminal_status(cfg: &crate::types::context::RuntimeContext, status: u8) {
    if cfg.cfg.interaction.enable_windows_terminal_status {
        println!("\x1b]9;4;{status};0\x07");
    }
}

#[test]
fn test_no_interaction() {
    use crate::types::interaction::{InteractionProvider, NoInteraction};

    let no_interaction = NoInteraction;
    // NoInteraction 会在测试模式下总是返回 true
    assert!(no_interaction.ask_yn("Test prompt?", true));
    assert!(no_interaction.ask_yn("Test prompt?", false));
    assert!(no_interaction.ask_yn_in_step("Step", "Test prompt?", true));
    assert!(no_interaction.ask_yn_in_step("Step", "Test prompt?", false));
}
