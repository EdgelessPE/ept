use crate::types::signature::Signature;
use anyhow::{anyhow, Result};
use std::path::Path;
use std::{fs::File, io::Read};

pub fn parse_signature(p: &String) -> Result<Signature> {
    let signature_path = Path::new(p);
    if !signature_path.exists() {
        return Err(anyhow!(
            "Error:Fatal:Can't find signature.toml path : {}",
            p
        ));
    }

    let mut text = String::new();
    File::open(p)?.read_to_string(&mut text)?;
    let sign = toml::from_str(&text)
        .map_err(|err| anyhow!("Error:Can't parse signature.toml at {} : {}", p, err))?;

    Ok(sign)
}

pub fn fast_parse_signature(raw: &mut Vec<u8>) -> Result<Signature> {
    let sign = toml::from_str(std::str::from_utf8(raw)?)
        .map_err(|err| anyhow!("Error:Can't parse signature.toml : {}", err))?;

    Ok(sign)
}
