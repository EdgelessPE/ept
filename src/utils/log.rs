use colored::Colorize;
use console::Term;
use regex::Regex;

use super::is_debug_mode;

lazy_static! {
    static ref RE: Regex =
        Regex::new(r"(Debug|Info|Warning|Error|Success)(\(\w+\))?:(.+)").unwrap();
    static ref TERM: Term = Term::stdout();
}

fn gen_log(msg: String, replace_head: Option<String>) -> Option<String> {
    for cap in RE.captures_iter(&msg) {
        if cap.len() != 4 {
            break;
        }

        let head = replace_head.unwrap_or(cap[1].to_string());
        let head = head.as_str();
        let c_head = match head {
            "Debug" => {
                if is_debug_mode() {
                    return None;
                }
                head.bright_white()
            }
            "Info" => head.bright_blue(),
            "Warning" => head.bright_yellow(),
            "Error" => head.bright_red(),
            "Success" => head.bright_green(),
            _ => head.white(),
        };

        if cap.get(2).is_some() {
            return Some(format!(
                "  {}{} {}",
                c_head,
                &cap[2].truecolor(100, 100, 100),
                &cap[3]
            ));
        } else {
            return Some(format!("{} {}", c_head, &cap[3]));
        }
    }
    return Some(format!("{}", msg));
}

pub fn log(msg: String) {
    let g = gen_log(msg, None);
    if g.is_some() {
        let content=g.unwrap();
        envmnt::set("LAST_LOG", &content);
        TERM.write_line(&content).unwrap();
    }
}

pub fn log_ok_last(msg: String) {
    let g = gen_log(format!("{}   {}", msg, "ok".green()), None);
    if g.is_some() {
        let content=g.unwrap();
        if envmnt::get_or("LAST_LOG","")==content{
            TERM.move_cursor_up(1).unwrap();
            TERM.clear_line().unwrap();
        }
        TERM.write_line(&content).unwrap();
    }
}

#[test]
fn test_log() {
    // envmnt::set("DEBUG","true");

    log("Debug:This is a debug".to_string());
    log("Info:This is a info".to_string());
    log("Warning:This is a warning".to_string());
    log("Error:This is an error".to_string());
    log("Success:This is a success".to_string());
    log("Unknown:This is an unknown".to_string());
    log("This is a plain text".to_string());

    log("Debug(Log):This is a debug".to_string());
    log("Info(Path):This is a info".to_string());
    log("Warning(Execute):This is a warning".to_string());
    log("Error(Link):This is an error".to_string());
    log("Success(Main):This is a success".to_string());
    log("Unknown(unknown):This is an unknown".to_string());
}

#[test]
fn test_log_success_last() {
    log("Info:Preparing...".to_string());

    log(format!("Info:Running remove workflow..."));
    std::thread::sleep(std::time::Duration::from_secs(1));
    log_ok_last(format!("Info:Running remove workflow..."));

    log(format!("Info:Cleaning..."));
    std::thread::sleep(std::time::Duration::from_secs(1));
    log_ok_last(format!("Info:Cleaning..."));

    log(format!("Info:Running setup workflow..."));
    std::thread::sleep(std::time::Duration::from_secs(1));
    log(format!("Warning:Notice this!"));
    std::thread::sleep(std::time::Duration::from_secs(1));
    log_ok_last(format!("Info:Running setup workflow..."));
}