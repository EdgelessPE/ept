use anyhow::{anyhow, Ok, Result};
use serde::{Deserialize, Serialize};
use std::fmt;
use strum_macros::{EnumString, IntoStaticStr};

#[derive(
    Serialize, Deserialize, Clone, Debug, PartialEq, PartialOrd, Eq, Hash, EnumString, IntoStaticStr,
)]
pub enum SysArch {
    X64,
    X86,
    ARM64,
}
impl fmt::Display for SysArch {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl SysArch {
    pub fn get_current_arch() -> Result<Self> {
        #[cfg(target_arch = "x86")]
        return Ok(Self::X86);
        #[cfg(target_arch = "x86_64")]
        return Ok(Self::X64);
        #[cfg(target_arch = "aarch64")]
        return Ok(Self::ARM64);

        #[allow(unreachable_code)]
        {
            Err(anyhow!("Error:Failed to get current system arch, that's amazing that you seems to be running a magic Windows OS"))
        }
    }

    fn parse(text: &str) -> Result<Self> {
        match text.to_uppercase().as_str() {
            "X64" => Ok(Self::X64),
            "X86" => Ok(Self::X86),
            "ARM64" => Ok(Self::ARM64),
            _ => Err(anyhow!(
                "Error:Failed to parse '{text}' as valid system arch"
            )),
        }
    }
}

pub fn is_current_arch_match(pkg_arch: &String) -> Result<()> {
    let sys_arch = SysArch::get_current_arch()?;
    let allowed_arch = match sys_arch {
        SysArch::X64 => {
            vec![SysArch::X64, SysArch::X86]
        }
        SysArch::X86 => {
            vec![SysArch::X86]
        }
        SysArch::ARM64 => {
            vec![SysArch::X64, SysArch::X86, SysArch::ARM64]
        }
    };

    if allowed_arch.contains(&SysArch::parse(pkg_arch)?) {
        Ok(())
    } else {
        Err(anyhow!(
            "Error:Package arch '{pkg_arch}' doesn't match current os arch '{sys_arch}'"
        ))
    }
}

#[test]
fn test_parse_arch() {
    assert_eq!(SysArch::parse("X64").unwrap(), SysArch::X64);
    assert_eq!(SysArch::parse("X86").unwrap(), SysArch::X86);
    assert_eq!(SysArch::parse("x86").unwrap(), SysArch::X86);
    assert_eq!(SysArch::parse("ARM64").unwrap(), SysArch::ARM64);
    assert_eq!(SysArch::parse("x64").unwrap(), SysArch::X64);
    assert_eq!(SysArch::parse("aRm64").unwrap(), SysArch::ARM64);
    assert!(SysArch::parse("RISC").is_err());
}

#[test]
fn test_is_current_arch_match() {
    let cur_arch = SysArch::get_current_arch().unwrap();
    let cur_arch_str = cur_arch.to_string();
    assert!(is_current_arch_match(&cur_arch_str).is_ok());

    #[cfg(target_arch = "x86")]
    {
        assert!(is_current_arch_match(&"X64".to_string()).is_err());
        assert!(is_current_arch_match(&"x86".to_string()).is_ok());
        assert!(is_current_arch_match(&"ARM64".to_string()).is_err());
    }

    #[cfg(target_arch = "x86_64")]
    {
        assert!(is_current_arch_match(&"x64".to_string()).is_ok());
        assert!(is_current_arch_match(&"X86".to_string()).is_ok());
        assert!(is_current_arch_match(&"ARM64".to_string()).is_err());
    }

    #[cfg(target_arch = "aarch64")]
    {
        assert!(is_current_arch_match(&"x64".to_string()).is_ok());
        assert!(is_current_arch_match(&"X86".to_string()).is_ok());
        assert!(is_current_arch_match(&"arm64".to_string()).is_ok());
    }
}
