pub mod blake3;
mod ecdsa;

use self::blake3::compute_hash_blake3;
use self::blake3::fast_compute_hash_blake3;
use self::ecdsa::{sign_with_ecdsa, verify_with_ecdsa};
use crate::ca::{get_own_pair, query_others_public};
use crate::log;
use anyhow::Result;

pub fn sign(target_file: &str) -> Result<String> {
    log!("Debug:Signing file '{target_file}'");
    // 获取私钥
    let (_, private) = get_own_pair()?;
    // 计算 blake3 摘要值
    let digest = compute_hash_blake3(target_file)?;
    // 计算签名
    let signature = sign_with_ecdsa(&private, &digest)?;
    log!("Debug:Successfully signed file '{target_file}'");
    Ok(signature)
}

pub fn verify(target_file: &str, package_signer: &str, signature: &str) -> Result<bool> {
    log!("Debug:Verifying signature for '{target_file}' from signer '{package_signer}'");
    // 查询公钥
    let public = query_others_public(package_signer)?;
    // 计算 blake3 摘要值
    let digest = compute_hash_blake3(target_file)?;
    // 验证签名
    let result = verify_with_ecdsa(&public, &digest, signature)?;
    log!("Debug:Signature verification result for '{target_file}': {result}");
    Ok(result)
}

pub fn fast_verify(raw: &[u8], package_signer: &str, signature: &str) -> Result<bool> {
    log!("Debug:Fast verifying signature from signer '{package_signer}'");
    // 查询公钥
    let public = query_others_public(package_signer)?;
    // 计算 blake3 摘要值
    let digest = fast_compute_hash_blake3(raw)?;
    // 验证签名
    let result = verify_with_ecdsa(&public, &digest, signature)?;
    log!("Debug:Fast signature verification result: {result}");
    Ok(result)
}
