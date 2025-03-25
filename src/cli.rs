use dialoguer::{theme::ColorfulTheme, Select, Input};
use sqlx::SqlitePool;
use crate::syslog_ingestion::start_syslog_server;
use crate::file_based_ingestion::ingest_log_file;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::task::JoinHandle;
use tokio::sync::broadcast;
use crate::config::AppConfig;

type TaskMap = Arc<Mutex<HashMap<String, JoinHandle<()>>>>;

pub async fn launch_cli(db_pool: SqlitePool, config: AppConfig,) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let tasks: TaskMap = Arc::new(Mutex::new(HashMap::new()));
    let (shutdown_tx, _) = broadcast::channel(16);

    let options = vec![
        "Start Syslog Server (custom port)",
        "Ingest From File (custom path)",
        "View Logs",
        "List Running Services",
        "Stop All Services",
        "Exit",
    ];

    loop {
        let selection = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("Select an action")
            .default(0)
            .items(&options)
            .interact()?;

        match selection {
            // ── Start Syslog Ingestor ─────────────────────────────────
            0 => {
                let port: u16 = Input::with_theme(&ColorfulTheme::default())
                    .with_prompt("Enter port for TCP/UDP syslog")
                    .default(config.syslog_tcp_port)
                    .interact_text()?;


                println!("Launching Syslog Server on port {}...", port);

                let db = db_pool.clone();
                let mut shutdown_rx = shutdown_tx.subscribe();
                let task_id = format!("syslog:{}:{}", port, port);

                let handle = tokio::spawn(async move {
                    if let Err(e) = start_syslog_server(port, port, db, shutdown_rx).await {
                        eprintln!("Syslog server exited: {}", e);
                    }
                });

                tasks.lock().unwrap().insert(task_id, handle);
            }

            // ── Ingest Log File ──────────────────────────────────────
            1 => {
                let path: String = Input::with_theme(&ColorfulTheme::default())
                    .with_prompt("Enter log file path")
                    .default(config.log_file_path.clone())
                    .interact_text()?;


                println!("Ingesting file: {}", path);
                ingest_log_file(&path, db_pool.clone()).await?;
            }

            // ── View Logs ────────────────────────────────────────────
            2 => {
                handle_log_query(&db_pool).await?;
            }

            // ── List Running Tasks ───────────────────────────────────
            3 => {
                let locked = tasks.lock().unwrap();
                if locked.is_empty() {
                    println!("No active services.");
                } else {
                    println!("Running services:");
                    for key in locked.keys() {
                        println!("- {}", key);
                    }
                }
            }

            // ── Stop All ─────────────────────────────────────────────
            4 => {
                println!("Stopping all services...");
                let _ = shutdown_tx.send(()); // broadcast shutdown

                let mut locked = tasks.lock().unwrap();
                for (key, handle) in locked.drain() {
                    println!("Waiting for task: {}", key);
                    let _ = handle.await;
                }
            }

            // ── Exit ─────────────────────────────────────────────────
            5 => {
                println!("Exiting.");
                break;
            }

            _ => unreachable!(),
        }
    }

    Ok(())
}

// ── View Logs Helper ───────────────────────────────────────────────
async fn handle_log_query(db_pool: &SqlitePool) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let keyword: String = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("Enter keyword to search (or leave empty for all)")
        .allow_empty(true)
        .interact_text()?;

    let query = if keyword.trim().is_empty() {
        "SELECT timestamp, protocol, source, message FROM logs ORDER BY id DESC LIMIT 20".to_string()
    } else {
        "SELECT timestamp, protocol, source, message FROM logs WHERE message LIKE ? ORDER BY id DESC LIMIT 20".to_string()
    };

    let rows = if keyword.trim().is_empty() {
        sqlx::query_as::<_, (Option<String>, String, String, String)>(&query)
            .fetch_all(db_pool)
            .await?
    } else {
        sqlx::query_as::<_, (Option<String>, String, String, String)>(&query)
            .bind(format!("%{}%", keyword))
            .fetch_all(db_pool)
            .await?
    };

    println!("\nRecent Logs:");
    for (timestamp, protocol, source, message) in rows {
        let ts = timestamp.unwrap_or_else(|| "Unknown".to_string());
        println!("[{}] [{}] {} → {}", ts, protocol, source, message);
    }

    Ok(())
}
// ────────────────────────────────────────────────────────────────────