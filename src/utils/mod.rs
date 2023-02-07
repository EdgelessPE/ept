use regex::Regex;
use colored::Colorize;

lazy_static! {
    static ref RE: Regex = Regex::new(r"(Debug|Info|Warning|Error)(\(\w+\))?:(.+)").unwrap();
}

pub fn log(msg: String) {
    for cap in RE.captures_iter(&msg){
        if cap.len()!=4{
            break;
        }
        
        let head=cap[1].to_string();
        let head=head.as_str();
        let c_head=match head {
            "Debug"=>{
                if is_debug_mode(){
                    return;
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
            _=>{
                head.white()
            },
        };

        if cap.get(2).is_some() {
            println!("  {}{} {}",c_head,&cap[2].truecolor(100,100,100),&cap[3]);
        }else{
            println!("{} {}",c_head,&cap[3]);
        }
        return;
    }
    println!("{}", msg);
}

pub fn is_debug_mode()->bool{
    envmnt::get_or("DEBUG", "false")==String::from("true")
}

#[test]
fn test_log(){
    envmnt::set("DEBUG","true");

    log("Debug:This is a debug".to_string());
    log("Info:This is a info".to_string());
    log("Warning:This is a warning".to_string());
    log("Error:This is an error".to_string());
    log("Unknown:This is an unknown".to_string());
    log("This is a plain text".to_string());
    
    log("Debug(Log):This is a debug".to_string());
    log("Info(Path):This is a info".to_string());
    log("Warning(Execute):This is a warning".to_string());
    log("Error(Link):This is an error".to_string());
    log("Unknown(unknown):This is an unknown".to_string());
}