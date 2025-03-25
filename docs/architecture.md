## 📚 Architecture & Features

This project is a custom-built mini-SIEM (Security Information and Event Management) system written in Rust. It ingests, parses, stores, and allows querying of logs from multiple sources via an interactive CLI.

---

### 🧩 Architecture Overview
                       ┌────────────────────────┐
                       │      config.toml       │
                       └──────────┬─────────────┘
                                  ▼
      ┌────────────┐       ┌─────────────┐       ┌────────────────┐
      │  CLI Menu  │◄──────│ AppConfig   │──────►│ Prompt Defaults│
      └────┬───────┘       └─────┬───────┘       └────────────────┘
           │                     │
           ▼                     ▼
  ┌────────────────┐     ┌────────────────────┐
  │ Start Syslog   │     │ Ingest From File   │
  │ Ingestor (TCP) │     │ (single-run)       │
  └────────┬───────┘     └─────────┬──────────┘
           ▼                        ▼
  ┌────────────────────┐   ┌────────────────────┐
  │ start_syslog_server│   │ ingest_log_file()  │
  └────────┬───────────┘   └─────────┬──────────┘
           ▼                        ▼
     ┌────────────┐           ┌────────────┐
     │ TCP/UDP    │           │ BufReader  │
     │ listeners  │           │ line-by-line
     └────┬───────┘           └────┬───────┘
          ▼                        ▼
  ┌──────────────────────┐ ┌──────────────────────┐
  │ log_parser::parse_log│ │ log_parser::parse_log│
  └────────────┬─────────┘ └────────────┬─────────┘
               ▼                        ▼
           ┌────────────┐      ┌───────────────────┐
           │ SQLite via │◄─────┤ Structured inserts│
           │ sqlx ORM   │      └───────────────────┘
           └────────────┘



---

### ✅ Features Implemented

#### 🔌 Ingestion

- ✅ **UDP & TCP Syslog Server**
  - Customizable ports (default 514)
  - Real-time streaming from network sources
- ✅ **File-Based Ingestor**
  - Reads line-by-line from a specified log file
  - Can ingest any syslog-format file

---

#### 🧠 Parsing & Normalization

- ✅ `log_parser.rs`
  - Regex-based syslog parser (RFC 3164-style)
  - Extracts:
    - `timestamp`
    - `hostname`
    - `process`
    - `message`
  - Adds protocol/source metadata per log

---

#### 💾 Storage

- ✅ SQLite + `sqlx`
  - Stores logs in a `logs` table
  - Schema:
    ```sql
    CREATE TABLE logs (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        timestamp DATETIME DEFAULT CURRENT_TIMESTAMP,
        protocol TEXT,
        source TEXT,
        message TEXT,
        parsed_timestamp TEXT,
        hostname TEXT,
        process TEXT
    );
    ```

---

#### ⚙️ Config File Support

- ✅ Uses `config` crate + `serde`
- ✅ Reads from `config.toml` at runtime
- ✅ CLI prompts use config values as defaults

#### 🖥️ Interactive CLI
- ✅ Built with dialoguer
- ✅ Actions available:

    * Start Syslog Server

    * Ingest From File

    * View Logs (search by keyword)

    * List Running Services

    * Gracefully Stop All Services

#### ⏳ Graceful Shutdown
- ✅ Uses tokio::sync::broadcast
- ✅ CLI can send shutdown signals to all running ingestion tasks
- ✅ Each server exits cleanly on shutdown

#### 🛣️ Planned Features
- [ ] Alerting based on log rules (e.g. regex matches)
- [ ] Export logs to CSV/JSON
- [ ] Live tail -f-style viewer
- [ ] Web UI with Axum or Tauri
- [ ] Sigma rule support