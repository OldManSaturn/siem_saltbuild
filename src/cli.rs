use dialoguer::{theme::ColorfulTheme, Select, Input};
use sqlx::SqlitePool;
use crate::syslog_ingestion::start_syslog_server;
use crate::file_based_ingestion::ingest_log_file;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::task::JoinHandle;
use tokio::sync::broadcast;

type TaskMap = Arc<Mutex<HashMap<String, JoinHandle<()>>>>;

pub async fn launch_cli(db_pool: SqlitePool) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let tasks: TaskMap = Arc::new(Mutex::new(HashMap::new()));
    let (shutdown_tx, _) = broadcast::channel(16);

    let options = vec![
        "Start Syslog Server (custom port)",
        "Ingest From File (custom path)",
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
                    .default(514)
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
                    .default("yourlogfile.log".into())
                    .interact_text()?;

                println!("Ingesting file: {}", path);
                ingest_log_file(&path, db_pool.clone()).await?;
            }

            // ── List Running Tasks ───────────────────────────────────
            2 => {
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
            3 => {
                println!("Stopping all services...");
                let _ = shutdown_tx.send(()); // broadcast shutdown

                let mut locked = tasks.lock().unwrap();
                for (key, handle) in locked.drain() {
                    println!("Waiting for task: {}", key);
                    let _ = handle.await;
                }
            }

            // ── Exit ─────────────────────────────────────────────────
            4 => {
                println!("Exiting.");
                break;
            }

            _ => unreachable!(),
        }
    }

    Ok(())
}
