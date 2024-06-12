// src/db.rs

use duckdb::{Connection, Result};
use dotenvy::dotenv;
use std::env;
use std::sync::{Arc, Mutex};

pub fn initialize_db() -> Result<Arc<Mutex<Connection>>> {
    dotenv().ok();
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let conn = Connection::open(database_url)?;
    conn.execute(
        "CREATE TABLE IF NOT EXISTS data_mahasiswa (
            id INTEGER PRIMARY KEY,
            nama TEXT NOT NULL,
            tgl_lahir DATE NOT NULL
        )",
        [],
    )?;
    Ok(Arc::new(Mutex::new(conn)))
}
