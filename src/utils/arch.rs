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
        return Err(anyhow!("Error:Failed to get current system arch, that's amazing that you seems to be running a magic Windows OS"));
    }
}

fn parse_arch(text: &String) -> Result<SysArch> {
    match text.as_str() {
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
