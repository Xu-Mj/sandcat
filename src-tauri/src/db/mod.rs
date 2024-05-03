pub mod message;
use sqlx::{sqlite::SqlitePoolOptions, SqlitePool};

pub async fn establish_connection() -> SqlitePool {
    dotenv::dotenv().expect("Unable to load environment variables from .env file");

    let db_url = std::env::var("DATABASE_URL").expect("Unable to read DATABASE_URL env var");

    SqlitePoolOptions::new()
        .max_connections(10)
        .connect(&db_url)
        .await
        .expect("Unable to connect to Postgres")
}
