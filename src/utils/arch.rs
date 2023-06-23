use anyhow::{anyhow,Result, Ok};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug,PartialEq,Eq)]
pub enum SysArch{
    X64,
    X86,
    ARM64,
}

pub fn get_arch()->Result<SysArch>{
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

pub fn parse_arch(text:&String)->Result<SysArch>{
    match text.as_str() {
        "X64"=>Ok(SysArch::X64),
        "X86"=>Ok(SysArch::X86),
        "ARM64"=>Ok(SysArch::ARM64),
        _=>Err(anyhow!("Error:Failed to parse '{text}' as valid system arch"))
    }
}