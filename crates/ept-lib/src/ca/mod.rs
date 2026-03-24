use anyhow::Result;

// 返回：（公钥，私钥）
pub fn get_own_pair() -> Result<(String, String)> {
    let public_file = include_str!("../../keys/public.pem").to_string();
    let private_file = include_str!("../../keys/private.key").to_string();
    Ok((public_file, private_file))
}

pub fn query_others_public(_email: &str) -> Result<String> {
    let public_file = include_str!("../../keys/public.pem").to_string();
    Ok(public_file)
}
