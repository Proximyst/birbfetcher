// birbfetcher - Collect bird images with ease.
// Copyright (C) 2020 Mariell Hoversholm
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

use crate::prelude::*;
use anyhow::Result;
use serde::Serialize;
use std::convert::Infallible;
use std::fmt::Debug;
use std::fs;
use std::path::PathBuf;
use warp::http::{Response, StatusCode};
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
pub async fn get_info_by_id(db: &MySqlPool, id: u32) -> Result<impl Reply, Rejection> {
    delegate! {
        get_info_by_id_impl(db, id) => |e|
            error!(
                "Error upon calling get_info_by_id HTTP endpoint for ID {}: {}",
                id, e
            )
    }
}

async fn get_info_by_id_impl(db: &MySqlPool, id: u32) -> Result<impl Reply> {
    let (id, hash, permalink, content_type, banned, verified): (u32, Vec<u8>, String, String, bool, bool) =
        sqlx::query_as(
            "SELECT id, hash, permalink, content_type, banned, verified FROM birbs WHERE id = ? LIMIT 1",
        )
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
        verified: bool,
    }

    let data = ImageData {
        id,
        hash: hex::encode_upper(hash),
        permalink,
        content_type,
        banned,
        verified,
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

// {{{ Handle rejections
pub async fn handle_rejection(rej: Rejection) -> Result<impl Reply, Infallible> {
    #[derive(Serialize)]
    struct Error<'a> {
        code: u16,
        code_name: Option<&'a str>,
        message: String,
    }

    let code;
    let message;

    if rej.is_not_found() {
        code = StatusCode::NOT_FOUND;
        message = "NOT_FOUND".into();
    } else if let Some(err) = rej.find::<StdErrorReject>() {
        code = StatusCode::BAD_REQUEST;
        message = err.0.to_string();
    } else {
        code = StatusCode::INTERNAL_SERVER_ERROR;
        message = format!("UNHANDLED_REJECTION: {:?}", rej);
    }

    Ok(warp::reply::with_status(
        warp::reply::json(&Error {
            code: code.as_u16(),
            code_name: code.canonical_reason(),
            message,
        }),
        code,
    ))
}
// }}}
