use crate::prelude::*;
use serde::Deserialize;
use strum_macros::Display;

const REDDIT_API: &'static str = "https://reddit.com";

#[derive(Copy, Clone, Eq, PartialEq, Debug, Display)]
pub enum PostType {
    #[strum(serialize = "new")]
    New,

    #[strum(serialize = "hot")]
    Hot,
}

#[derive(Debug, PartialEq, Eq, Deserialize)]
pub struct RedditPost {
    pub banned_by: String,
    pub subreddit: String,
    pub likes: u32,
    pub view_count: u64,
    pub title: String,
    pub score: i64,
    pub hidden: bool,
    pub permalink: String,
    pub url: String,
    pub subreddit_type: String,
    pub hide_score: bool,
    pub quarantine: bool,
}

pub async fn request_posts(subreddit: &str, ty: PostType) -> Result<Vec<RedditPost>, RedditError> {
    let req = crate::REQWEST_CLIENT
        .get(&format!(
            "{}/r/{}/{}.json?limit=100",
            REDDIT_API, subreddit, ty
        ))
        .send()
        .await?;

    if !req.status().is_success() {
        return Err(RedditError::Unsuccessful(req.status()));
    }

    #[derive(Deserialize)]
    struct PostContainer {
        children: Vec<RedditPost>,
    }
    #[derive(Deserialize)]
    struct Post {
        data: PostContainer,
    }

    let post: Post = serde_json::from_str(&req.text().await?)?;

    Ok(post.data.children)
}
