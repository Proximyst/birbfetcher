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
