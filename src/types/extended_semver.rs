use anyhow::{anyhow, Error, Result};
use semver::{BuildMetadata, Prerelease};
use std::cmp::Ordering::{Equal, Greater, Less};
use std::fmt;
use std::{cmp::Ordering, str::FromStr};

#[derive(Clone, Debug, Eq)]
pub struct ExSemVer {
    pub major: u64,
    pub minor: u64,
    pub patch: u64,
    pub reserved: u64,
    pub semver_instance: semver::Version,
}

impl ExSemVer {
    pub fn new(major: u64, minor: u64, patch: u64, reserved: u64) -> Self {
        ExSemVer {
            major,
            minor,
            patch,
            reserved,
            semver_instance: semver::Version {
                major,
                minor,
                patch,
                pre: Prerelease::EMPTY,
                build: BuildMetadata::EMPTY,
            },
        }
    }
    pub fn parse(text: String) -> Result<Self> {
        // 使用小数点分割
        let s: Vec<&str> = text.split(".").collect();
        if s.len() != 4 {
            return Err(anyhow!(
                "Error:Can't parse '{}' as extended semver : expected 4 fields, got {} ",
                text,
                s.len()
            ));
        }

        // 生成标准 semver
        let sem_version = format!("{}.{}.{}", s[0], s[1], s[2]);
        let semver_instance = semver::Version::parse(&sem_version)?;

        // 解析字符串为 u64
        let major = s[0].parse::<u64>()?;
        let minor = s[1].parse::<u64>()?;
        let patch = s[2].parse::<u64>()?;
        let reserved = s[3].parse::<u64>()?;

        Ok(ExSemVer {
            major,
            minor,
            patch,
            reserved,
            semver_instance,
        })
    }
    pub fn set_reserved(&mut self, reserved: u64) {
        self.reserved = reserved;
    }
}

impl From<semver::Version> for ExSemVer {
    fn from(sv: semver::Version) -> Self {
        Self {
            major: sv.major,
            minor: sv.minor,
            patch: sv.patch,
            reserved: 0,
            semver_instance: sv,
        }
    }
}

impl FromStr for ExSemVer {
    type Err = Error;
    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        return ExSemVer::parse(String::from(s));
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
        write!(
            f,
            "{}.{}.{}.{}",
            self.major, self.minor, self.patch, self.reserved
        )
    }
}

#[test]
fn test_ex_semver() {
    let v1 = ExSemVer::new(1, 2, 3, 4);
    let v2 = ExSemVer::from_str("1.2.3.4").unwrap();
    assert_eq!(v1, v2);
    assert_eq!(v1.to_string(), String::from("1.2.3.4"));

    let v1 = ExSemVer::parse("1.2.3.4".to_string()).unwrap();
    let v2 = ExSemVer::parse("1.3.3.1".to_string()).unwrap();
    assert!(v1 < v2);

    let v1 = ExSemVer::parse("9.114.2.1".to_string()).unwrap();
    let v2 = ExSemVer::parse("10.0.0.0".to_string()).unwrap();
    assert!(v1 < v2);

    let v1 = ExSemVer::parse("114.514.1919.810".to_string()).unwrap();
    let v2 = ExSemVer::parse("114.514.1919.810".to_string()).unwrap();
    assert!(v1 == v2);

    let v1 = ExSemVer::parse("1.2.3.10".to_string()).unwrap();
    let v2 = ExSemVer::parse("1.2.3.2".to_string()).unwrap();
    assert!(v1 >= v2);

    let sv = semver::Version::from_str("114.514.19").unwrap();
    let v1 = ExSemVer::from(sv);
    let mut v2 = ExSemVer::from_str("114.514.19.0").unwrap();
    assert_eq!(v1, v2);
    v2.set_reserved(810);
    let v3 = ExSemVer::from_str("114.514.19.810").unwrap();
    assert_eq!(v2, v3);
}
