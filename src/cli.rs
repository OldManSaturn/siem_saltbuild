use dialoguer::{theme::ColorfulTheme, Select, Input};
use sqlx::SqlitePool;
use crate::syslog_ingestion::start_syslog_server;
use crate::file_based_ingestion::ingest_log_file;

pub async fn launch_cli(db_pool: SqlitePool) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let options = vec![
        "Start Syslog Server (custom port)",
        "Ingest From File (custom path)",
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
                tokio::spawn(async move {
                    if let Err(e) = start_syslog_server(port, port, db).await {
                        eprintln!("Syslog server exited with error: {}", e);
                    }
                });
                    
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
                println!("Exiting.");
                break;
            }
            _ => unreachable!(),
        }
    }

    Ok(())
}
// ─────────────────────────────────────────