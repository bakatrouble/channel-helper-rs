use std::io::Cursor;
use std::sync::Arc;
use axum::{Json, Router};
use axum::extract::State;
use axum::routing::post;
use base64::Engine;
use base64::prelude::BASE64_STANDARD;
use image::ImageFormat;
use image::imageops::FilterType;
use reqwest::StatusCode;
use tokio::net::TcpListener;
use serde::{Deserialize, Serialize};
use crate::config::Config;
use crate::database::{Database, MediaType, UploadTask};
use crate::utils::{image_hash};

pub async fn run_server(cfg: Config, db: Arc<Database>) -> anyhow::Result<()> {
    let app = Router::new()
        .route("/post_photo", post(post_photo))
        .with_state(db);

    let listener = TcpListener::bind(format!("0.0.0.0:{}", cfg.api_port.unwrap()))
        .await?;
    axum::serve(listener, app).await?;

    Ok(())
}

#[derive(Debug, Deserialize)]
struct PostPhotoBase64 {
    base64: String,
}

#[derive(Debug, Deserialize)]
struct PostPhotoUrl {
    url: String,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum PostPhoto {
    Base64(PostPhotoBase64),
    Url(PostPhotoUrl),
}

#[derive(Debug, Serialize)]
struct PostPhotoResponseSuccess {
    success: bool,
}

#[derive(Debug, Serialize)]
struct PostPhotoResponseError {
    success: bool,
    duplicate: bool,
    reason: String,
}

#[derive(Debug, Serialize)]
#[serde(untagged)]
enum PostPhotoResponse {
    Ok(PostPhotoResponseSuccess),
    Err(PostPhotoResponseError),
}

async fn post_photo(
    State(db): State<Arc<Database>>,
    Json(payload): Json<PostPhoto>,
) -> (StatusCode, Json<PostPhotoResponse>) {
    macro_rules! post_photo_error {
        ($f:expr, $dup:expr) => {
            {
                log::error!($f);
                (
                    StatusCode::BAD_REQUEST,
                    Json(PostPhotoResponse::Err(PostPhotoResponseError {
                        success: false,
                        duplicate: $dup,
                        reason: format!($f),
                    }))
                )
            }
        }
    }

    let mut photo = match payload {
        PostPhoto::Base64(body) => {
            match BASE64_STANDARD.decode(body.base64) {
                Ok(v) => v,
                Err(e) => {
                    return post_photo_error!("Base64 decode error: {e:?}", false);
                }
            }
        },
        PostPhoto::Url(body) => {
            let bytes = match reqwest::get(body.url).await {
                Ok(v) => v.bytes(),
                Err(e) => {
                    return post_photo_error!("Request error: {e:?}", false);
                }
            };
            match bytes.await {
                Ok(v) => v.to_vec(),
                Err(e) => {
                    return post_photo_error!("Image download error: {e:?}", false);
                }
            }
        },
    };
    let hash = match image_hash(photo.as_slice()) {
        Ok(v) => v,
        Err(e) => {
            return post_photo_error!("Image hashing error: {e:?}", false);
        }
    };

    match db.post_with_hash_exists(hash.clone()).await {
        Ok(false) => {},
        Ok(true) => {
            return post_photo_error!("Hash {hash} already exists", true);
        },
        Err(e) => {
            return post_photo_error!("Database error: {e:?}", false);
        },
    }

    image::load_from_memory(photo.as_slice()).unwrap()
        .resize(2000, 2000, FilterType::Lanczos3)
        .write_to(&mut Cursor::new(photo.as_mut_slice()), ImageFormat::Jpeg).unwrap();

    let upload_task = UploadTask {
        id: None,
        media_type: MediaType::Photo,
        data: photo,
        processed: false,
        image_hash: Some(hash),
    };

    match db.create_upload_task(upload_task).await {
        Ok(_) => {
            (
                StatusCode::OK,
                Json(PostPhotoResponse::Ok(PostPhotoResponseSuccess {
                    success: true,
                }))
            )
        },
        Err(e) => {
            post_photo_error!("Database error: {e:?}", false)
        },
    }
}
