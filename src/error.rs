// birbfetcher - Collect bird images with ease.
// Copyright (C) 2020-2021 Mariell Hoversholm
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

/// An error related to the Reddit API.
#[derive(Debug, Error)]
pub enum RedditError {
    /// The HTTP client could not fetch results correctly.
    #[error("error when fetching result: {0}")]
    Reqwest(#[from] reqwest::Error),

    /// Serde could not deserialise the result.
    #[error("error when deserializing result: {0}")]
    Serde(#[from] serde_json::Error),

    /// The HTTP request returned a bad status code.
    #[error("unsuccessful http request: {0}")]
    Unsuccessful(reqwest::StatusCode),

    #[error("no such post was found")]
    NoPost,
}

#[derive(Debug, Error)]
pub enum CheckingError {
    #[error("error when modifying database: {0}")]
    Sql(#[from] sqlx::Error),

    #[error("unsuccessful reddit api request: {0}")]
    Reddit(#[from] RedditError),
}

/// An error related to processing of images.
#[derive(Debug, Error)]
pub enum ProcessingError {
    /// The image of the post could not be fetched correctly.
    #[error("error when fetching image: {0}")]
    Reqwest(#[from] reqwest::Error),

    /// The HTTP request to the image returned a bad status code.
    #[error("unsuccessful http request: {0}")]
    Unsuccessful(reqwest::StatusCode),

    /// A disallowed content type was returned by the image's host.
    #[error("invalid content type")]
    InvalidContentType,

    /// The post already exists in our database.
    #[error("post is a duplicate")]
    Duplicate,

    /// The post could not be saved.
    #[error("saving the image encountered an error: {0}")]
    SaveError(#[from] std::io::Error),

    /// The post could not be put into our database.
    #[error("sql error: {0}")]
    SqlError(#[from] sqlx::Error),
}

/// An error related to the serving of images and information.
#[derive(Debug, Error)]
pub enum HttpErrorKind {
    /// An error occurred while fetching data from our database.
    #[error("sql error: {0}")]
    SqlError(#[from] sqlx::Error),

    /// An error occurred while reading some file.
    #[error("io error: {0}")]
    IoError(#[from] std::io::Error),

    /// Warp returned an error in something HTTP related.
    #[error("warp http error: {0}")]
    WarpHttpError(#[from] warp::http::Error),
}

/// An error wrapper with a status code for `HttpErrorKind`s.
#[derive(Debug, Error)]
#[error("{source}")]
pub struct HttpError {
    /// The status code to return in the response.
    pub status: StatusCode,

    /// The kind of error that caused this, with any potentially attached state.
    pub source: HttpErrorKind,
}

// Mark the error as a rejection for Warp.
impl warp::reject::Reject for HttpError {}

/// A helper trait for converting from types to `HttpError`s in some way.
pub trait HttpErrorHelper {
    /// The output of this trait implementation.
    type Output;

    /// Convert this type into a `Self::Output`.
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
