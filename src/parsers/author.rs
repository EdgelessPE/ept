use crate::types::author::Author;
use anyhow::{anyhow, Result};
use lazy_static::lazy_static;
use regex::Regex;

lazy_static! {
    static ref RE: Regex = Regex::new(r"([^<\s]+)\s*(<\s*([\w@\.]+)\s*>)?").unwrap();
}

pub fn parse_author(raw: &String) -> Result<Author> {
    if let Some(cap) = RE.captures_iter(raw).next() {
        if cap.len() != 4 {
            return Err(anyhow!("Error:Can't parse '{raw}' as valid author"));
        }
        return Ok(Author {
            name: cap[1].to_string(),
            email: if cap.get(3).is_some() {
                Some(cap[3].to_string())
            } else {
                None
            },
        });
    }

    Err(anyhow!("Error:Can't parse '{raw}' as valid author"))
}

#[test]
fn test_parse_author() {
    assert_eq!(
        Author {
            name: "Cno".to_string(),
            email: None
        },
        parse_author(&"Cno".to_string()).unwrap()
    );
    assert_eq!(
        Author {
            name: "Cno".to_string(),
            email: Some("dsyourshy@qq.com".to_string())
        },
        parse_author(&"Cno <dsyourshy@qq.com>".to_string()).unwrap()
    );
}
