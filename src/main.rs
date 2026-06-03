use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::get,
    Json, Router,
};
use serde::{Deserialize, Serialize};
use sqlx::sqlite::SqlitePoolOptions;
use sqlx::SqlitePool; // a pool of reusable database connections
use tower_http::trace::TraceLayer; // auto-logs every HTTP request/response
use tracing::info; // info!(...) = print an informational log line

// ---- DATA MODELS ----

// FromRow = sqlx can build a Todo directly from a database row.
#[derive(Clone, Serialize, sqlx::FromRow)]
struct Todo {
    id: i64, // SQLite integers are 64-bit, so we use i64 here
    title: String,
    done: bool,
}

#[derive(Deserialize)]
struct CreateTodo {
    title: String,
}

#[derive(Deserialize)]
struct UpdateTodo {
    title: Option<String>,
    done: Option<bool>,
}

#[tokio::main]
async fn main() {
    // Turn on logging. The filter string decides what gets printed:
    //   info                  = show info-level logs from our app
    //   tower_http=debug      = show each HTTP request/response
    // You can override it at runtime with the RUST_LOG environment variable.
    tracing_subscriber::fmt()
        .with_env_filter("info,tower_http=debug")
        .init();

    // Connect to the real database file `todos.db`.
    let pool = init_db("sqlite:todos.db?mode=rwc").await;

    // The shared state is now the connection pool (it's cheap to clone & share).
    let app = Router::new()
        .route("/todos", get(list_todos).post(create_todo))
        .route(
            "/todos/{id}",
            get(get_todo).put(update_todo).delete(delete_todo),
        )
        .layer(TraceLayer::new_for_http()) // log every request that comes in
        .with_state(pool);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:4000")
        .await
        .unwrap();
    info!("🚀 Server running on http://127.0.0.1:4000 (data persists in todos.db)");
    axum::serve(listener, app).await.unwrap();
}

// Connect to a database and make sure the `todos` table exists.
// Used by main (with a file) AND by tests (with an in-memory db).
// max_connections(1): SQLite serializes writes anyway, and it keeps an
// in-memory ("sqlite::memory:") database alive for the pool's whole life.
async fn init_db(url: &str) -> SqlitePool {
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

// ---- HANDLERS ----
// Every DB call is async (.await) and can fail, so handlers return Result.
// `.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?` = "if the query errored,
// turn it into a 500 and return early" (the `?` is Rust's error shortcut).

// GET /todos
async fn list_todos(State(pool): State<SqlitePool>) -> Result<Json<Vec<Todo>>, StatusCode> {
    let todos = sqlx::query_as::<_, Todo>("SELECT id, title, done FROM todos")
        .fetch_all(&pool) // fetch_all -> Vec<Todo>
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(todos))
}

// POST /todos  (RETURNING gives us back the row we just inserted, with its new id)
async fn create_todo(
    State(pool): State<SqlitePool>,
    Json(payload): Json<CreateTodo>,
) -> Result<(StatusCode, Json<Todo>), StatusCode> {
    // VALIDATION: reject a missing/blank title.
    // .trim() removes surrounding spaces, so "   " counts as empty too.
    if payload.title.trim().is_empty() {
        return Err(StatusCode::BAD_REQUEST); // 400 — the client sent bad data
    }

    let todo = sqlx::query_as::<_, Todo>(
        "INSERT INTO todos (title, done) VALUES (?, 0) RETURNING id, title, done",
    )
    .bind(payload.title) // ? placeholder is filled by .bind() (safe from SQL injection)
    .fetch_one(&pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    // {} fills in the value; the part before = is just a label in the log line
    info!(id = todo.id, title = %todo.title, "created todo");
    Ok((StatusCode::CREATED, Json(todo)))
}

// GET /todos/{id}
async fn get_todo(
    State(pool): State<SqlitePool>,
    Path(id): Path<i64>,
) -> Result<Json<Todo>, StatusCode> {
    let todo = sqlx::query_as::<_, Todo>("SELECT id, title, done FROM todos WHERE id = ?")
        .bind(id)
        .fetch_optional(&pool) // fetch_optional -> Option<Todo> (None if no row)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match todo {
        Some(t) => Ok(Json(t)),
        None => Err(StatusCode::NOT_FOUND),
    }
}

// PUT /todos/{id}  (fetch, apply partial changes, save)
async fn update_todo(
    State(pool): State<SqlitePool>,
    Path(id): Path<i64>,
    Json(payload): Json<UpdateTodo>,
) -> Result<Json<Todo>, StatusCode> {
    // VALIDATION: if a title was sent, it must not be blank.
    if let Some(title) = &payload.title {
        if title.trim().is_empty() {
            return Err(StatusCode::BAD_REQUEST);
        }
    }

    // 1. Load the existing row (404 if it doesn't exist)
    let existing = sqlx::query_as::<_, Todo>("SELECT id, title, done FROM todos WHERE id = ?")
        .bind(id)
        .fetch_optional(&pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let mut todo = existing.ok_or(StatusCode::NOT_FOUND)?; // None -> return 404

    // 2. Apply only the fields the client actually sent
    if let Some(title) = payload.title {
        todo.title = title;
    }
    if let Some(done) = payload.done {
        todo.done = done;
    }

    // 3. Persist the change
    sqlx::query("UPDATE todos SET title = ?, done = ? WHERE id = ?")
        .bind(&todo.title)
        .bind(todo.done)
        .bind(id)
        .execute(&pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(todo))
}

// DELETE /todos/{id}
async fn delete_todo(
    State(pool): State<SqlitePool>,
    Path(id): Path<i64>,
) -> Result<StatusCode, StatusCode> {
    let result = sqlx::query("DELETE FROM todos WHERE id = ?")
        .bind(id)
        .execute(&pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    if result.rows_affected() > 0 {
        info!(id, "deleted todo");
        Ok(StatusCode::NO_CONTENT) // deleted something
    } else {
        info!(id, "delete failed: no such todo");
        Err(StatusCode::NOT_FOUND) // no row matched
    }
}

// ---- TESTS ----
// #[cfg(test)] = "only compile this block when running `cargo test`" (not in the real app).
#[cfg(test)]
mod tests {
    use super::*; // bring everything above (Todo, handlers, init_db...) into the tests

    // We call the handler functions DIRECTLY, building their extractor arguments by hand:
    //   State(pool)          instead of the framework injecting it
    //   Json(CreateTodo{..}) instead of a real JSON request body
    //   Path(id)             instead of a real URL
    // Each test gets its OWN fresh in-memory database, so they never interfere.

    #[tokio::test] // like #[test] but for async functions
    async fn create_then_list_works() {
        let pool = init_db("sqlite::memory:").await;

        // Create a todo
        let (status, Json(todo)) =
            create_todo(State(pool.clone()), Json(CreateTodo { title: "Test".into() }))
                .await
                .unwrap();

        assert_eq!(status, StatusCode::CREATED); // got 201?
        assert_eq!(todo.title, "Test"); // right title?
        assert_eq!(todo.id, 1); // first row gets id 1
        assert!(!todo.done); // defaults to not-done

        // List should now contain exactly one todo
        let Json(todos) = list_todos(State(pool)).await.unwrap();
        assert_eq!(todos.len(), 1);
    }

    #[tokio::test]
    async fn empty_title_is_rejected() {
        let pool = init_db("sqlite::memory:").await;

        let result =
            create_todo(State(pool), Json(CreateTodo { title: "   ".into() })).await;

        // We expect an error, specifically 400 Bad Request
        assert_eq!(result.unwrap_err(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn get_missing_todo_returns_404() {
        let pool = init_db("sqlite::memory:").await;

        let result = get_todo(State(pool), Path(999)).await;

        assert_eq!(result.unwrap_err(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn delete_then_gone() {
        let pool = init_db("sqlite::memory:").await;

        // Make one, then delete it
        create_todo(State(pool.clone()), Json(CreateTodo { title: "Bye".into() }))
            .await
            .unwrap();
        let status = delete_todo(State(pool.clone()), Path(1)).await.unwrap();
        assert_eq!(status, StatusCode::NO_CONTENT);

        // Deleting again should now be a 404
        let second = delete_todo(State(pool), Path(1)).await;
        assert_eq!(second.unwrap_err(), StatusCode::NOT_FOUND);
    }
}
