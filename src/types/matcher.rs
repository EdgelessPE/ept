use regex::Regex;
use semver::VersionReq;

use anyhow::{anyhow, Result};

use crate::utils::is_url;

lazy_static! {
    static ref PACKAGE_MATCHER_REGEX: Regex =
        Regex::new(r"^(([^/]+/)?[^/]+/)?[^/@]+(@[\w\.-]+)?$").unwrap();
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
            res.version_req =
                Some(VersionReq::parse(str.trim_matches('"')).map_err(|e| {
                    anyhow!("Error:Failed to parse '{str}' as valid VersionReq : {e}")
                })?);
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
}

#[derive(Clone, Debug, PartialEq)]
pub enum PackageInputEnum {
    PackageMatcher(PackageMatcher),
    Url(String),
    LocalPath(String),
}

impl PackageInputEnum {
    pub fn parse(text: String, deny_mirror: bool, deny_version_matcher: bool) -> Result<Self> {
        // 使用正则匹配
        if is_url(&text) {
            return Ok(PackageInputEnum::Url(text));
        }
        if PACKAGE_MATCHER_REGEX.is_match(&text) {
            let m = PackageMatcher::parse(&text, deny_mirror, deny_version_matcher)?;
            return Ok(PackageInputEnum::PackageMatcher(m));
        }

        // 兜底，作为本地路径
        Ok(PackageInputEnum::LocalPath(text))
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
