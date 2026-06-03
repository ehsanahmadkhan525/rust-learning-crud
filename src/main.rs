// Declare the other files as modules of this project.
// `mod x;` tells Rust "there's a file src/x.rs — include it."
mod db;
mod handlers;
mod models;

use axum::{routing::get, Router};
use tower_http::trace::TraceLayer;
use tracing::info;

// Bring the things we need into scope. `crate::` means "from this project".
use crate::db::init_db;
use crate::handlers::{create_todo, delete_todo, get_todo, list_todos, update_todo};

#[tokio::main]
async fn main() {
    // Turn on logging (info level for us, request logs from tower_http).
    tracing_subscriber::fmt()
        .with_env_filter("info,tower_http=debug")
        .init();

    // Connect to the real database file `todos.db`.
    let pool = init_db("sqlite:todos.db?mode=rwc").await;

    // Wire up routes to handlers (which now live in handlers.rs).
    let app = Router::new()
        .route("/todos", get(list_todos).post(create_todo))
        .route(
            "/todos/{id}",
            get(get_todo).put(update_todo).delete(delete_todo),
        )
        .layer(TraceLayer::new_for_http())
        .with_state(pool);

    // 0.0.0.0 = listen on ALL network interfaces (required so Docker can reach it).
    // Locally you still visit it at http://127.0.0.1:4000.
    let listener = tokio::net::TcpListener::bind("0.0.0.0:4000")
        .await
        .unwrap();
    info!("🚀 Server running on http://127.0.0.1:4000 (data persists in todos.db)");
    axum::serve(listener, app).await.unwrap();
}
