use std::path::Path;

use regex::Regex;
use semver::VersionReq;

use anyhow::{anyhow, Ok, Result};

use crate::utils::{format_path, is_url};

lazy_static! {
    static ref PACKAGE_MATCHER_REGEX: Regex =
        Regex::new(r"^(([^/]+/)?[^/]+/)?[^/@]+(@[\^\*\w\.-]+)?$").unwrap();
}

#[derive(Clone, Debug, PartialEq)]
pub struct PackageMatcher {
    pub name: String,
    pub scope: Option<String>,
    pub mirror: Option<String>,
    pub version_req: Option<VersionReq>,
}

impl PackageMatcher {
    pub fn parse(text: &String, deny_mirror: bool, deny_version_matcher: bool) -> Result<Self> {
        if text.is_empty() {
            return Err(anyhow!("Error:Empty input text"));
        }
        let mut res = PackageMatcher {
            name: text.to_string(),
            scope: None,
            mirror: None,
            version_req: None,
        };
        // 分割 @ 并解析 VersionReq
        let lhs = if text.contains('@') {
            if deny_version_matcher {
                return Err(anyhow!("Error:Version matcher not allowed"));
            }
            let sp: Vec<&str> = text.split('@').collect();
            if sp.len() != 2 {
                return Err(anyhow!(
                    "Error:Invalid package matcher : there can be at most one '@', got {len}",
                    len = sp.len() - 1
                ));
            }
            let t = sp.first().unwrap();
            let str = sp.get(1).unwrap();
            res.version_req =
                Some(VersionReq::parse(str.trim_matches('"')).map_err(|e| {
                    anyhow!("Error:Failed to parse '{str}' as valid VersionReq : {e}")
                })?);
            t.to_string()
        } else {
            text.to_string()
        };

        // 分割 lhs
        let mut sp: Vec<&str> = lhs.split('/').collect();
        if sp.len() > 3 {
            return Err(anyhow!(
                "Error:Invalid package key '{text}', expect pattern '(MIRROR/)(SCOPE/)NAME'"
            ));
        }
        sp.reverse();
        if let Some(name) = sp.first() {
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
}

#[derive(Clone, Debug, PartialEq)]
pub enum PackageInputEnum {
    PackageMatcher(PackageMatcher),
    Url(String),
    LocalPath(String),
}

impl PackageInputEnum {
    pub fn parse(text: String, deny_mirror: bool, deny_version_matcher: bool) -> Result<Self> {
        // 判断是否为 URL
        if is_url(&text) {
            return Ok(PackageInputEnum::Url(text));
        }

        // 如果本地存在该路径，则作为路径处理
        let p = Path::new(&text);
        if p.exists() {
            return Ok(PackageInputEnum::LocalPath(format_path(&text)));
        }

        // 使用正则匹配 PackageMatcher
        if PACKAGE_MATCHER_REGEX.is_match(&text) {
            let m = PackageMatcher::parse(&text, deny_mirror, deny_version_matcher)?;
            return Ok(PackageInputEnum::PackageMatcher(m));
        }

        Err(anyhow!(
            "Error:Failed to parse '{text}' as valid package input"
        ))
    }
}

#[test]
fn test_parse_package_matcher() {
    assert_eq!(
        PackageMatcher::parse(&"VSCode".to_string(), false, false).unwrap(),
        PackageMatcher {
            name: "VSCode".to_string(),
            scope: None,
            mirror: None,
            version_req: None
        }
    );
    assert_eq!(
        PackageMatcher::parse(&"VSCode@1.0.0".to_string(), false, false).unwrap(),
        PackageMatcher {
            name: "VSCode".to_string(),
            scope: None,
            mirror: None,
            version_req: Some(VersionReq::parse("1.0.0").unwrap())
        }
    );
    assert_eq!(
        PackageMatcher::parse(&"Microsoft/VSCode@^1.1.0".to_string(), false, false).unwrap(),
        PackageMatcher {
            name: "VSCode".to_string(),
            scope: Some("Microsoft".to_string()),
            mirror: None,
            version_req: Some(VersionReq::parse("^1.1.0").unwrap())
        }
    );
    assert_eq!(
        PackageMatcher::parse(
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
    assert!(PackageMatcher::parse(&"Official/Microsoft/VSCode".to_string(), true, false).is_err());
    assert!(PackageMatcher::parse(&"VSCode@\">=0.1.0\"".to_string(), false, true).is_err());
}

#[test]
fn test_parse_package_input_enum() {
    assert_eq!(
        PackageInputEnum::parse(
            "https://nep.edgeless.top/static/test.nep".to_string(),
            false,
            false
        )
        .unwrap(),
        PackageInputEnum::Url("https://nep.edgeless.top/static/test.nep".to_string())
    );
    assert_eq!(
        PackageInputEnum::parse(".\\Cargo.lock".to_string(), false, false).unwrap(),
        PackageInputEnum::LocalPath("Cargo.lock".to_string())
    );
    assert_eq!(
        PackageInputEnum::parse("VSCode".to_string(), false, false).unwrap(),
        PackageInputEnum::PackageMatcher(PackageMatcher {
            name: "VSCode".to_string(),
            scope: None,
            mirror: None,
            version_req: None
        })
    );
    assert_eq!(
        PackageInputEnum::parse("VSCode@1.1.4".to_string(), false, false).unwrap(),
        PackageInputEnum::PackageMatcher(PackageMatcher {
            name: "VSCode".to_string(),
            scope: None,
            mirror: None,
            version_req: Some(VersionReq::parse("1.1.4").unwrap())
        })
    );
}
