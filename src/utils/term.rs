use std::io::stdin;

use crate::utils::log;

pub fn ask_yn()->bool{
    let mut input=String::new();
    let term_in=stdin();
    term_in.read_line(&mut input).unwrap();
    if &input[0..1]=="y"{
        true
    }else{
        false
    }
}

#[test]
fn test_ask_yn(){
    log("Warning:Please select (y/n)?".to_string());
    let res=ask_yn();
    println!("{}",res);
}