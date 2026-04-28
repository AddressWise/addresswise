use std::time::Duration;

use sqlx::postgres::PgPoolOptions;
use sqlx::{Executor, PgPool};

pub async fn connect_with_retry(
    database_url: &str,
    max_connections: u32,
    trigram_similarity_threshold: f64,
) -> Result<PgPool, sqlx::Error> {
    let mut last_error = None;
    let threshold = trigram_similarity_threshold.clamp(0.01, 1.0);
    let after_connect_sql = format!("SET pg_trgm.similarity_threshold = {threshold}");

    for attempt in 1..=20 {
        let pool = PgPoolOptions::new()
            .max_connections(max_connections)
            .after_connect({
                let after_connect_sql = after_connect_sql.clone();
                move |conn, _meta| {
                    let after_connect_sql = after_connect_sql.clone();
                    Box::pin(async move {
                        conn.execute(after_connect_sql.as_str()).await?;
                        Ok(())
                    })
                }
            })
            .connect(database_url)
            .await;

        match pool {
            Ok(pool) => return Ok(pool),
            Err(err) => {
                tracing::warn!(attempt, error = %err, "database connection failed");
                last_error = Some(err);
                tokio::time::sleep(Duration::from_secs(2)).await;
            }
        }
    }

    Err(last_error.expect("retry loop should capture a database error"))
}

pub async fn run_migrations(pool: &PgPool) -> Result<(), sqlx::migrate::MigrateError> {
    sqlx::migrate!("./migrations").run(pool).await
}
