mod api;
mod db;
mod error;
mod index;
mod models;
mod normalize;
mod repository;
mod service;
mod state;

use ntex::web::{App, HttpServer, middleware};
use repository::AddressRepository;
use service::AddressService;
use state::AppState;
use tracing_subscriber::EnvFilter;
use crate::index::AddressIndex;
use futures::StreamExt;
use tantivy::{doc, IndexWriter};

use crate::error::AppError;

const DEFAULT_DATABASE_URL: &str = "postgres://address:address@127.0.0.1:5432/address_wise";
const DEFAULT_BIND: &str = "0.0.0.0:8080";
const DEFAULT_TANTIVY_WRITER_MEMORY_BYTES: usize = 1_000_000_000;

async fn sync_postgres_to_tantivy(repository: &AddressRepository, address_index: &AddressIndex) -> Result<(), AppError> {
    let writer_memory_budget = env_usize(
        "TANTIVY_WRITER_MEMORY_BYTES",
        DEFAULT_TANTIVY_WRITER_MEMORY_BYTES,
    );
    let mut writer: IndexWriter = address_index.index.writer(writer_memory_budget)?;
    let f = &address_index.fields;
    
    let mut stream = repository.stream_all();
    let mut count = 0;
    
    while let Some(row) = stream.next().await {
        let row = row?;
        
        writer.add_document(doc!(
            f.id => row.id as u64,
            f.search_text => row.search_text,
        ))?;
        
        count += 1;
        if count % 100_000 == 0 {
            tracing::info!(count, "indexed documents...");
        }
    }
    
    tracing::info!(count, "committing index...");
    writer.commit()?;
    tracing::info!("index committed successfully.");
    
    Ok(())
}

#[ntex::main]
async fn main() -> Result<(), AppError> {
    dotenvy::dotenv().ok();
    init_tracing();

    let database_url =
        std::env::var("DATABASE_URL").unwrap_or_else(|_| DEFAULT_DATABASE_URL.to_string());
    let bind = std::env::var("API_BIND").unwrap_or_else(|_| DEFAULT_BIND.to_string());
    let index_path = std::env::var("INDEX_PATH").unwrap_or_else(|_| ".tantivy_index".to_string());
    
    let max_connections = env_u32("DB_MAX_CONNECTIONS", 10);
    let candidate_limit = env_i64("ADDRESS_RESOLVE_CANDIDATE_LIMIT", 800);
    let trigram_threshold = env_f64("PG_TRGM_SIMILARITY_THRESHOLD", 0.3);

    let pool = db::connect_with_retry(&database_url, max_connections, trigram_threshold).await?;
    db::run_migrations(&pool).await?;

    let address_index = AddressIndex::open_or_create(&index_path)?;
    
    let repository = AddressRepository::new(pool);
    
    // Check if we need to index
    if address_index.reader.searcher().num_docs() == 0 {
        tracing::info!("index is empty, starting initial indexing from postgres...");
        sync_postgres_to_tantivy(&repository, &address_index).await?;
    }

    let state = AppState {
        addresses: AddressService::new(repository, address_index, candidate_limit),
    };

    tracing::info!(%bind, "starting ntex server");

    HttpServer::new(move || {
        let state = state.clone();

        async move {
            App::new()
                .state(state)
                .middleware(middleware::Logger::default())
                .configure(api::configure)
        }
    })
    .bind(&bind)?
    .run()
    .await?;

    Ok(())
}

fn init_tracing() {
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("address_wise=info,ntex=info,sqlx=warn"));

    tracing_subscriber::fmt().with_env_filter(filter).init();
}

fn env_u32(name: &str, default: u32) -> u32 {
    std::env::var(name)
        .ok()
        .and_then(|value| value.parse::<u32>().ok())
        .unwrap_or(default)
}

fn env_i64(name: &str, default: i64) -> i64 {
    std::env::var(name)
        .ok()
        .and_then(|value| value.parse::<i64>().ok())
        .filter(|value| *value > 0)
        .unwrap_or(default)
}

fn env_f64(name: &str, default: f64) -> f64 {
    std::env::var(name)
        .ok()
        .and_then(|value| value.parse::<f64>().ok())
        .unwrap_or(default)
}

fn env_usize(name: &str, default: usize) -> usize {
    std::env::var(name)
        .ok()
        .and_then(|value| value.parse::<usize>().ok())
        .filter(|value| *value > 0)
        .unwrap_or(default)
}
