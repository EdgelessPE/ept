use crate::types::Signature;
use anyhow::{anyhow, Result};
use std::path::Path;
use std::{fs::File, io::Read};

pub fn parse_signature(p: String) -> Result<Signature> {
    let signature_path = Path::new(&p);
    if !signature_path.exists() {
        return Err(anyhow!("Error:Fatal:Can't find signature.toml path : {}", p));
    }

    let mut text = String::new();
    File::open(&p)?.read_to_string(&mut text)?;
    let sign_res = toml::from_str(&text);
    if sign_res.is_err() {
        return Err(anyhow!(
            "Error:Can't parse signature.toml at {} : {}",
            p,
            sign_res.err().unwrap()
        ));
    }

    Ok(sign_res.unwrap())
}
