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
use crate::reddit::*;
use sha2::Digest as _;
use std::path::PathBuf;
use std::time::Instant;

pub async fn fetch_posts(db: &MySqlPool, birb_dir: &PathBuf, subreddits: &[String]) {
    let mut posts = Vec::with_capacity(subreddits.len() * 100);
    for sub in subreddits {
        match request_posts(sub, PostType::Hot).await {
            Ok(v) => posts.extend(v),
            Err(e) => error!("Could not fetch any posts for {}/hot: {}", sub, e),
        }
        match request_posts(sub, PostType::New).await {
            Ok(v) => posts.extend(v),
            Err(e) => error!("Could not fetch any posts for {}/new: {}", sub, e),
        }
    }

    info!(
        "Fetched {} posts, will now start processing them all...",
        posts.len()
    );
    let start = Instant::now();
    for post in posts.iter().filter(|p| p.is_safe()) {
        match process_post(db, birb_dir, post).await {
            Ok(()) => (),
            Err(e) => warn!("Error on processing post ({:?}): {}", post, e),
        }
    }
    let elapsed = start.elapsed();
    info!(
        "Finished processing posts! Took {} seconds.",
        elapsed.as_secs()
    );
}

async fn process_post(
    db: &MySqlPool,
    birb_dir: &PathBuf,
    post: &RedditPost,
) -> Result<(), ProcessingError> {
    let image = crate::REQWEST_CLIENT.get(&post.url).send().await?;
    if !image.status().is_success() {
        return Err(ProcessingError::Unsuccessful(image.status()));
    }

    // I'm not particularly interested in *what* went wrong.
    let content_type = image
        .headers()
        .get(reqwest::header::CONTENT_TYPE)
        .ok_or(ProcessingError::InvalidContentType)?;
    let content_type = content_type
        .to_str()
        .map_err(|_| ProcessingError::InvalidContentType)?
        .to_owned();

    let body = image.bytes().await?;

    let hash = crate::utils::sha256(|h| h.update(&body));
    let hash_hex = hex::encode_upper(&hash);
    let path = birb_dir.join(&hash_hex);

    if path.metadata().is_ok() {
        return Err(ProcessingError::Duplicate);
    }

    std::fs::write(path, &body)?;

    sqlx::query(
        "INSERT INTO birbs (hash, permalink, source_url, content_type) VALUES (?, ?, ?, ?)",
    )
    .bind(hash)
    .bind(&post.permalink)
    .bind(&post.url)
    .bind(content_type)
    .execute(db)
    .await?;

    Ok(())
}
