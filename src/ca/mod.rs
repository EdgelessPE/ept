use anyhow::{anyhow, Result};
use std::{fs::read_to_string, path::Path};

// 返回：（公钥，私钥）
pub fn get_own_pair() -> Result<(String, String)> {
    check()?;
    let public_file = read_to_string("./keys/public.pem")?;
    let private_file = read_to_string("./keys/private.key")?;
    Ok((public_file, private_file))
}

pub fn query_others_public(_email: &String) -> Result<String> {
    check()?;
    let public_file = read_to_string("./keys/public.pem")?;
    Ok(public_file)
}

fn check() -> Result<()> {
    let manifest = vec!["./keys/public.pem", "./keys/private.key"];
    for file_name in manifest {
        let p = Path::new(&file_name);
        if !p.exists() {
            return Err(anyhow!(
                "Error:Missing '{}', please place 'keys' folder in current dir",
                &file_name,
            ));
        }
    }

    Ok(())
}
