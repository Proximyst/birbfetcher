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
