use crate::log;
use crate::types::signature::Signature;
use anyhow::{anyhow, Result};
use std::path::Path;
use std::{fs::File, io::Read};

pub fn parse_signature(p: &str) -> Result<Signature> {
    log!("Debug:Parsing signature from '{p}'");
    let signature_path = Path::new(p);
    if !signature_path.exists() {
        return Err(anyhow!("Error:Fatal:Can't find signature.toml path : {p}"));
    }

    let mut text = String::new();
    File::open(p)?.read_to_string(&mut text)?;
    let sign: Signature = toml::from_str(&text)
        .map_err(|err| anyhow!("Error:Can't parse signature.toml at {p} : {err}"))?;

    log!(
        "Debug:Successfully parsed signature for '{name}'",
        name = sign.package.raw_name_stem
    );
    Ok(sign)
}

pub fn fast_parse_signature(raw: &mut [u8]) -> Result<Signature> {
    let sign: Signature = toml::from_str(std::str::from_utf8(raw)?)
        .map_err(|err| anyhow!("Error:Can't parse signature.toml : {err}"))?;

    log!(
        "Debug:Successfully parsed signature for '{name}'",
        name = sign.package.raw_name_stem
    );
    Ok(sign)
}
