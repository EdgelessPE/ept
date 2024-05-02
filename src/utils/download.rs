use anyhow::Result;
use reqwest::blocking::Client;
use std::fs::File;
use std::io::{Read, Write};

pub fn download(url: &String, at: &String) -> Result<()> {
    let mut response = Client::new().get(url).send()?;

    let total_size = response.content_length().unwrap_or(0);

    let mut position = 0;
    let mut buffer = [0u8; 1024];

    let mut file = File::create(at)?;

    while let Ok(length) = response.read(&mut buffer) {
        if length == 0 {
            break;
        }

        file.write_all(&buffer[..length])?;

        // 更新已处理的数据量
        position += length as u64;

        // 打印下载进度
        println!(
            "Downloaded {} of {}, {:.2}%",
            position,
            total_size,
            if total_size > 0 {
                (100 * position / total_size) as usize
            } else {
                0
            }
        );
    }

    Ok(())
}

#[test]
fn test_download() {
    download(
        &"http:/localhost:3000/api/redirect?path=/nep/Microsoft/VSCode/VSCode_1.85.1.0_Cno.nep"
            .to_string(),
        &"down_test.nep".to_string(),
    )
    .unwrap();
}
