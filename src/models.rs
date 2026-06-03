// Data shapes for the app. Everything here is `pub` (public) so OTHER files
// (modules) like handlers.rs are allowed to use them. Without `pub`, a name is
// private to its own file.

use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, sqlx::FromRow)]
pub struct Todo {
    pub id: i64, // fields also need `pub` to be read from another module
    pub title: String,
    pub done: bool,
}

#[derive(Deserialize)]
pub struct CreateTodo {
    pub title: String,
}

#[derive(Deserialize)]
pub struct UpdateTodo {
    pub title: Option<String>,
    pub done: Option<bool>,
}
