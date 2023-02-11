use crate::types::Author;
use lazy_static::lazy_static;
use regex::Regex;
use anyhow::{anyhow,Result};

lazy_static! {
    static ref RE: Regex =
        Regex::new(r"([^<]+)\s*(<\s*([\w@\.]+)\s*>)?").unwrap();
}

pub fn parse_author(raw:String)->Result<Author>{
    for cap in RE.captures_iter(&raw) {
        if cap.len()!=4{
            break;
        }
        return Ok(Author{
            name:cap[1].to_string(),
            email:if cap.get(3).is_some() {
                Some(cap[3].to_string())
            }else{
                None
            }
        });
    }

    Err(anyhow!("Error:Can't parse '{}' as valid author",&raw))
}

#[test]
fn test_parse_author(){
    println!("{:?}",parse_author("Cno".to_string()));
    println!("{:?}",parse_author("Cno <dsyourshy@qq.com>".to_string()));
}