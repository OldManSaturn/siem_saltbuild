use dialoguer::{theme::ColorfulTheme, Select, Input};
use sqlx::SqlitePool;
use crate::syslog_ingestion::start_syslog_server;
use crate::file_based_ingestion::ingest_log_file;
use std::collections::HashMap;
use tokio::task::JoinHandle;
use std::sync::{Arc, Mutex};

type TaskMap = Arc<Mutex<HashMap<String, JoinHandle<()>>>>;
pub async fn launch_cli(db_pool: SqlitePool) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let tasks: TaskMap = Arc::new(Mutex::new(HashMap::new()));
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
            0 => {
                let port: u16 = Input::with_theme(&ColorfulTheme::default())
                    .with_prompt("Enter port for TCP/UDP syslog")
                    .default(514)
                    .interact_text()?;
        
                println!("Launching Syslog Server on port {}...", port);
        
                let db = db_pool.clone();
                let task_id = format!("syslog:{}:{}", port, port);
                let tasks_clone = tasks.clone();
        
                let handle = tokio::spawn(async move {
                    if let Err(e) = start_syslog_server(port, port, db).await {
                        eprintln!("Syslog server exited: {}", e);
                    }
                });
        
                tasks.lock().unwrap().insert(task_id, handle);
            }
        
            1 => {
                let path: String = Input::with_theme(&ColorfulTheme::default())
                    .with_prompt("Enter log file path")
                    .default("yourlogfile.log".into())
                    .interact_text()?;
        
                println!("Ingesting file: {}", path);
                ingest_log_file(&path, db_pool.clone()).await?;
            }
        
            2 => {
                println!("Running services:");
                for key in tasks.lock().unwrap().keys() {
                    println!("- {}", key);
                }
            }
        
            3 => {
                println!("Stopping all services...");
                let mut locked = tasks.lock().unwrap();
                for (key, handle) in locked.drain() {
                    println!("Stopping: {}", key);
                    handle.abort(); // force stop
                }
            }
        
            4 => {
                println!("Exiting.");
                break;
            }
        
            _ => unreachable!(),
        }
    }

    Ok(())
}
// ─────────────────────────────────────────