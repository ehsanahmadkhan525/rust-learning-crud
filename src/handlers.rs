// All the request handlers. Each is `pub` so main.rs can wire it to a route.

use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use sqlx::SqlitePool;
use tracing::info;

// Pull our data shapes in from the models module (crate = "this project").
use crate::models::{CreateTodo, Todo, UpdateTodo};

// GET /todos
pub async fn list_todos(State(pool): State<SqlitePool>) -> Result<Json<Vec<Todo>>, StatusCode> {
    let todos = sqlx::query_as::<_, Todo>("SELECT id, title, done FROM todos")
        .fetch_all(&pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(todos))
}

// POST /todos
pub async fn create_todo(
    State(pool): State<SqlitePool>,
    Json(payload): Json<CreateTodo>,
) -> Result<(StatusCode, Json<Todo>), StatusCode> {
    // VALIDATION: reject a missing/blank title.
    if payload.title.trim().is_empty() {
        return Err(StatusCode::BAD_REQUEST);
    }

    let todo = sqlx::query_as::<_, Todo>(
        "INSERT INTO todos (title, done) VALUES (?, 0) RETURNING id, title, done",
    )
    .bind(payload.title)
    .fetch_one(&pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    info!(id = todo.id, title = %todo.title, "created todo");
    Ok((StatusCode::CREATED, Json(todo)))
}

// GET /todos/{id}
pub async fn get_todo(
    State(pool): State<SqlitePool>,
    Path(id): Path<i64>,
) -> Result<Json<Todo>, StatusCode> {
    let todo = sqlx::query_as::<_, Todo>("SELECT id, title, done FROM todos WHERE id = ?")
        .bind(id)
        .fetch_optional(&pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match todo {
        Some(t) => Ok(Json(t)),
        None => Err(StatusCode::NOT_FOUND),
    }
}

// PUT /todos/{id}
pub async fn update_todo(
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

    let existing = sqlx::query_as::<_, Todo>("SELECT id, title, done FROM todos WHERE id = ?")
        .bind(id)
        .fetch_optional(&pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let mut todo = existing.ok_or(StatusCode::NOT_FOUND)?;

    if let Some(title) = payload.title {
        todo.title = title;
    }
    if let Some(done) = payload.done {
        todo.done = done;
    }

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
pub async fn delete_todo(
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
        Ok(StatusCode::NO_CONTENT)
    } else {
        info!(id, "delete failed: no such todo");
        Err(StatusCode::NOT_FOUND)
    }
}

// ---- TESTS ----
#[cfg(test)]
mod tests {
    use super::*; // handlers + their imported types (CreateTodo, Todo, State, Json, Path...)
    use crate::db::init_db; // init_db lives in the db module, so import it explicitly

    #[tokio::test]
    async fn create_then_list_works() {
        let pool = init_db("sqlite::memory:").await;

        let (status, Json(todo)) =
            create_todo(State(pool.clone()), Json(CreateTodo { title: "Test".into() }))
                .await
                .unwrap();

        assert_eq!(status, StatusCode::CREATED);
        assert_eq!(todo.title, "Test");
        assert_eq!(todo.id, 1);
        assert!(!todo.done);

        let Json(todos) = list_todos(State(pool)).await.unwrap();
        assert_eq!(todos.len(), 1);
    }

    #[tokio::test]
    async fn empty_title_is_rejected() {
        let pool = init_db("sqlite::memory:").await;
        let result = create_todo(State(pool), Json(CreateTodo { title: "   ".into() })).await;
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
        // `let _ =` = "I'm deliberately ignoring the return value" (silences the warning)
        let _ = create_todo(State(pool.clone()), Json(CreateTodo { title: "Bye".into() }))
            .await
            .unwrap();
        let status = delete_todo(State(pool.clone()), Path(1)).await.unwrap();
        assert_eq!(status, StatusCode::NO_CONTENT);

        let second = delete_todo(State(pool), Path(1)).await;
        assert_eq!(second.unwrap_err(), StatusCode::NOT_FOUND);
    }
}
