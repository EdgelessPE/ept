use anyhow::{anyhow, Ok, Result};
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
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

pub fn get_arch() -> Result<SysArch> {
    #[cfg(target_arch = "x86")]
    return Ok(SysArch::X86);
    #[cfg(target_arch = "x86_64")]
    return Ok(SysArch::X64);
    #[cfg(target_arch = "aarch64")]
    return Ok(SysArch::ARM64);

    #[allow(unreachable_code)]
    {
        Err(anyhow!("Error:Failed to get current system arch, that's amazing that you seems to be running a magic Windows OS"))
    }
}

fn parse_arch(text: &String) -> Result<SysArch> {
    match text.to_uppercase().as_str() {
        "X64" => Ok(SysArch::X64),
        "X86" => Ok(SysArch::X86),
        "ARM64" => Ok(SysArch::ARM64),
        _ => Err(anyhow!(
            "Error:Failed to parse '{text}' as valid system arch"
        )),
    }
}

pub fn is_current_arch_match(pkg_arch: &String) -> Result<()> {
    let sys_arch = get_arch()?;
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

    if allowed_arch.contains(&parse_arch(pkg_arch)?) {
        Ok(())
    } else {
        Err(anyhow!(
            "Error:Package arch '{pkg_arch}' doesn't match current os arch '{sys_arch:?}'"
        ))
    }
}

#[test]
fn test_parse_arch() {
    assert_eq!(parse_arch(&"X64".to_string()).unwrap(), SysArch::X64);
    assert_eq!(parse_arch(&"x64".to_string()).unwrap(), SysArch::X64);
    assert_eq!(parse_arch(&"X86".to_string()).unwrap(), SysArch::X86);
    assert_eq!(parse_arch(&"x86".to_string()).unwrap(), SysArch::X86);
    assert_eq!(parse_arch(&"ARM64".to_string()).unwrap(), SysArch::ARM64);
    assert_eq!(parse_arch(&"aRm64".to_string()).unwrap(), SysArch::ARM64);
    assert!(parse_arch(&"RISC".to_string()).is_err());
}

#[test]
fn test_is_current_arch_match() {
    let cur_arch = get_arch().unwrap();
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
