use anyhow::{anyhow,Result};
use openssl::pkey::PKey;
use openssl::sign::{Signer, Verifier};
use openssl::hash::MessageDigest;
use base64::{Engine as _, engine::{self, general_purpose}, alphabet};

pub fn sign_with_rsa(private_key:String,digest:String)->Result<String>{
    let private=PKey::private_key_from_pem(private_key.as_bytes())?;
    let mut signer=Signer::new(MessageDigest::sha256(),&private)?;
    signer.update(digest.as_bytes())?;
    let signature=signer.sign_to_vec()?;
    let signature_base64 = general_purpose::STANDARD.encode(&signature);

    Ok(signature_base64)
}

#[test]
fn test_sign_with_rsa(){
    let private_key=
"-----BEGIN RSA PRIVATE KEY-----

-----END RSA PRIVATE KEY-----
";
    let res=sign_with_rsa(private_key.to_string(), "digest".to_string());
    println!("{:?}",res);
}