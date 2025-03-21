use imghash::{ImageHasher, perceptual::PerceptualHasher};
use reqwest::Response;
use teloxide::types::File;

pub async fn download_file(file: &File, token: &str) -> reqwest::Result<Response> {
    reqwest::get(format!(
        "https://api.telegram.org/file/bot{}/{}",
        token, file.path
    ))
    .await
}

pub fn image_hash(image_bytes: &[u8]) -> anyhow::Result<String> {
    let img = image::load_from_memory(image_bytes)?;
    let hasher = PerceptualHasher::default();
    Ok(hasher.hash_from_img(&img).encode())
}
