use anyhow::{Context, Result};
use sqlx::PgPool;

const DEFAULT_DATABASE_URL: &str = "postgres://address:address@localhost:5432/address_wise";
#[tokio::main]
async fn main() -> Result<()> {
    let database_url =
        std::env::var("DATABASE_URL").unwrap_or_else(|_| DEFAULT_DATABASE_URL.to_string());
    let pool = PgPool::connect(&database_url)
        .await
        .with_context(|| format!("failed to connect to PostgreSQL at {database_url}"))?;

    let updated = sqlx::query(
        r#"
        UPDATE addresses
        SET search_text = trim(
            regexp_replace(
                regexp_replace(
                    regexp_replace(
                        translate(
                            lower(full_address),
                            'áàâäãåąæçčćďéěèêëęíìîïľĺłñňóòôöõøřšśťúůùûüýÿžźż',
                            'aaaaaaaacccdeeeeeeiiiilllnnoooooorsstuuuuuyyzzz'
                        ),
                        '[[:space:],./#-]+',
                        ' ',
                        'g'
                    ),
                    '[^a-z0-9 ]+',
                    '',
                    'g'
                ),
                ' +',
                ' ',
                'g'
            )
        )
        WHERE search_text !~ '^[ -~]*$'
        "#,
    )
    .execute(&pool)
    .await
    .context("failed to rewrite stored search_text values")?
    .rows_affected();

    pool.close().await;

    println!("updated={updated}");
    Ok(())
}
