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

mod discord;
mod error;
mod http;
mod migrations;
mod reddit;
mod tasks;
mod utils;

mod prelude {
    pub use crate::error::*;
    pub use log::{debug, error, info, trace, warn};
    pub use sqlx::prelude::*;
    pub use sqlx::MySqlPool;
    pub use std::sync::Arc;
}

use self::prelude::*;
use anyhow::{Context as _, Result};
use once_cell::sync::Lazy;
use reqwest::Client as ReqwestClient;
use serenity::client::Client as DiscordClient;
use serenity::framework::standard::StandardFramework;
use std::collections::{HashMap, HashSet};
use std::env;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::time::Duration;
use strum::IntoEnumIterator as _;
use warp::Filter as _;

/// An asynchronous reqwest client for HTTP requests.
///
/// This embeds an identifying User-Agent header into every request.
pub static REQWEST_CLIENT: Lazy<ReqwestClient> = Lazy::new(|| {
    use reqwest::header::{HeaderMap, HeaderValue, USER_AGENT};
    let mut headers = HeaderMap::new();
    headers.insert(
        USER_AGENT,
        HeaderValue::from_str(
            // Either the env var...
            &env::var("USER_AGENT")
                // or the format mandated by Reddit's API spec.
                .unwrap_or_else(|_| "Mozilla/5.0 birbfetcher/bot".into()),
        )
        // Panicking is fine because only the env var may be invalid in this case
        .expect("a valid USER_AGENT env var is required"),
    );

    ReqwestClient::builder()
        .use_rustls_tls() // TODO(Proximyst): Support OpenSSL
        .default_headers(headers)
        .build()
        .expect("a reqwest client is required")
});

#[tokio::main]
async fn main() {
    eprintln!(concat!(
        env!("CARGO_PKG_NAME"),
        " (v",
        env!("CARGO_PKG_VERSION"),
        ")"
    ));
    eprintln!(
        r#"
birbfetcher Copyright (C) 2020 Mariell Hoversholm
This program comes with ABSOLUTELY NO WARRANTY.
This is free software, and you are welcome to redistribute it
under certain conditions.
"#
    );

    match err_main().await {
        Ok(()) => return,
        Err(e) => {
            error!("Error on running the application:\n{:?}", e);
            std::process::exit(1);
        }
    }
}

async fn err_main() -> Result<()> {
    match dotenv::dotenv() {
        Ok(_) => (),
        Err(e) if e.not_found() => (), // No .env file is fine (may e.g. use Docker).
        Err(e) => return Err(e.into()),
    }
    pretty_env_logger::try_init()?;

    let db = env::var("DATABASE_URL").context("`DATABASE_URL` must be set")?;
    info!("Connecting to database...");
    let pool = MySqlPool::new(&db).await?;

    // {{{ Database migrations
    // Make sure the table exists before we try to modify and fetch from it.
    // We use a TINYINT(0) DEFAULT 0 because that makes sure only 1 row exists, ever.
    sqlx::query(
        r#"
CREATE TABLE IF NOT EXISTS `meta_version`
(
    `key` TINYINT(0) NOT NULL DEFAULT 0,
    `version` INT UNSIGNED NOT NULL,

    PRIMARY KEY (`key`)
)
    "#,
    )
    .execute(&pool)
    .await?;
    // If we fail inserting, just move along. We don't really care.
    sqlx::query("INSERT IGNORE INTO `meta_version` (`version`) VALUES (0)")
        .execute(&pool)
        .await?;

    // Fetch the version; no limit because there may only be 1 row.
    let version: (u32,) = sqlx::query_as("SELECT `version` FROM `meta_version`")
        .fetch_one(&pool)
        .await?;
    let version = version.0 as u32;

    debug!("Found DB version: {}", version);

    // Let's use a transaction to be able to rollback.
    let mut tx = pool.begin().await?;
    for migration in self::migrations::Migrations::iter()
        // We only want to migrate if the version is currently below that which we want to
        // migrate up to. I.e. if we're version 1, we don't want to migrate to version 1.
        .filter(|mig| (*mig as u32) > version)
    {
        // Migrations is #[repr(u32)].
        let ver = migration as u32;
        debug!("Applying migration to V{}", ver);

        // Queries are limited by ;s.
        for query in migration.queries() {
            // If this query fails, the transaction is rolled back.
            sqlx::query(&query).execute(&mut tx).await?;
        }

        // If this query fails, the transaction is rolled back.
        sqlx::query(&format!("UPDATE `meta_version` SET `version` = {}", ver))
            .execute(&mut tx)
            .await?;
        debug!("Migration for V{} successful.", ver);
    }
    debug!("Committing transaction...");
    tx.commit().await?;
    debug!("Migrations finished!");
    // }}}

    info!("Database connection created, and migrations finished!");

    let birb_dir = env::var("BIRB_DIRECTORY").unwrap_or_else(|_| "birbs".into());
    let birb_dir = PathBuf::from(birb_dir);
    if birb_dir.metadata().is_err() {
        // Metadata should only fail if there is no such dir.
        std::fs::create_dir_all(&birb_dir)?;
    }

    let subreddits = env::var("SUBREDDITS")
        .map(|l| l.split(',').map(str::to_owned).collect::<Vec<_>>())
        .unwrap_or_else(|_| vec!["birbs".into(), "parrots".into(), "birb".into()]);

    // {{{ Discord bot
    // TODO(Proximyst): Replace with API and separate bot/UI
    let mut discord = DiscordClient::new(
        &env::var("DISCORD_TOKEN").context("`DISCORD_TOKEN` must be set")?,
        self::discord::Handler,
    )
    .context("error creating client")?;

    let (discord_owners, bot_id) = match discord.cache_and_http.http.get_current_application_info()
    {
        Ok(info) => {
            let mut owners = HashSet::new();
            owners.insert(info.owner.id);

            (owners, info.id)
        }
        Err(why) => return Err(why.into()),
    };

    {
        let mut data = discord.data.write();
        data.insert::<self::discord::DatabaseContainer>(pool.clone());
        data.insert::<self::discord::ImagesContainer>(HashMap::new());
    }

    discord.with_framework(
        StandardFramework::new()
            .configure(|c| {
                c.prefix(&env::var("DISCORD_PREFIX").unwrap_or_else(|_| "b!".into()))
                    .on_mention(Some(bot_id))
                    .owners(discord_owners)
                    .delimiters(vec![" "])
            })
            .help(&self::discord::HELP)
            .group(&self::discord::OWNER_GROUP),
    );

    tokio::spawn(async move {
        if let Err(e) = discord.start() {
            error!("Discord error: {:?}", e);
        }
    });
    // }}}

    // {{{ Fetch posts every 10 min timer
    let timer_pool = pool.clone();
    let timer_birb_dir = birb_dir.clone();
    tokio::spawn(async move {
        let mut timer = async_timer::Interval::platform_new(Duration::from_secs(600));
        let subreddits = subreddits;
        let pool = timer_pool;
        let birb_dir = timer_birb_dir;

        loop {
            tasks::fetch_posts(&pool, &birb_dir, &subreddits).await;
            timer.as_mut().await;
        }
    });
    // }}}

    // {{{ Verify posts every 5s timer
    let timer_pool = pool.clone();
    tokio::spawn(async move {
        let mut timer = async_timer::Interval::platform_new(Duration::from_secs(5));
        let mut curr_id = 0u32;
        let pool = timer_pool;

        loop {
            let query = sqlx::query_as(
                r#"
                SELECT `id`, `permalink`
                FROM `birbs`
                WHERE `banned` = false
                    AND `verified` = false
                    AND `id` > ?
                ORDER BY `id` ASC
                LIMIT 1"#,
            )
            .bind(curr_id)
            .fetch_optional(&pool)
            .await;
            match query {
                Err(e) => error!("Could not find next ID to verify: {}", e),
                Ok(res) => {
                    match res {
                        None => {
                            curr_id = 0;
                            tokio::time::delay_for(Duration::from_secs(600)).await;
                            continue;
                        }
                        Some((id, permalink)) => {
                            // Infer the type of the result.
                            let (id, permalink): (u32, String) = (id, permalink);
                            curr_id = id;

                            if let Err(e) = tasks::process_checking(&pool, id, &permalink).await {
                                error!("Error when processing {} ({}): {}", id, permalink, e);
                            }
                        }
                    }
                }
            }
            timer.as_mut().await;
        }
    });
    // }}}

    // {{{ GET / - random image
    let root_pool = pool.clone();
    let root_birb_dir = birb_dir.clone();
    let root = warp::get().and(warp::path::end()).and_then(move || {
        let pool = root_pool.clone();
        let birb_dir = root_birb_dir.clone();
        async move { self::http::random_image(&pool, &birb_dir).await }
    });
    // }}}

    // {{{ GET /random/image - random image
    let random_pool = pool.clone();
    let random_birb_dir = birb_dir.clone();
    let random = warp::get()
        .and(warp::path("random"))
        .and(warp::path("image"))
        .and(warp::path::end())
        .and_then(move || {
            let pool = random_pool.clone();
            let birb_dir = random_birb_dir.clone();
            async move { self::http::random_image(&pool, &birb_dir).await }
        });
    // }}}

    // {{{ GET /id/:id - get image by id if unbanned
    let get_by_id_pool = pool.clone();
    let get_by_id_birb_dir = birb_dir.clone();
    let get_by_id = warp::get()
        .and(warp::path("id"))
        .and(warp::path::param())
        .and(warp::path::end())
        .and_then(move |id: u32| {
            let pool = get_by_id_pool.clone();
            let birb_dir = get_by_id_birb_dir.clone();
            async move { self::http::get_by_id(&pool, &birb_dir, id).await }
        });
    // }}}

    // {{{ GET /info/random - get random image info
    let get_random_info_pool = pool.clone();
    let get_random_info = warp::get()
        .and(warp::path("info"))
        .and(warp::path("random"))
        .and(warp::path::end())
        .and_then(move || {
            let pool = get_random_info_pool.clone();
            async move { self::http::get_random_info(&pool).await }
        });
    // }}}

    // {{{ GET /info/id/:id - get image info by id
    let get_info_by_id_pool = pool.clone();
    let get_info_by_id = warp::get()
        .and(warp::path("info"))
        .and(warp::path("id"))
        .and(warp::path::param())
        .and(warp::path::end())
        .and_then(move |id: u32| {
            let pool = get_info_by_id_pool.clone();
            async move { self::http::get_info_by_id(&pool, id).await }
        });
    // }}}

    warp::serve(
        root.or(random)
            .or(get_by_id)
            .or(get_random_info)
            .or(get_info_by_id)
            .recover(self::http::handle_rejection),
    )
    .run(
        env::var("ADDRESS")
            .unwrap_or_else(|_| "0.0.0.0:8080".into())
            .parse::<SocketAddr>()?,
    )
    .await;

    Ok(())
}
