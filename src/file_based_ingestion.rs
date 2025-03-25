use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use sqlx::SqlitePool;
use crate::log_parser::{parse_log, ParsedLog};

pub async fn ingest_log_file(path: &str, db_pool: SqlitePool) -> Result<(), Box<dyn Error + Send + Sync>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);

    for line_result in reader.lines() {
        let line = line_result?;
        let parsed = parse_log("FILE", path, &line);
        sqlx::query(
            "INSERT INTO logs (protocol, source, message, parsed_timestamp, hostname, process)
             VALUES (?, ?, ?, ?, ?, ?)"
        )
        .bind(&parsed.protocol)
        .bind(&parsed.source)
        .bind(&parsed.message)
        .bind(&parsed.timestamp)
        .bind(&parsed.hostname)
        .bind(&parsed.process)
        .execute(&db_pool)
        .await?;
    }

    println!("Finished ingesting file: {}", path);
    Ok(())
}
