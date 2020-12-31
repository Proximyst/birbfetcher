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

// TODO(Proximyst): Ugly file, needa redo this.

use crate::prelude::*;
use serde::Deserialize;
use serde_json::Value as JsonValue;
use strum_macros::Display;

/// The base URL of the Reddit API.
const REDDIT_API: &'static str = "https://reddit.com";

/// The types of posts we can fetch.
#[derive(Copy, Clone, Eq, PartialEq, Debug, Display)]
pub enum PostType {
    #[strum(serialize = "new")]
    New,

    #[strum(serialize = "hot")]
    Hot,
}

/// A data structure of Reddit posts.
#[derive(Debug, Deserialize)]
pub struct RedditPost {
    pub banned_by: Option<String>,

    #[serde(default = "String::new")]
    pub subreddit: String,

    #[serde(default)]
    pub score: i64,

    #[serde(default)]
    pub hidden: bool,

    pub permalink: String,

    #[serde(default = "String::new")]
    pub url: String,

    #[serde(default = "String::new")]
    pub subreddit_type: String,

    #[serde(default)]
    pub quarantine: bool,

    #[serde(default)]
    pub over_18: bool,

    #[serde(default)]
    pub created: f64,
}

pub async fn request_single_post(permalink: &str) -> Result<RedditPost, RedditError> {
    trace!("Requesting post for {}...", permalink);
    let req = crate::REQWEST_CLIENT
        .get(&format!("{}/{}.json", REDDIT_API, permalink,))
        .send()
        .await?;

    if !req.status().is_success() {
        trace!("Got unsuccessful post for {}", permalink);
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

    trace!("Deserializing post {} into container", permalink);
    let post: JsonValue = serde_json::from_str(&req.text().await?)?;
    let post = post.as_array()
        .map(|p| p.first())
        .flatten()
        .cloned()
        .ok_or(RedditError::NoPost)?;
    let mut post: Post = serde_json::from_value(post)?;

    trace!("Post {} properly fetched!", permalink);
    post.data.children.pop()
        .map(|p| p.data)
        .ok_or(RedditError::NoPost.into())
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
