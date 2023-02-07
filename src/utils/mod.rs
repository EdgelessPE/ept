use regex::Regex;
use colored::Colorize;
use console::Term;

const BACKSPACE: char = 8u8 as char;

lazy_static! {
    static ref RE: Regex = Regex::new(r"(Debug|Info|Warning|Error|Success)(\(\w+\))?:(.+)").unwrap();
    static ref TERM:Term=Term::stdout();
}

fn gen_log(msg: String)->Option<String> {
    for cap in RE.captures_iter(&msg){
        if cap.len()!=4{
            break;
        }
        
        let head=cap[1].to_string();
        let head=head.as_str();
        let c_head=match head {
            "Debug"=>{
                if is_debug_mode(){
                    return None;
                }
                head.bright_white()
            },
            "Info"=>{
                head.bright_blue()
            },
            "Warning"=>{
                head.bright_yellow()
            },
            "Error"=>{
                head.bright_red()
            },
            "Success"=>{
                head.bright_green()
            },
            _=>{
                head.white()
            },
        };

        if cap.get(2).is_some() {
            return Some(format!("  {}{} {}",c_head,&cap[2].truecolor(100,100,100),&cap[3]));
        }else{
            return Some(format!("{} {}",c_head,&cap[3]));
        }
    }
    return Some(format!("{}", msg));
}

pub fn log(msg:String){
    let g=gen_log(msg);
    if g.is_some(){
        TERM.write_line(&g.unwrap()).unwrap();
    }
}

pub fn log_overwrite(msg:String){
    let g=gen_log(msg);
    if g.is_some(){
        TERM.move_cursor_up(1).unwrap();
        TERM.clear_line().unwrap();
        TERM.write_line(&g.unwrap()).unwrap();
    }
}

pub fn is_debug_mode()->bool{
    envmnt::get_or("DEBUG", "false")==String::from("true")
}

#[test]
fn test_log(){
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
fn test_log_overwrite(){
    log("Info:Working...".to_string());
    std::thread::sleep(std::time::Duration::from_secs(1));
    log_overwrite("Success:Done!".to_string());
    std::thread::sleep(std::time::Duration::from_secs(1));
}