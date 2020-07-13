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
use reqwest::Client;
use std::env;
use std::path::PathBuf;
use std::time::Duration;
use strum::IntoEnumIterator as _;
use warp::Filter as _;

pub static REQWEST_CLIENT: Lazy<Client> = Lazy::new(|| {
    use reqwest::header::{HeaderMap, HeaderValue, USER_AGENT};
    let mut headers = HeaderMap::new();
    headers.insert(
        USER_AGENT,
        HeaderValue::from_str(
            &env::var("USER_AGENT").unwrap_or_else(|_| "Mozilla/5.0 birbfetcher/bot".into()),
        )
        .expect("a valid USER_AGENT env var is required"),
    );

    Client::builder()
        .use_rustls_tls()
        .default_headers(headers)
        .build()
        .expect("a reqwest client is required")
});

#[tokio::main]
async fn main() {
    match err_main().await {
        Ok(()) => return,
        Err(e) => {
            error!("Error on running the application:");
            error!("{:?}", e);
            std::process::exit(1);
        }
    }
}

async fn err_main() -> Result<()> {
    match dotenv::dotenv() {
        Ok(_) => (),
        Err(e) if e.not_found() => (),
        Err(e) => return Err(e.into()),
    }
    pretty_env_logger::try_init()?;

    let db = env::var("DATABASE_URL").context("`DATABASE_URL` must be set")?;
    info!("Connecting to database...");
    let pool = MySqlPool::new(&db).await?;

    // {{{ Database migrations
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
    sqlx::query("INSERT IGNORE INTO `meta_version` (`version`) VALUES (0)")
        .execute(&pool)
        .await?;

    let version: (u32,) = sqlx::query_as("SELECT `version` FROM `meta_version`")
        .fetch_one(&pool)
        .await?;
    let version = version.0 as u32;

    debug!("Found DB version: {}", version);

    for migration in self::migrations::Migrations::iter().filter(|mig| (*mig as u32) > version) {
        let ver = migration as u32;
        debug!("Applying migration to V{}", ver);
        for query in migration.queries() {
            sqlx::query(&query).execute(&pool).await?;
        }
        debug!("Finished migrating to V{}, now setting version...", ver);
        sqlx::query(&format!("UPDATE `meta_version` SET `version` = {}", ver))
            .execute(&pool)
            .await?;
        debug!("Version set to {}", ver);
    }
    // }}}

    info!("Database connection created, and migrations finished!");

    let birb_dir = env::var("BIRB_DIRECTORY").unwrap_or_else(|_| "birbs".into());
    let birb_dir = PathBuf::from(birb_dir);
    if birb_dir.metadata().is_err() {
        std::fs::create_dir_all(&birb_dir)?;
    }

    let subreddits = env::var("SUBREDDITS")
        .map(|l| l.split(',').map(str::to_owned).collect::<Vec<_>>())
        .unwrap_or_else(|_| vec!["birbs".into(), "parrots".into(), "birb".into()]);

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

    // GET / - random image
    let random = warp::path::end().and_then(move || {
        let pool = pool.clone();
        let birb_dir = birb_dir.clone();
        async move { self::http::random(&pool, &birb_dir).await }
    });

    warp::serve(random).run(([0, 0, 0, 0], 8080)).await;

    Ok(())
}
