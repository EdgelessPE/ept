mod blake3;
mod rsa;

use anyhow::{Result};
use self::blake3::compute_hash_blake3;
use self::rsa::{sign_with_rsa,verify_with_rsa};
use crate::ca::{get_own_pair,query_others_public};

pub fn sign(target_file:String)->Result<String>{
    // 获取私钥
    let (_,private)=get_own_pair()?;
    // 计算 blake3 摘要值
    let digest=compute_hash_blake3(target_file)?;
    // 计算签名
    sign_with_rsa(private, digest)
}

pub fn verify(target_file:String,packager:String,signature:String)->Result<bool>{
    // 查询公钥
    let public=query_others_public(packager)?;
    // 计算 blake3 摘要值
    let digest=compute_hash_blake3(target_file)?;
    // 验证签名
    verify_with_rsa(public, digest, signature)
}