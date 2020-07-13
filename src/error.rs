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
