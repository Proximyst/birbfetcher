mod error;
mod migrations;

mod prelude {
    pub use crate::error::*;
    pub use log::{debug, error, info, trace, warn};
    pub use sqlx::prelude::*;
    pub use sqlx::MySqlPool;
    pub use std::sync::Arc;

    pub type Pool = Arc<MySqlPool>;
}

use self::prelude::*;
use anyhow::{Context as _, Result};
use std::env;
use strum::IntoEnumIterator as _;

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
    env_logger::try_init()?;

    let db = env::var("DATABASE_URL").context("`DATABASE_URL` must be set")?;
    info!("Connecting to database...");
    let pool = MySqlPool::new(&db).await?;

    sqlx::query(
        r#"
CREATE TABLE IF NOT EXISTS `meta_version`
(
    `key` TINYINT(1) NOT NULL DEFAULT 0,
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
        debug!("Finished migrating to V{}", ver);
    }
    
    info!("Database connection created, and migrations finished!");

    Ok(())
}
