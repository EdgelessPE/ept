use colored::Colorize;
use console::Term;
use regex::Regex;

use super::{is_debug_mode, is_no_warning_mode};

lazy_static! {
    static ref RE: Regex =
        Regex::new(r"(Debug|Info|Warning|Error|Success)(\(\w+\))?:(.+)").unwrap();
    static ref TERM: Term = Term::stdout();
}

fn gen_log(msg: &String, replace_head: Option<String>) -> Option<String> {
    if let Some(cap) = RE.captures_iter(msg).next() {
        if cap.len() != 4 {
            return Some(msg.clone());
        }

        let head = replace_head.unwrap_or(cap[1].to_string());
        let head = head.as_str();
        if head == "Debug" && !is_debug_mode() {
            return None;
        }
        if head == "Warning" && is_no_warning_mode() {
            return None;
        }
        let c_head = match head {
            "Debug" => head.truecolor(50, 50, 50),
            "Info" => head.bright_blue(),
            "Warning" => head.bright_yellow(),
            "Error" => head.bright_red(),
            "Success" => head.bright_green(),
            _ => head.white(),
        };

        if cap.get(2).is_some() {
            return Some(format!(
                "  {c_head}{s} {m}",
                s = cap[2].truecolor(100, 100, 100),
                m = &cap[3]
            ));
        } else {
            return Some(format!("{c_head} {m}", m = &cap[3]));
        }
    }
    Some(msg.to_string())
}

pub fn fn_log(msg: String) {
    let g = gen_log(&msg, None);
    if let Some(content) = g {
        envmnt::set("LAST_LOG", &msg);
        TERM.write_line(&content).unwrap();
    }
}

pub fn fn_log_ok_last(msg: String) {
    let g = gen_log(&format!("{msg}   {ok}", ok = "ok".green()), None);
    if let Some(content) = g {
        let last_log = envmnt::get_or("LAST_LOG", "");
        if last_log == msg {
            TERM.move_cursor_up(1).unwrap();
            TERM.clear_line().unwrap();
        }
        TERM.write_line(&content).unwrap();
    }
}

#[macro_export]
macro_rules! log {
    ($($x:tt)*) => {
        $crate::utils::log::fn_log(format!($($x)*))
    };
}

#[macro_export]
macro_rules! log_ok_last {
    ($($x:expr),*) => {
        $crate::utils::log::fn_log_ok_last(format!($($x),*))
    };
}

#[test]
fn test_log() {
    // envmnt::set("DEBUG","true");

    fn_log("Debug:This is a debug".to_string());
    fn_log("Info:This is a info".to_string());
    fn_log("Warning:This is a warning".to_string());
    fn_log("Error:This is an error".to_string());
    fn_log("Success:This is a success".to_string());
    fn_log("Unknown:This is an unknown".to_string());
    fn_log("This is a plain text".to_string());

    fn_log("Debug(Log):This is a debug".to_string());
    fn_log("Info(Path):This is a info".to_string());
    fn_log("Warning(Execute):This is a warning".to_string());
    fn_log("Error(Link):This is an error".to_string());
    fn_log("Success(Main):This is a success".to_string());
    fn_log("Unknown(unknown):This is an unknown".to_string());
}

#[test]
fn test_log_success_last() {
    fn_log("Info:Preparing...".to_string());

    fn_log("Info:Running remove workflow...".to_string());
    std::thread::sleep(std::time::Duration::from_secs(1));
    fn_log_ok_last("Info:Running remove workflow...".to_string());

    fn_log("Info:Cleaning...".to_string());
    std::thread::sleep(std::time::Duration::from_secs(1));
    fn_log_ok_last("Info:Cleaning...".to_string());

    fn_log("Info:Running setup workflow...".to_string());
    std::thread::sleep(std::time::Duration::from_secs(1));
    fn_log("Warning:Notice this!".to_string());
    std::thread::sleep(std::time::Duration::from_secs(1));
    fn_log_ok_last("Info:Running setup workflow...".to_string());
}
