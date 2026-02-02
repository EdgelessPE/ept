use crate::types::context::RuntimeContext;
use encoding::all::GBK;
use encoding::{DecoderTrap, Encoding};

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
pub fn write_windows_terminal_status(ctx: &RuntimeContext, status: u8) {
    if ctx.cfg.interaction.enable_windows_terminal_status {
        println!("\x1b]9;4;{status};0\x07");
    }
}
