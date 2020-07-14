use crate::prelude::*;
use anyhow::Result;
use std::fs;
use std::path::PathBuf;
use warp::http::Response;
use warp::Rejection;

macro_rules! delegate {
    ($impl:expr => |$err:ident| $errlog:block) => {
        Ok(match $impl.await {
            Ok(resp) => resp,
            Err($err) => {
                $errlog;
                Response::builder()
                    .status(500)
                    .body($err.to_string().into_bytes())
                    .unwrap()
            }
        })
    };
    ($impl:expr => |$err:ident| $errlog:expr) => {
        delegate!($impl => |$err| { $errlog; })
    };
}

macro_rules! cookie {
    ($name:literal = $value:expr) => {
        format!(
            "{name}={value}; SameSite=Strict; HttpOnly",
            name = $name,
            value = $value
        )
    };
}

pub async fn random(db: &MySqlPool, birb_dir: &PathBuf) -> Result<Response<Vec<u8>>, Rejection> {
    delegate! {
        random_impl(db, birb_dir) => |e|
            error!("Error upon calling random HTTP endpoint: {}", e)
    }
}

async fn random_impl(db: &MySqlPool, birb_dir: &PathBuf) -> Result<Response<Vec<u8>>> {
    serve_image(
        birb_dir,

        sqlx::query_as(
            "SELECT id, hash, permalink, content_type FROM birbs WHERE banned = false ORDER BY RAND() LIMIT 1"
        )
        .fetch_one(db)
        .await?
    ).await
}

pub async fn get_by_id(
    db: &MySqlPool,
    birb_dir: &PathBuf,
    id: u32,
) -> Result<Response<Vec<u8>>, Rejection> {
    delegate! {
        get_by_id_impl(db, birb_dir, id) => |e|
            error!(
                "Error upon calling get_by_id HTTP endpoint for ID {}: {}",
                id, e
            )
    }
}

async fn get_by_id_impl(db: &MySqlPool, birb_dir: &PathBuf, id: u32) -> Result<Response<Vec<u8>>> {
    serve_image(
        birb_dir,

        sqlx::query_as(
            "SELECT id, hash, permalink, content_type FROM birbs WHERE banned = false AND id = ? LIMIT 1"
        )
        .bind(id)
        .fetch_one(db)
        .await?
    ).await
}

async fn serve_image(
    birb_dir: &PathBuf,
    (id, hash, permalink, content_type): (u32, Vec<u8>, String, String),
) -> Result<Response<Vec<u8>>> {
    let hex = hex::encode_upper(hash);
    let file = birb_dir.join(&hex);

    Response::builder()
        .header("Content-Type", content_type)
        .header(warp::http::header::SET_COOKIE, cookie!("Id" = id))
        .header(
            warp::http::header::SET_COOKIE,
            cookie!("Permalink" = permalink),
        )
        .header(warp::http::header::SET_COOKIE, cookie!("Hash" = hex))
        .body(fs::read(file)?)
        .map_err(Into::into)
}
