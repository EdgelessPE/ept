use anyhow::Result;
use std::fs::read_to_string;

// 返回：（公钥，私钥）
pub fn get_own_pair() -> Result<(String, String)> {
    let public_file = read_to_string("./keys/public.pem")?;
    let private_file = read_to_string("./keys/private.key")?;
    Ok((public_file, private_file))
}

pub fn query_others_public(email: String) -> Result<String> {
    let public_file = read_to_string("./keys/public.pem")?;
    Ok(public_file)
}
