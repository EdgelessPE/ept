use anyhow::Result;
use reqwest::blocking::{ClientBuilder, Response};

pub fn get(url: &str) -> Result<Response> {
    let client_builder = ClientBuilder::new();
    let client = client_builder.zstd(true).build()?;
    let response = client.get(url).send()?;
    Ok(response)
}
