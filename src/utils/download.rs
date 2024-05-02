use anyhow::Result;
use indicatif::{ProgressBar, ProgressState, ProgressStyle};
use reqwest::blocking::Client;
use std::fs::File;
use std::io::{Read, Write};

pub fn download(url: &String, at: &String) -> Result<()> {
    log!("Info:Start downloading '{url}'");
    // 创建进度条
    let pb = ProgressBar::new(0);
    pb.set_style(ProgressStyle::with_template("{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({bytes_per_sec}, {eta})")
        .unwrap()
        .with_key("eta", |state: &ProgressState, w: &mut dyn std::fmt::Write| write!(w, "{:.1}s", state.eta().as_secs_f64()).unwrap())
        .progress_chars("#>-"));
    let client = Client::new();

    // 发送 GET 请求
    let mut response = client.get(url).send()?;

    // 尝试获取内容长度
    let content_length = response.content_length().unwrap_or(0);
    pb.set_length(content_length);

    // 创建文件以写入数据
    let mut file = File::create(at)?;

    let mut buf = vec![0; 1024];
    while let Ok(n) = response.read(&mut buf) {
        if n == 0 {
            break;
        }

        // 更新进度条
        pb.set_position(pb.position() + n as u64);

        // 写入文件
        file.write_all(&buf[0..n])?;
    }
    // 下载完成，清除进度条
    pb.finish_and_clear();
    log!("Info:Downloaded file stored at '{at}'");

    Ok(())
}

// #[test]
// fn test_download() {
//     download(
//         &"http:/localhost:3000/api/redirect?path=/nep/Microsoft/VSCode/VSCode_1.85.1.0_Cno.nep"
//             .to_string(),
//         &"down_test.nep".to_string(),
//     )
//     .unwrap();
// }
