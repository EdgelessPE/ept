use std::io::stdin;
use std::str::from_utf8;

use crate::utils::is_confirm_mode;

pub fn ask_yn() -> bool {
    if is_confirm_mode() {
        return true;
    } else {
        let mut input = String::new();
        let term_in = stdin();
        term_in.read_line(&mut input).unwrap();
        if &input[0..1] == "y" {
            true
        } else {
            false
        }
    }
}

pub fn read_console(v: Vec<u8>) -> String {
    let msg_res = from_utf8(&v);
    if msg_res.is_err() {
        log!("Warning(Execute):Console output can't be parsed with utf8");
        String::new()
    } else {
        msg_res.unwrap().to_string()
    }
}

#[test]
fn test_ask_yn() {
    envmnt::set("CONFIRM", "true");
    log!("Warning:Please select? (y/n)");
    // let res = ask_yn();
    // println!("{res}");
}
