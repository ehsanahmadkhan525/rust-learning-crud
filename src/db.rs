// Database setup, shared by main (real file db) and tests (in-memory db).

use sqlx::sqlite::SqlitePoolOptions;
use sqlx::SqlitePool;

// Connect to a database and make sure the `todos` table exists.
// max_connections(1): SQLite serializes writes anyway, and it keeps an
// in-memory ("sqlite::memory:") database alive for the pool's whole life.
pub async fn init_db(url: &str) -> SqlitePool {
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect(url)
        .await
        .unwrap();

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS todos (
            id    INTEGER PRIMARY KEY AUTOINCREMENT,
            title TEXT    NOT NULL,
            done  BOOLEAN NOT NULL DEFAULT 0
        )",
    )
    .execute(&pool)
    .await
    .unwrap();

    pool
}
