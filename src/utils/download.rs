use anyhow::{anyhow, Result};
use indicatif::{ProgressBar, ProgressState, ProgressStyle};
use reqwest::blocking::Client;
use std::cmp::min;
use std::fs::File;
use std::io::{Read, Write};
use std::path::PathBuf;

use crate::p2s;

use super::allocate_path_temp;

pub fn download(url: &String, at: &PathBuf) -> Result<()> {
    let url = url.replace("+", "%2B");
    log!("Info:Start downloading '{url}'");

    // 创建进度条
    let pb = ProgressBar::new(0);
    pb.set_style(ProgressStyle::with_template("{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({eta})")
        .unwrap()
        .with_key("eta", |state: &ProgressState, w: &mut dyn std::fmt::Write| write!(w, "{:.1}s", state.eta().as_secs_f64()).unwrap())
        .progress_chars("#>-"));
    let client = Client::new();

    // 发送 GET 请求
    let mut response = client
        .get(&url)
        .send()
        .map_err(|e| anyhow!("Error:Failed to request url '{url}' : {e}"))?;

    // 尝试获取内容长度
    let content_length = response.content_length().unwrap_or(0);
    pb.set_length(content_length);

    // 创建文件以写入数据
    let mut file = File::create(&at)?;

    let mut buf = vec![0; 1024];
    let mut downloaded = 0;
    while let Ok(n) = response.read(&mut buf) {
        if n == 0 {
            break;
        }

        // 更新进度条
        let new = min(downloaded + n as u64, content_length);
        downloaded = new;
        pb.set_position(new);

        // 写入文件
        file.write_all(&buf[0..n])?;
    }
    // 下载完成，清除进度条
    pb.finish_and_clear();
    log!("Info:Downloaded file stored at '{at}'", at = p2s!(at));

    Ok(())
}

pub fn download_nep(url: &String) -> Result<PathBuf> {
    // 下载文件到临时目录
    let temp_dir = allocate_path_temp(&"download".to_string(), false)?;
    let p = temp_dir.join("downloaded.nep");
    download(url, &p)?;

    Ok(p)
}

pub fn fill_url_template(
    url_template: &String,
    scope: &String,
    software: &String,
    file_name: &String,
) -> Result<String> {
    let mut res = url_template.clone();
    if res.contains("{scope}") {
        res = res.replace("{scope}", scope);
    } else {
        return Err(anyhow!(
            "Error:Invalid url template '{url_template}' : missing field 'scope'"
        ));
    }
    if res.contains("{software}") {
        res = res.replace("{software}", software);
    } else {
        return Err(anyhow!(
            "Error:Invalid url template '{url_template}' : missing field 'software'"
        ));
    }
    if res.contains("{file_name}") {
        res = res.replace("{file_name}", file_name);
    } else {
        return Err(anyhow!(
            "Error:Invalid url template '{url_template}' : missing field 'file_name'"
        ));
    }
    Ok(res)
}

// #[test]
// fn test_download() {
//     download(
//         &"http:/localhost:3000/api/redirect?path=/Socket/Edgeless_Beta_4.1.0.iso".to_string(),
//         &"down_test.iso".to_string(),
//     )
//     .unwrap();
// }
