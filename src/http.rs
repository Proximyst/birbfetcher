use crate::prelude::*;
use anyhow::Result;
use serde::Serialize;
use std::fs;
use std::path::PathBuf;
use warp::http::Response;
use warp::{Rejection, Reply};

// {{{ Macros
macro_rules! delegate {
    ($impl:expr => |$err:ident| $errlog:block) => {
        match $impl.await {
            Ok(resp) => Ok(resp),
            Err($err) => {
                $errlog;
                Err(warp::reject::custom(StdErrorReject::from($err)))
            }
        }
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
// }}}

// {{{ GET / - random image
pub async fn random(db: &MySqlPool, birb_dir: &PathBuf) -> Result<impl Reply, Rejection> {
    delegate! {
        random_impl(db, birb_dir) => |e|
            error!("Error upon calling random HTTP endpoint: {}", e)
    }
}

async fn random_impl(db: &MySqlPool, birb_dir: &PathBuf) -> Result<impl Reply> {
    serve_image(
        birb_dir,

        sqlx::query_as(
            "SELECT id, hash, permalink, content_type FROM birbs WHERE banned = false ORDER BY RAND() LIMIT 1"
        )
        .fetch_one(db)
        .await?
    ).await
}
// }}}

// {{{ GET /id/:id - get image by id
pub async fn get_by_id(
    db: &MySqlPool,
    birb_dir: &PathBuf,
    id: u32,
) -> Result<impl Reply, Rejection> {
    delegate! {
        get_by_id_impl(db, birb_dir, id) => |e|
            error!(
                "Error upon calling get_by_id HTTP endpoint for ID {}: {}",
                id, e
            )
    }
}

async fn get_by_id_impl(db: &MySqlPool, birb_dir: &PathBuf, id: u32) -> Result<impl Reply> {
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
// }}}

// {{{ GET /info/id/:id - get image info by id
pub async fn get_info_by_id(
    db: &MySqlPool,
    id: u32,
) -> Result<impl Reply, Rejection> {
    delegate! {
        get_info_by_id_impl(db, id) => |e|
            error!(
                "Error upon calling get_info_by_id HTTP endpoint for ID {}: {}",
                id, e
            )
    }
}

async fn get_info_by_id_impl(db: &MySqlPool, id: u32) -> Result<impl Reply> {
    let (id, hash, permalink, content_type, banned): (u32, Vec<u8>, String, String, bool) =
        sqlx::query_as("SELECT id, hash, permalink, content_type, banned FROM birbs WHERE id = ? LIMIT 1")
            .bind(id)
            .fetch_one(db)
            .await?;

    #[derive(Serialize)]
    struct ImageData {
        id: u32,
        hash: String,
        permalink: String,
        content_type: String,
        banned: bool,
    }

    let data = ImageData {
        id,
        hash: hex::encode_upper(hash),
        permalink,
        content_type,
        banned,
    };

    Ok(warp::reply::json(&data))
}
// }}}

// {{{ serve_image - serve an image from db info
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
// }}}

// {{{ Reject impl
#[derive(Debug)]
struct StdErrorReject(String);

impl warp::reject::Reject for StdErrorReject {}

impl<E: ToString> From<E> for StdErrorReject {
    fn from(e: E) -> Self {
        StdErrorReject(e.to_string())
    }
}
// }}}
