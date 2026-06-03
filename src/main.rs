use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::get,
    Json, Router,
};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool; // a pool of reusable database connections

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
    // Connect to (and create, thanks to ?mode=rwc) a SQLite file `todos.db`.
    let pool = SqlitePool::connect("sqlite:todos.db?mode=rwc")
        .await
        .unwrap();

    // Create the table once if it doesn't exist yet.
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

    // The shared state is now the connection pool (it's cheap to clone & share).
    let app = Router::new()
        .route("/todos", get(list_todos).post(create_todo))
        .route(
            "/todos/{id}",
            get(get_todo).put(update_todo).delete(delete_todo),
        )
        .with_state(pool);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:4000")
        .await
        .unwrap();
    println!("🚀 Server running on http://127.0.0.1:4000 (data persists in todos.db)");
    axum::serve(listener, app).await.unwrap();
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
    let todo = sqlx::query_as::<_, Todo>(
        "INSERT INTO todos (title, done) VALUES (?, 0) RETURNING id, title, done",
    )
    .bind(payload.title) // ? placeholder is filled by .bind() (safe from SQL injection)
    .fetch_one(&pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
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
        Ok(StatusCode::NO_CONTENT) // deleted something
    } else {
        Err(StatusCode::NOT_FOUND) // no row matched
    }
}
