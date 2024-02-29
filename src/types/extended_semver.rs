use anyhow::{anyhow, Error, Result};
use semver::{BuildMetadata, Prerelease};
use std::cmp::Ordering::{Equal, Greater, Less};
use std::fmt;
use std::{cmp::Ordering, str::FromStr};

fn split_pre_build(raw: &String) -> (String, String) {
    // 分割 -
    let sp: Vec<&str> = raw.split("-").collect();
    let clear = sp[0].to_string();

    // 分割 +
    let sp: Vec<&str> = clear.split("+").collect();
    let clear = sp[0].to_string();

    // 根据 clear 长度拆出 pre 和 build 部分
    let pre_build = &raw[clear.len()..];

    (clear, pre_build.to_string())
}

#[test]
fn test_split_pre_build() {
    assert_eq!(
        split_pre_build(&"1.0.0.0".to_string()),
        ("1.0.0.0".to_string(), "".to_string())
    );
    assert_eq!(
        split_pre_build(&"1.0.0.0-alpha".to_string()),
        ("1.0.0.0".to_string(), "-alpha".to_string())
    );
    assert_eq!(
        split_pre_build(&"1.2.0.0-alpha.1".to_string()),
        ("1.2.0.0".to_string(), "-alpha.1".to_string())
    );
    assert_eq!(
        split_pre_build(&"1.0.0.0-alpha.beta".to_string()),
        ("1.0.0.0".to_string(), "-alpha.beta".to_string())
    );
    assert_eq!(
        split_pre_build(&"1.0.0.0-beta".to_string()),
        ("1.0.0.0".to_string(), "-beta".to_string())
    );
    assert_eq!(
        split_pre_build(&"1.0.0.0-beta.2".to_string()),
        ("1.0.0.0".to_string(), "-beta.2".to_string())
    );
    assert_eq!(
        split_pre_build(&"1.0.0.0-beta.11+build114514".to_string()),
        ("1.0.0.0".to_string(), "-beta.11+build114514".to_string())
    );
    assert_eq!(
        split_pre_build(&"1.0.0.0-rc.1".to_string()),
        ("1.0.0.0".to_string(), "-rc.1".to_string())
    );
    assert_eq!(
        split_pre_build(&"1.0.0.0+BLAKE1919810".to_string()),
        ("1.0.0.0".to_string(), "+BLAKE1919810".to_string())
    );
}

/// 拓展的 SemVer 规范，在修订号后面多了一位保留号（reserved）用于标记不同的打包版本
#[derive(Clone, Debug, Eq)]
pub struct ExSemVer {
    pub major: u64,
    pub minor: u64,
    pub patch: u64,
    pub reserved: u64,
    pub pre: Prerelease,
    pub build: BuildMetadata,

    pub semver_instance: semver::Version,
}

impl ExSemVer {
    pub fn _new(
        major: u64,
        minor: u64,
        patch: u64,
        reserved: u64,
        pre: Prerelease,
        build: BuildMetadata,
    ) -> Self {
        ExSemVer {
            major,
            minor,
            patch,
            reserved,
            pre: pre.clone(),
            build: build.clone(),
            semver_instance: semver::Version {
                major,
                minor,
                patch,
                pre,
                build,
            },
        }
    }
    pub fn parse(text: &String) -> Result<Self> {
        // 分割 pre 和 build
        let (clear_text, pre_build) = split_pre_build(text);

        // 使用小数点分割
        let s: Vec<&str> = clear_text.split(".").collect();
        let length = s.len();
        if length != 3 && length != 4 {
            return Err(anyhow!(
                "Error:Can't parse '{text}' as extended semver : expected 3 or 4 fields, got {length} "
            ));
        }

        // 生成标准 semver
        let sem_version = format!("{}.{}.{}{}", s[0], s[1], s[2], pre_build);
        let semver = semver::Version::parse(&sem_version)?;
        let semver_instance = semver.clone();

        // 解析字符串为 u64
        let major = s[0].parse::<u64>()?;
        let minor = s[1].parse::<u64>()?;
        let patch = s[2].parse::<u64>()?;
        let reserved = s.get(3).unwrap_or(&"0").parse::<u64>()?;

        Ok(ExSemVer {
            major,
            minor,
            patch,
            reserved,
            pre: semver.pre,
            build: semver.build,
            semver_instance,
        })
    }
    pub fn set_reserved(&mut self, reserved: u64) {
        self.reserved = reserved;
    }
}

impl From<semver::Version> for ExSemVer {
    fn from(sv: semver::Version) -> Self {
        let semver_instance = sv.clone();
        Self {
            major: sv.major,
            minor: sv.minor,
            patch: sv.patch,
            reserved: 0,
            pre: sv.pre,
            build: sv.build,
            semver_instance,
        }
    }
}

impl FromStr for ExSemVer {
    type Err = Error;
    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        return ExSemVer::parse(&String::from(s));
    }
}

impl PartialEq for ExSemVer {
    fn eq(&self, other: &Self) -> bool {
        let res = self.semver_instance.eq(&other.semver_instance);
        if res {
            self.reserved == other.reserved
        } else {
            res
        }
    }
    fn ne(&self, other: &Self) -> bool {
        !self.eq(other)
    }
}

impl PartialOrd for ExSemVer {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        let res = self.semver_instance.partial_cmp(&other.semver_instance);
        if res.is_none() {
            return None;
        }
        match res {
            Some(Equal) => self.reserved.partial_cmp(&other.reserved),
            _ => res,
        }
    }
    fn ge(&self, other: &Self) -> bool {
        matches!(self.partial_cmp(other), Some(Greater | Equal))
    }
    fn gt(&self, other: &Self) -> bool {
        matches!(self.partial_cmp(other), Some(Greater))
    }
    fn le(&self, other: &Self) -> bool {
        matches!(self.partial_cmp(other), Some(Less | Equal))
    }
    fn lt(&self, other: &Self) -> bool {
        matches!(self.partial_cmp(other), Some(Less))
    }
}

impl Ord for ExSemVer {
    fn cmp(&self, other: &Self) -> Ordering {
        if *self < *other {
            Less
        } else if *self == *other {
            Equal
        } else {
            Greater
        }
    }
    fn max(self, other: Self) -> Self {
        match self.cmp(&other) {
            Ordering::Less | Ordering::Equal => other,
            Ordering::Greater => self,
        }
    }
    fn min(self, other: Self) -> Self {
        match self.cmp(&other) {
            Ordering::Less | Ordering::Equal => self,
            Ordering::Greater => other,
        }
    }
    fn clamp(self, min: Self, max: Self) -> Self {
        assert!(min <= max);
        if self < min {
            min
        } else if self > max {
            max
        } else {
            self
        }
    }
}

impl fmt::Display for ExSemVer {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let pre = if self.pre.is_empty() {
            "".to_string()
        } else {
            String::from("-") + self.pre.as_str()
        };
        let build = if self.build.is_empty() {
            "".to_string()
        } else {
            String::from("+") + self.build.as_str()
        };

        write!(
            f,
            "{}.{}.{}.{}{pre}{build}",
            self.major, self.minor, self.patch, self.reserved
        )
    }
}

#[test]
fn test_ex_semver() {
    // 不带 pre 和 build
    let v1 = ExSemVer::_new(1, 2, 3, 4, Prerelease::EMPTY, BuildMetadata::EMPTY);
    let v2 = ExSemVer::from_str("1.2.3.4").unwrap();
    assert_eq!(v1, v2);
    assert_eq!(v1.to_string(), String::from("1.2.3.4"));
    assert_eq!(format!("{v2}"), "1.2.3.4".to_string());

    assert!(ExSemVer::parse(&"1.12".to_string()).is_err());
    assert!(ExSemVer::parse(&"1.12.3".to_string()).is_ok());
    assert!(ExSemVer::parse(&"1.12.3.9.0".to_string()).is_err());

    let v1 = ExSemVer::parse(&"1.2.3.4".to_string()).unwrap();
    let v2 = ExSemVer::parse(&"1.3.3.1".to_string()).unwrap();
    assert!(v1 < v2);

    let v1 = ExSemVer::parse(&"9.114.2.1".to_string()).unwrap();
    let v2 = ExSemVer::parse(&"10.0.0.0".to_string()).unwrap();
    assert!(v1 < v2);

    let v1 = ExSemVer::parse(&"114.514.1919.810".to_string()).unwrap();
    let v2 = ExSemVer::parse(&"114.514.1919.810".to_string()).unwrap();
    assert!(v1 == v2);

    let v1 = ExSemVer::parse(&"1.2.3.10".to_string()).unwrap();
    let v2 = ExSemVer::parse(&"1.2.3.2".to_string()).unwrap();
    assert!(v1 >= v2);

    let sv = semver::Version::from_str("114.514.19").unwrap();
    let v1 = ExSemVer::from(sv);
    let mut v2 = ExSemVer::from_str("114.514.19.0").unwrap();
    assert_eq!(v1, v2);
    v2.set_reserved(810);
    let v3 = ExSemVer::from_str("114.514.19.810").unwrap();
    assert_eq!(v2, v3);

    // 带 pre 和 build
    let v1 = ExSemVer::parse(&"1.2.3.4-alpha.2.turing".to_string()).unwrap();
    assert_eq!(
        v1,
        ExSemVer::_new(
            1,
            2,
            3,
            4,
            Prerelease::from_str("alpha.2.turing").unwrap(),
            BuildMetadata::EMPTY
        )
    );

    let v2 = ExSemVer::parse(&"1.20.3.4+build114514".to_string()).unwrap();
    assert_eq!(
        v2,
        ExSemVer::_new(
            1,
            20,
            3,
            4,
            Prerelease::EMPTY,
            BuildMetadata::from_str("build114514").unwrap()
        )
    );

    let v3 = ExSemVer::parse(&"1.12.3.4-beta.2.edgeless+blake456".to_string()).unwrap();
    assert_eq!(
        v3,
        ExSemVer::_new(
            1,
            12,
            3,
            4,
            Prerelease::from_str("beta.2.edgeless").unwrap(),
            BuildMetadata::from_str("blake456").unwrap()
        )
    );

    assert!(v1 < v3);
    assert!(v1 < v2);
    assert!(v2 > v3);

    assert_eq!(
        format!("{v3}"),
        "1.12.3.4-beta.2.edgeless+blake456".to_string()
    );

    assert!(ExSemVer::parse(&"1.12.3-alpha".to_string()).is_ok());
    assert!(ExSemVer::parse(&"1.12.3-alpha-beta".to_string()).is_ok());
    assert!(ExSemVer::parse(&"1.12.3+alpha-beta".to_string()).is_ok());

    assert!(ExSemVer::parse(&"1.12-alpha".to_string()).is_err());
    assert!(ExSemVer::parse(&"1.12-alpha-beta".to_string()).is_err());
    assert!(ExSemVer::parse(&"1.12+alpha-beta".to_string()).is_err());

    // 测试 pre 与 build比较关系
    let arr: Vec<ExSemVer> = vec![
        "1.0.0.0-alpha",
        "1.0.0.0-alpha.1",
        "1.0.0.0-alpha.beta",
        "1.0.0.0-beta",
        "1.0.0.0-beta.2",
        "1.0.0.0-beta.11",
        "1.0.0.0-rc.1",
        "1.0.0.0",
    ]
    .into_iter()
    .map(|str| ExSemVer::from_str(str).unwrap())
    .collect();

    for i in 0..arr.len() - 1 {
        assert!(arr[i] < arr[i + 1]);
    }
}
