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
    pub banned_by: Option<String>,
    #[serde(default = "String::new")]
    pub subreddit: String,
    pub score: i64,
    pub hidden: bool,
    pub permalink: String,
    #[serde(default = "String::new")]
    pub url: String,
    #[serde(default = "String::new")]
    pub subreddit_type: String,
    pub quarantine: bool,
}

pub async fn request_posts(subreddit: &str, ty: PostType) -> Result<Vec<RedditPost>, RedditError> {
    trace!("Requesting posts for {} type {}...", subreddit, ty);
    let req = crate::REQWEST_CLIENT
        .get(&format!(
            "{}/r/{}/{}.json?limit=100",
            REDDIT_API, subreddit, ty
        ))
        .send()
        .await?;

    if !req.status().is_success() {
        trace!("Got unsuccessful posts for {} & {}", ty, subreddit);
        return Err(RedditError::Unsuccessful(req.status()));
    }

    #[derive(Deserialize)]
    struct PostContainerData {
        data: RedditPost,
    }
    #[derive(Deserialize)]
    struct PostContainer {
        children: Vec<PostContainerData>,
    }
    #[derive(Deserialize)]
    struct Post {
        data: PostContainer,
    }

    trace!(
        "Deserializing posts for {}/{} into container",
        subreddit,
        ty
    );
    let post: Post = serde_json::from_str(&req.text().await?)?;

    trace!("Posts for {}/{} properly fetched!", subreddit, ty);
    Ok(post.data.children.into_iter().map(|p| p.data).collect())
}

impl RedditPost {
    pub fn is_unsafe(&self) -> bool {
        self.subreddit.is_empty() // Just don't process
            || self.url.is_empty() // Just don't process
            || self.hidden
            || self.quarantine
            || match self.banned_by {
                None => false,
                Some(ref s) if s.is_empty() => false,
                _ => true,
            }
            || self.score < 1
            || self.subreddit_type != "public"
            || !self.is_url_safe()
    }

    #[inline(always)]
    pub fn is_safe(&self) -> bool {
        !self.is_unsafe()
    }

    pub fn is_url_safe(&self) -> bool {
        !self.url.trim().is_empty()
            && self.url.starts_with("https://i.redd.it/")
            && [".jpg", ".jpeg", ".png", ".gif", ".gifv", ".webm"]
                .iter()
                .any(|ext| self.url.ends_with(ext))
    }
}
