use std::io::stdin;

use crate::utils::{is_confirm_mode, log};

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

#[test]
fn test_ask_yn() {
    envmnt::set("CONFIRM", "true");
    log("Warning:Please select (y/n)?".to_string());
    let res = ask_yn();
    println!("{}", res);
}
