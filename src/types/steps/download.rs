use std::path::Path;

use super::TStep;
use crate::{
    executor::values_validator_path,
    p2s,
    signature::blake3::compute_hash_blake3,
    types::{
        interpretable::Interpretable,
        mixed_fs::MixedFS,
        permissions::{Generalizable, Permission, PermissionKey, PermissionLevel},
        workflow::WorkflowContext,
    },
    utils::{
        cache::spawn_cache, download::download, get_path_cache, wild_match::contains_wild_match,
    },
};
use anyhow::{anyhow, Ok, Result};
use regex::Regex;
use serde::{Deserialize, Serialize};

lazy_static! {
    static ref RE: Regex = Regex::new(r"^[0-9a-zA-Z]{64}$").unwrap();
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
    //# `at = "bin/b3sum.exe"`
    //@ 是合法的相对路径
    //@ 在校验工作流时不存在该文件
    pub at: String,
}

impl TStep for StepDownload {
    fn run(self, cx: &mut WorkflowContext) -> Result<i32> {
        //- （仅能在拓展工作流中使用）从 URL 下载文件并使用提供的 BLAKE3 Hash 校验完整性。
        // 下载
        let p = Path::new(&cx.located).join(&self.at).to_path_buf();
        let cache_path = get_path_cache()?;
        let cache_ctx = download(
            &self.url,
            p.clone(),
            Some((cache_path, self.hash_blake3.clone())),
        )
        .map_err(|e| {
            anyhow!(
                "Error(Download):Failed to download from '{}' to '{}': {e}",
                self.url,
                self.at
            )
        })?;
        // 校验
        let got_hash = compute_hash_blake3(&p2s!(p)).map_err(|e| {
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

        // 缓存
        spawn_cache(cache_ctx)
            .map_err(|e| anyhow!("Error(Download)::Failed to cache downloaded file : {e}"))?;

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
    fn verify_step(&self, ctx: &super::VerifyStepCtx) -> Result<()> {
        // 只能在展开工作流中使用
        if !ctx.is_expand_flow {
            return Err(anyhow!(
                "Error(Download):Download step can only be used in expand workflow"
            ));
        }
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

        // 校验时该文件不存在
        if Path::new(&ctx.located).join(&self.at).exists() {
            return Err(anyhow!("Error(Download):File '{}' already exists", self.at));
        }

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

#[test]
fn test_download() {
    // 复制下载测试文件
    if !Path::new("test").exists() {
        std::fs::create_dir("test").unwrap();
    }
    std::fs::copy(
        "examples/Dism++/Dism++/Dism++x64.exe",
        "test/download-test.exe",
    )
    .unwrap();

    // 启动下载测试服务器
    let (addr, mut handler) = crate::utils::test::_run_static_file_server();

    // 执行测试
    let mut cx = WorkflowContext::_demo();
    let step = StepDownload {
        url: format!("{addr}/download-test.exe"),
        hash_blake3: "0218ef74c47f601d555499bcc3b02564d9de34ad1e2ee712af10957e2799f0fd".to_string(),
        at: "test/target-test.exe".to_string(),
    };

    let mut ctx = crate::types::steps::VerifyStepCtx::_demo();
    ctx.is_expand_flow = true;
    step.verify_step(&ctx).unwrap();
    step.run(&mut cx).unwrap();

    // 下载地址错误
    assert!(StepDownload {
        url: format!("{addr}/download-test.apk"),
        hash_blake3: "0218ef74c47f601d555499bcc3b02564d9de34ad1e2ee712af10957e2799f0fd".to_string(),
        at: "test/target-test.apk".to_string(),
    }
    .run(&mut cx)
    .is_err());

    // 哈希校验失败
    assert!(StepDownload {
        url: format!("{addr}/download-test.exe"),
        hash_blake3: "1218ef74c47f601d555499bcc3b02564d9de34ad1e2ee712af10957e2799f0fd".to_string(),
        at: "test/target-test.exe".to_string(),
    }
    .run(&mut cx)
    .is_err());

    handler.kill().unwrap();
}

#[test]
fn test_download_corelation() {
    let mut cx = WorkflowContext::_demo();
    let mut mixed_fs = MixedFS::new("".to_string());
    let addr = "http://localhost:8080";

    // 反向工作流
    StepDownload {
        url: format!("{addr}/download-test.exe"),
        hash_blake3: "0218ef74c47f601d555499bcc3b02564d9de34ad1e2ee712af10957e2799f0fd".to_string(),
        at: "test/target-test.exe".to_string(),
    }
    .reverse_run(&mut cx)
    .unwrap();

    // 装箱单
    assert!(StepDownload {
        url: format!("{addr}/download-test.exe"),
        hash_blake3: "0218ef74c47f601d555499bcc3b02564d9de34ad1e2ee712af10957e2799f0fd".to_string(),
        at: "test/target-test.exe".to_string(),
    }
    .get_manifest(&mut mixed_fs)
    .is_empty());
    assert!(mixed_fs.exists("./test/target-test.exe"));

    // 权限
    assert_eq!(
        *StepDownload {
            url: format!("{addr}/download-test.exe"),
            hash_blake3: "0218ef74c47f601d555499bcc3b02564d9de34ad1e2ee712af10957e2799f0fd"
                .to_string(),
            at: "test/target-test.exe".to_string(),
        }
        .generalize_permissions()
        .unwrap()
        .first()
        .unwrap(),
        Permission {
            key: PermissionKey::download_file,
            level: PermissionLevel::Important,
            targets: vec![format!("{addr}/download-test.exe")],
        }
    );

    // 解释
    assert_eq!(
        StepDownload {
            url: "${Home}".to_string(),
            hash_blake3: "${Home}".to_string(),
            at: "${Home}".to_string(),
        }
        .interpret(|s| s.replace("${Home}", "C:/Users/Nep")),
        StepDownload {
            url: "C:/Users/Nep".to_string(),
            hash_blake3: "${Home}".to_string(),
            at: "C:/Users/Nep".to_string(),
        }
    );

    // 校验正确
    crate::utils::test::_ensure_clear_test_dir();
    let mut ctx: crate::types::steps::VerifyStepCtx = crate::types::steps::VerifyStepCtx::_demo();
    ctx.is_expand_flow = true;
    assert!(StepDownload {
        url: format!("{addr}/download-test.exe"),
        hash_blake3: "0218ef74c47f601d555499bcc3b02564d9de34ad1e2ee712af10957e2799f0fd".to_string(),
        at: "test/target-test.exe".to_string(),
    }
    .verify_step(&ctx)
    .is_ok());
    assert!(StepDownload {
        url: "https://github.com/BLAKE3-team/BLAKE3/releases/download/1.5.4/b3sum_windows_x64_bin.exe".to_string(),
        hash_blake3: "0218ef74c47f601d555499bcc3b02564d9de34ad1e2ee712af10957e2799f0fd".to_string(),
        at: "test/target-test.exe".to_string(),
    }
    .verify_step(&ctx)
    .is_ok());

    // 校验错误
    // url 协议错误
    assert!(StepDownload {
        url: "ftp://localhost/download-test.exe".to_string(),
        hash_blake3: "0218ef74c47f601d555499bcc3b02564d9de34ad1e2ee712af10957e2799f0fd".to_string(),
        at: "test/target-test.exe".to_string(),
    }
    .verify_step(&ctx)
    .is_err());
    // 哈希值长度错误
    assert!(StepDownload {
        url: format!("{addr}/download-test.exe"),
        hash_blake3: "0218ef74c47f601d555499bcc3b02564d9de34ad1e2ee712af10957e2799f0fd1"
            .to_string(),
        at: "test/target-test.exe".to_string(),
    }
    .verify_step(&ctx)
    .is_err());
    // 哈希值长度错误
    assert!(StepDownload {
        url: format!("{addr}/download-test.exe"),
        hash_blake3: "18ef74c47f601d555499bcc3b02564d9de34ad1e2ee712af10957e2799f0fd1".to_string(),
        at: "test/target-test.exe".to_string(),
    }
    .verify_step(&ctx)
    .is_err());
    // 哈希值非法字符
    assert!(StepDownload {
        url: format!("{addr}/download-test.exe"),
        hash_blake3: "0218ef74c47f601d555499bcc3b02564d9de34ad1e2ee712af10957e2799f0f!".to_string(),
        at: "test/target-test.exe".to_string(),
    }
    .verify_step(&ctx)
    .is_err());
    // 存放路径使用内置变量开头
    assert!(StepDownload {
        url: format!("{addr}/download-test.exe"),
        hash_blake3: "0218ef74c47f601d555499bcc3b02564d9de34ad1e2ee712af10957e2799f0fd".to_string(),
        at: "${Home}/test/target-test.exe".to_string(),
    }
    .verify_step(&ctx)
    .is_err());
    // 正确：存放路径使用内置变量但是不在开头
    assert!(StepDownload {
        url: format!("{addr}/download-test.exe"),
        hash_blake3: "0218ef74c47f601d555499bcc3b02564d9de34ad1e2ee712af10957e2799f0fd".to_string(),
        at: "test/${ExitCode}/target-test.exe".to_string(),
    }
    .verify_step(&ctx)
    .is_ok());
    // 存放路径使用绝对路径
    assert!(StepDownload {
        url: format!("{addr}/download-test.exe"),
        hash_blake3: "0218ef74c47f601d555499bcc3b02564d9de34ad1e2ee712af10957e2799f0fd".to_string(),
        at: "C:/test/target-test.exe".to_string(),
    }
    .verify_step(&ctx)
    .is_err());
    // 存放路径包含 ..
    assert!(StepDownload {
        url: format!("{addr}/download-test.exe"),
        hash_blake3: "0218ef74c47f601d555499bcc3b02564d9de34ad1e2ee712af10957e2799f0fd".to_string(),
        at: "../test/target-test.exe".to_string(),
    }
    .verify_step(&ctx)
    .is_err());
    // 存放路径包含通配符
    assert!(StepDownload {
        url: format!("{addr}/download-test.exe"),
        hash_blake3: "0218ef74c47f601d555499bcc3b02564d9de34ad1e2ee712af10957e2799f0fd".to_string(),
        at: "test/target-*.exe".to_string(),
    }
    .verify_step(&ctx)
    .is_err());
    // 在非展开工作流中使用
    ctx.is_expand_flow = false;
    assert!(StepDownload {
        url: format!("{addr}/download-test.exe"),
        hash_blake3: "0218ef74c47f601d555499bcc3b02564d9de34ad1e2ee712af10957e2799f0fd".to_string(),
        at: "test/target-test.exe".to_string(),
    }
    .verify_step(&ctx)
    .is_err());
    // 文件已存在
    std::fs::write("test/target-test.exe", "114").unwrap();
    assert!(StepDownload {
        url: format!("{addr}/download-test.exe"),
        hash_blake3: "0218ef74c47f601d555499bcc3b02564d9de34ad1e2ee712af10957e2799f0fd".to_string(),
        at: "test/target-test.exe".to_string(),
    }
    .verify_step(&ctx)
    .is_err());
}
