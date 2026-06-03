use axum::{
    extract::{Path, State}, // Path = pull values out of the URL, like /todos/{id}
    http::StatusCode,
    routing::get,
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};

// ---- DATA MODELS ----

#[derive(Clone, Serialize, Deserialize)]
struct Todo {
    id: u32,
    title: String,
    done: bool,
}

// Body for creating a todo (server assigns id, defaults done=false)
#[derive(Deserialize)]
struct CreateTodo {
    title: String,
}

// Body for updating a todo. Both fields are Option, so the client can send
// just the ones they want to change (partial update, like PATCH semantics).
#[derive(Deserialize)]
struct UpdateTodo {
    title: Option<String>,
    done: Option<bool>,
}

// ---- SHARED STATE ----
type Db = Arc<Mutex<Vec<Todo>>>;

#[tokio::main]
async fn main() {
    let db: Db = Arc::new(Mutex::new(Vec::new()));

    let app = Router::new()
        // "/todos": list all (GET) + create (POST)
        .route("/todos", get(list_todos).post(create_todo))
        // "/todos/{id}": get one (GET) + update (PUT) + delete (DELETE)
        .route(
            "/todos/{id}",
            get(get_todo).put(update_todo).delete(delete_todo),
        )
        .with_state(db);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:4000")
        .await
        .unwrap();
    println!("🚀 Server running on http://127.0.0.1:4000");
    axum::serve(listener, app).await.unwrap();
}

// ---- HANDLERS ----

// GET /todos  -> all todos
async fn list_todos(State(db): State<Db>) -> Json<Vec<Todo>> {
    let todos = db.lock().unwrap();
    Json(todos.clone())
}

// POST /todos  -> create
async fn create_todo(
    State(db): State<Db>,
    Json(payload): Json<CreateTodo>,
) -> (StatusCode, Json<Todo>) {
    let mut todos = db.lock().unwrap();
    let new_id = todos.len() as u32 + 1;
    let todo = Todo {
        id: new_id,
        title: payload.title,
        done: false,
    };
    todos.push(todo.clone());
    (StatusCode::CREATED, Json(todo))
}

// GET /todos/{id}  -> one todo, or 404
// Return type Result<Json<Todo>, StatusCode>:
//   Ok(Json(todo)) = 200 with the todo
//   Err(StatusCode::NOT_FOUND) = 404
async fn get_todo(
    State(db): State<Db>,
    Path(id): Path<u32>,
) -> Result<Json<Todo>, StatusCode> {
    let todos = db.lock().unwrap();
    // .iter().find(...) returns Option<&Todo>; map it into our Result
    match todos.iter().find(|t| t.id == id) {
        Some(todo) => Ok(Json(todo.clone())),
        None => Err(StatusCode::NOT_FOUND),
    }
}

// PUT /todos/{id}  -> update title and/or done, or 404
async fn update_todo(
    State(db): State<Db>,
    Path(id): Path<u32>,
    Json(payload): Json<UpdateTodo>,
) -> Result<Json<Todo>, StatusCode> {
    let mut todos = db.lock().unwrap();
    // iter_mut() gives mutable references so we can change the todo in place
    match todos.iter_mut().find(|t| t.id == id) {
        Some(todo) => {
            // "if let Some(x)" = run this block only when the Option has a value
            if let Some(title) = payload.title {
                todo.title = title;
            }
            if let Some(done) = payload.done {
                todo.done = done;
            }
            Ok(Json(todo.clone()))
        }
        None => Err(StatusCode::NOT_FOUND),
    }
}

// DELETE /todos/{id}  -> 204 No Content, or 404
async fn delete_todo(State(db): State<Db>, Path(id): Path<u32>) -> StatusCode {
    let mut todos = db.lock().unwrap();
    let len_before = todos.len();
    // retain() keeps only the items where the closure is true — i.e. drops the matching id
    todos.retain(|t| t.id != id);
    if todos.len() < len_before {
        StatusCode::NO_CONTENT // something was removed
    } else {
        StatusCode::NOT_FOUND // nothing matched that id
    }
}
