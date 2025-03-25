use sqlx::SqlitePool;
use std::error::Error;

pub async fn init_db() -> Result<SqlitePool, Box<dyn Error + Send + Sync>> {
    let pool = SqlitePool::connect("sqlite://./logs.db").await?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS logs (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            timestamp DATETIME DEFAULT CURRENT_TIMESTAMP,
            protocol TEXT,
            source TEXT,
            message TEXT,
            parsed_timestamp TEXT,
            hostname TEXT,
            process TEXT
        );
        "#
    )
    .execute(&pool)
    .await?;

    Ok(pool)
}
