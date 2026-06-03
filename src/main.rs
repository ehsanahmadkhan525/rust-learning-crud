use axum::{routing::get, Router}; // `use` = import (like Python's `from axum import ...`)

// #[tokio::main] is a MACRO that sets up the async runtime and lets main be `async`
#[tokio::main]
async fn main() {
    // Build our app: one route, GET "/", handled by `hello`
    let app = Router::new().route("/", get(hello));

    // Bind to a TCP port (like uvicorn binding 127.0.0.1:4000)
    let listener = tokio::net::TcpListener::bind("127.0.0.1:4000")
        .await // .await = wait for this async operation (like Python's await)
        .unwrap(); // unwrap() = "if this failed, crash now" (we'll handle errors properly later)

    println!("🚀 Server running on http://127.0.0.1:4000");

    // Start serving
    axum::serve(listener, app).await.unwrap();
}

// A handler = an async function that returns a response. Like a FastAPI path function.
async fn hello() -> &'static str {
    "Hello from Axum!"
}
