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

use thiserror::Error;
use warp::http::StatusCode;

#[derive(Debug, Error)]
pub enum RedditError {
    #[error("error when fetching result: {0}")]
    Reqwest(#[from] reqwest::Error),

    #[error("error when deserializing result: {0}")]
    Serde(#[from] serde_json::Error),

    #[error("unsuccessful http request: {0}")]
    Unsuccessful(reqwest::StatusCode),
}

#[derive(Debug, Error)]
pub enum ProcessingError {
    #[error("error when fetching image: {0}")]
    Reqwest(#[from] reqwest::Error),

    #[error("unsuccessful http request: {0}")]
    Unsuccessful(reqwest::StatusCode),

    #[error("invalid content type")]
    InvalidContentType,

    #[error("post is a duplicate")]
    Duplicate,

    #[error("saving the image encountered an error: {0}")]
    SaveError(#[from] std::io::Error),

    #[error("sql error: {0}")]
    SqlError(#[from] sqlx::Error),
}

#[derive(Debug, Error)]
pub enum HttpErrorKind {
    #[error("sql error: {0}")]
    SqlError(#[from] sqlx::Error),

    #[error("io error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("warp http error: {0}")]
    WarpHttpError(#[from] warp::http::Error),
}

#[derive(Debug, Error)]
#[error("{source}")]
pub struct HttpError {
    pub status: StatusCode,
    pub source: HttpErrorKind,
}

impl warp::reject::Reject for HttpError {}

pub trait HttpErrorHelper {
    type Output;

    fn status(self, status: StatusCode) -> Self::Output;
}

impl<T> HttpErrorHelper for T
where
    T: Into<HttpErrorKind>,
{
    type Output = HttpError;

    fn status(self, status: StatusCode) -> Self::Output {
        HttpError {
            status,
            source: self.into(),
        }
    }
}

impl<T, E> HttpErrorHelper for Result<T, E>
where
    E: Into<HttpErrorKind>,
{
    type Output = Result<T, HttpError>;

    fn status(self, status: StatusCode) -> Self::Output {
        self.map_err(|e| HttpError {
            status,
            source: e.into(),
        })
    }
}
