use crate::prelude::*;
use anyhow::Result;
use std::fs;
use std::path::PathBuf;
use warp::http::Response;
use warp::Rejection;

pub async fn random(db: &MySqlPool, birb_dir: &PathBuf) -> Result<Response<Vec<u8>>, Rejection> {
    Ok(match random_impl(db, birb_dir).await {
        Ok(resp) => resp,
        Err(e) => {
            error!("Error upon calling random HTTP endpoint: {}", e);
            Response::builder()
                .status(500)
                .body(e.to_string().into_bytes())
                .unwrap()
        }
    })
}

async fn random_impl(db: &MySqlPool, birb_dir: &PathBuf) -> Result<Response<Vec<u8>>> {
    let data: (u32, Vec<u8>, String, String,) = sqlx::query_as(
        "SELECT id, hash, permalink, content_type FROM birbs WHERE banned = false ORDER BY RAND() LIMIT 1"
        )
        .fetch_one(db)
        .await?;
    let (id, hash, permalink, content_type) = data;

    let hex = hex::encode_upper(hash);
    let file = birb_dir.join(&hex);

    Response::builder()
        .header("Content-Type", content_type)
        .header(
            warp::http::header::SET_COOKIE,
            format!("Id={}; SameSite=Strict; HttpOpnly", id),
        )
        .header(
            warp::http::header::SET_COOKIE,
            format!("Permalink={}; SameSite=Strict; HttpOpnly", permalink),
        )
        .header(
            warp::http::header::SET_COOKIE,
            format!("Hash={}; SameSite=Strict; HttpOpnly", hex),
        )
        .body(fs::read(file)?)
        .map_err(Into::into)
}

pub async fn get_by_id(db: &MySqlPool, birb_dir: &PathBuf, id: u32) -> Result<Response<Vec<u8>>, Rejection> {
    Ok(match get_by_id_impl(db, birb_dir, id).await {
        Ok(resp) => resp,
        Err(e) => {
            error!("Error upon calling get_by_id HTTP endpoint for ID {}: {}", id, e);
            Response::builder()
                .status(500)
                .body(e.to_string().into_bytes())
                .unwrap()
        }
    })
}

async fn get_by_id_impl(db: &MySqlPool, birb_dir: &PathBuf, id: u32) -> Result<Response<Vec<u8>>> {
    let data: (u32, Vec<u8>, String, String,) = sqlx::query_as(
        "SELECT id, hash, permalink, content_type FROM birbs WHERE banned = false AND id = ? ORDER BY RAND() LIMIT 1"
        )
        .bind(id)
        .fetch_one(db)
        .await?;
    let (_, hash, permalink, content_type) = data;

    let hex = hex::encode_upper(hash);
    let file = birb_dir.join(&hex);

    Response::builder()
        .header("Content-Type", content_type)
        .header(
            warp::http::header::SET_COOKIE,
            format!("Id={}; SameSite=Strict; HttpOpnly", id),
        )
        .header(
            warp::http::header::SET_COOKIE,
            format!("Permalink={}; SameSite=Strict; HttpOpnly", permalink),
        )
        .header(
            warp::http::header::SET_COOKIE,
            format!("Hash={}; SameSite=Strict; HttpOpnly", hex),
        )
        .body(fs::read(file)?)
        .map_err(Into::into)
}

