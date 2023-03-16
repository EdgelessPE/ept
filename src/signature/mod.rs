mod blake3;
mod ecdsa;

use self::blake3::{compute_hash_blake3, fast_compute_hash_blake3};
use self::ecdsa::{sign_with_ecdsa, verify_with_ecdsa};
use crate::ca::{get_own_pair, query_others_public};
use anyhow::Result;

pub fn sign(target_file: String) -> Result<String> {
    // 获取私钥
    let (_, private) = get_own_pair()?;
    // 计算 blake3 摘要值
    let digest = compute_hash_blake3(target_file)?;
    // 计算签名
    sign_with_ecdsa(private, digest)
}

pub fn verify(target_file: String, package_signer: String, signature: String) -> Result<bool> {
    // 查询公钥
    let public = query_others_public(package_signer)?;
    // 计算 blake3 摘要值
    let digest = compute_hash_blake3(target_file)?;
    // 验证签名
    verify_with_ecdsa(public, digest, signature)
}

pub fn fast_verify(raw: &Vec<u8>, package_signer: String, signature: String) -> Result<bool> {
    // 查询公钥
    let public = query_others_public(package_signer)?;
    // 计算 blake3 摘要值
    let digest = fast_compute_hash_blake3(raw)?;
    // 验证签名
    verify_with_ecdsa(public, digest, signature)
}
