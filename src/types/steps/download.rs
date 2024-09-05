use std::path::Path;

use super::TStep;
use crate::{
    executor::values_validator_path,
    signature::compute_hash_blake3,
    types::{
        interpretable::Interpretable,
        mixed_fs::MixedFS,
        permissions::{Generalizable, Permission, PermissionKey, PermissionLevel},
        verifiable::Verifiable,
        workflow::WorkflowContext,
    },
    utils::{download::download, wild_match::contains_wild_match},
};
use anyhow::{anyhow, Ok, Result};
use regex::Regex;
use serde::{Deserialize, Serialize};

lazy_static! {
    static ref RE: Regex = Regex::new(r"[0-9a-zA-Z]{64}").unwrap();
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct StepDownload {
    /// 下载地址。
    //# `url = "https://github.com/BLAKE3-team/BLAKE3/releases/download/1.5.4/b3sum_windows_x64_bin.exe"`
    //@ 是合法的 HTTP 或 HTTPS 地址
    pub url: String,
    /// 使用 BLAKE3 计算得到的哈希值。
    //# `hash_blake3 = "2646a295ef814f070b188278e11ee321d9c6fa18fb9e5bf4b11fcf05b6ef4ff6"`
    //@ 长度为 64 位，且由数字、小写字母、大写字母构成
    pub hash_blake3: String,
    /// 保存的相对位置，起始目录为包内容目录（和包名相同的目录）
    //# `at = "bin/b3sum_windows_x64_bin.exe"`
    //@ 是合法的相对路径
    pub at: String,
}

impl TStep for StepDownload {
    fn run(self, cx: &mut WorkflowContext) -> Result<i32> {
        // 下载
        let p = Path::new(&cx.located).join(&self.at).to_path_buf();
        download(&self.url, &p).map_err(|e| {
            anyhow!(
                "Error(Download):Failed to download from '{}' to '{}': {e}",
                self.url,
                self.at
            )
        })?;
        // 校验
        let got_hash = compute_hash_blake3(&self.at).map_err(|e| {
            anyhow!(
                "Error(Download):Failed to compute hash for file '{}': {e}",
                self.at
            )
        })?;
        if got_hash != self.hash_blake3 {
            return Err(anyhow!(
                "Error(Download):Hash mismatch for file '{}' : expected '{}', got '{got_hash}'",
                self.at,
                self.hash_blake3
            ));
        }

        Ok(0)
    }
    fn reverse_run(self, _cx: &mut WorkflowContext) -> Result<()> {
        Ok(())
    }
    fn get_manifest(&self, fs: &mut MixedFS) -> Vec<String> {
        // 添加下载后的文件
        fs.add(&self.at, "");
        Vec::new()
    }
}

impl Verifiable for StepDownload {
    fn verify_self(&self, _: &String) -> Result<()> {
        // 校验下载地址
        if !self.url.starts_with("http://") && !self.url.starts_with("https://") {
            return Err(anyhow!(
                "Error(Download):Invalid 'url' field '{}' : 'http' or 'https' protocol expected",
                self.url
            ));
        }

        // 哈希值是合法的 BLAKE3 哈希值
        if !RE.is_match(&self.hash_blake3) {
            return Err(anyhow!(
                "Error(Download):Invalid 'hash_blake3' field '{}'",
                self.hash_blake3
            ));
        }

        // 存放位置为合法的相对路径，不应该包含通配符
        if Path::new(&self.at).is_absolute() || self.at.starts_with("${") {
            return Err(anyhow!(
                "Error(Download):Invalid 'at' field '{}' : relative path expected",
                self.at
            ));
        }
        if contains_wild_match(&self.at) {
            return Err(anyhow!(
                "Error(Download):Invalid 'at' field '{}' : wild match not allowed",
                self.at
            ));
        }
        values_validator_path(&self.at).map_err(|e| {
            anyhow!("Error(Download):Failed to validate field 'at' as valid path : {e}")
        })?;

        Ok(())
    }
}

impl Generalizable for StepDownload {
    fn generalize_permissions(&self) -> Result<Vec<Permission>> {
        Ok(vec![Permission {
            key: PermissionKey::download_file,
            level: PermissionLevel::Important,
            targets: vec![self.url.to_owned()],
        }])
    }
}

impl Interpretable for StepDownload {
    fn interpret<F>(self, interpreter: F) -> Self
    where
        F: Fn(String) -> String,
    {
        Self {
            url: interpreter(self.url),
            hash_blake3: self.hash_blake3,
            at: interpreter(self.at),
        }
    }
}
