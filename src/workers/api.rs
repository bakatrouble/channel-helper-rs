use crate::{
    config::Config,
    database::{Database, MediaType},
    utils::image_hash,
};
use axum::{Json, Router, extract::State, http::Method, routing::post};
use base64::{Engine, prelude::BASE64_STANDARD};
use image::{ImageFormat, imageops::FilterType};
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use std::{io::Cursor, sync::Arc};
use tokio::net::TcpListener;
use tower::ServiceBuilder;
use tower_http::cors::{Any, CorsLayer};

pub async fn run_server(cfg: Config, db: Arc<Database>) -> anyhow::Result<()> {
    let cors = CorsLayer::new()
        .allow_methods([Method::POST])
        .allow_origin(Any);

    let app = Router::new()
        .route("/post_photo", post(post_photo))
        .layer(ServiceBuilder::new().layer(cors))
        .with_state(db);

    let listener = TcpListener::bind(format!("0.0.0.0:{}", cfg.api_port.unwrap())).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

#[derive(Debug, Deserialize)]
struct PostMediaBase64 {
    base64: String,
}

#[derive(Debug, Deserialize)]
struct PostMediaUrl {
    url: String,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum PostMediaRequest {
    Base64(PostMediaBase64),
    Url(PostMediaUrl),
}

#[derive(Debug, Serialize)]
struct PostMediaResponseSuccess {
    success: bool,
}

#[derive(Debug, Serialize)]
struct PostMediaResponseError {
    success: bool,
    duplicate: bool,
    reason: String,
}

#[derive(Debug, Serialize)]
#[serde(untagged)]
enum PostMediaResponse {
    Ok(PostMediaResponseSuccess),
    Err(PostMediaResponseError),
}

macro_rules! post_media_error {
    ($f:expr, $dup:expr) => {{
        log::error!($f);
        (
            StatusCode::BAD_REQUEST,
            Json(PostMediaResponse::Err(PostMediaResponseError {
                success: false,
                duplicate: $dup,
                reason: format!($f),
            })),
        )
    }};
}

async fn post_photo(
    State(db): State<Arc<Database>>,
    Json(payload): Json<PostMediaRequest>,
) -> (StatusCode, Json<PostMediaResponse>) {
    let mut photo = match payload {
        PostMediaRequest::Base64(body) => match BASE64_STANDARD.decode(body.base64) {
            Ok(v) => v,
            Err(e) => {
                return post_media_error!("Base64 decode error: {e:?}", false);
            }
        },
        PostMediaRequest::Url(body) => {
            let bytes = match reqwest::get(body.url).await {
                Ok(v) => v.bytes(),
                Err(e) => {
                    return post_media_error!("Request error: {e:?}", false);
                }
            };
            match bytes.await {
                Ok(v) => v.to_vec(),
                Err(e) => {
                    return post_media_error!("Image download error: {e:?}", false);
                }
            }
        }
    };
    let hash = match image_hash(photo.as_slice()) {
        Ok(v) => v,
        Err(e) => {
            return post_media_error!("Image hashing error: {e:?}", false);
        }
    };

    match db.post_with_hash_exists(hash.clone()).await {
        Ok(false) => {}
        Ok(true) => {
            return post_media_error!("Hash {hash} already exists", true);
        }
        Err(e) => {
            return post_media_error!("Database error: {e:?}", false);
        }
    }

    image::load_from_memory(photo.as_slice())
        .unwrap()
        .resize(2000, 2000, FilterType::Lanczos3)
        .write_to(&mut Cursor::new(photo.as_mut_slice()), ImageFormat::Jpeg)
        .unwrap();

    match db
        .create_upload_task(MediaType::Photo, photo, Some(hash))
        .await
    {
        Ok(_) => (
            StatusCode::OK,
            Json(PostMediaResponse::Ok(PostMediaResponseSuccess {
                success: true,
            })),
        ),
        Err(e) => {
            post_media_error!("Database error: {e:?}", false)
        }
    }
}
