use crate::types::matcher::PackageMatcher;
use crate::utils::is_confirm_mode;
use anyhow::{anyhow, Result};
use encoding::all::GBK;
use encoding::{DecoderTrap, Encoding};
use semver::VersionReq;
use std::io::stdin;

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
    // 先尝试使用 GBK 编码转换
    if let Ok(str) = GBK.decode(&v, DecoderTrap::Strict) {
        return str;
    }

    // 宽松 UTF-8 兜底
    String::from_utf8_lossy(&v).to_string()
}

pub fn parse_package_matcher(
    text: &String,
    deny_mirror: bool,
    deny_version_matcher: bool,
) -> Result<PackageMatcher> {
    if text.len() == 0 {
        return Err(anyhow!("Error:Empty input text"));
    }
    let mut res = PackageMatcher {
        name: text.to_string(),
        scope: None,
        mirror: None,
        version_req: None,
    };
    // 分割 @ 并解析 VersionReq
    let lhs = if text.contains("@") {
        if deny_version_matcher {
            return Err(anyhow!("Error:Version matcher not allowed"));
        }
        let sp: Vec<&str> = text.split("@").collect();
        if sp.len() != 2 {
            return Err(anyhow!(
                "Error:Invalid package matcher : there can be at most one '@', got {len}",
                len = sp.len() - 1
            ));
        }
        let t = sp.get(0).unwrap();
        let str = sp.get(1).unwrap();
        res.version_req = Some(
            VersionReq::parse(str.trim_matches('"'))
                .map_err(|e| anyhow!("Error:Failed to parse '{str}' as valid VersionReq : {e}"))?,
        );
        t.to_string()
    } else {
        text.to_string()
    };

    // 分割 lhs
    let mut sp: Vec<&str> = lhs.split("/").collect();
    if sp.len() > 3 {
        return Err(anyhow!(
            "Error:Invalid package key '{text}', expect pattern '(MIRROR/)(SCOPE/)NAME'"
        ));
    }
    sp.reverse();
    if let Some(name) = sp.get(0) {
        res.name = name.to_string()
    }
    if let Some(scope) = sp.get(1) {
        res.scope = Some(scope.to_string())
    }
    if let Some(mirror) = sp.get(2) {
        if deny_mirror {
            return Err(anyhow!("Error:Mirror specifier not allowed"));
        }
        res.mirror = Some(mirror.to_string())
    }

    Ok(res)
}

#[test]
fn test_ask_yn() {
    envmnt::set("CONFIRM", "true");
    log!("Warning:Please select? (y/n)");
    let res = ask_yn();
    assert!(res);
}

#[test]
fn test_parse_package_matcher() {
    assert_eq!(
        parse_package_matcher(&"VSCode".to_string(), false, false).unwrap(),
        PackageMatcher {
            name: "VSCode".to_string(),
            scope: None,
            mirror: None,
            version_req: None
        }
    );
    assert_eq!(
        parse_package_matcher(&"VSCode@1.0.0".to_string(), false, false).unwrap(),
        PackageMatcher {
            name: "VSCode".to_string(),
            scope: None,
            mirror: None,
            version_req: Some(VersionReq::parse("1.0.0").unwrap())
        }
    );
    assert_eq!(
        parse_package_matcher(&"Microsoft/VSCode@^1.1.0".to_string(), false, false).unwrap(),
        PackageMatcher {
            name: "VSCode".to_string(),
            scope: Some("Microsoft".to_string()),
            mirror: None,
            version_req: Some(VersionReq::parse("^1.1.0").unwrap())
        }
    );
    assert_eq!(
        parse_package_matcher(
            &"Official/Microsoft/VSCode@\">=0.1.0\"".to_string(),
            false,
            false
        )
        .unwrap(),
        PackageMatcher {
            name: "VSCode".to_string(),
            scope: Some("Microsoft".to_string()),
            mirror: Some("Official".to_string()),
            version_req: Some(VersionReq::parse(">=0.1.0").unwrap())
        }
    );

    // 测试 deny
    assert!(parse_package_matcher(&"Official/Microsoft/VSCode".to_string(), true, false).is_err());
    assert!(parse_package_matcher(&"VSCode@\">=0.1.0\"".to_string(), false, true).is_err());
}
