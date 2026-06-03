# Rust Learning Log — building a CRUD app

Coming from: Python / FastAPI
Goal: learn Rust basics → build a CRUD REST API
Started: 2026-06-03

---

## Roadmap

- [x] 1. Hello Cargo — create project, understand structure, build & run
- [x] 2. Rust basics — variables, structs, ownership, enums
- [x] 3. First web server with Axum
- [x] 4. CRUD endpoints with in-memory storage
- [x] 5. Add a real database (SQLite via sqlx)

### Level 2 — make it production-shaped
- [x] 6. Input validation (reject empty titles → 400)
- [x] 7. Logging with the `tracing` crate
- [x] 8. Tests (`#[test]` / `#[tokio::test]`)
- [x] 9. Split into modules (models.rs, handlers, etc.)
- [x] 10. Compile-time-checked SQL (`sqlx::query!` macro)
- [x] 11. Dockerize & run anywhere

---

## Concept map (Python → Rust)

| Concept | Python / FastAPI | Rust |
|---|---|---|
| Package manager | pip / poetry | cargo |
| Deps file | requirements.txt / pyproject.toml | Cargo.toml |
| Web framework | FastAPI | Axum |
| Async runtime | asyncio / uvicorn | Tokio |
| Validation/models | Pydantic | serde + structs |
| DB layer | SQLAlchemy | sqlx |

---

## Plain-English explanations (no tech background needed)

Everyday analogies for every concept in this project. Read this when a term feels confusing.

### The tools
- **The compiler (rustc)** = a very strict proofreader. Before your program is allowed
  to run, it reads EVERYTHING and refuses to continue until every mistake is fixed.
  (Python is like sending a text message instantly — mistakes only show up later, mid-use.)
  This is why Rust feels picky at first but rarely crashes once it runs.
- **Cargo** = your project's personal assistant. It fetches supplies (code libraries),
  assembles the project, and runs it for you. `cargo run` = "build it and start it."
- **A library / crate** = a pre-made box of tools someone else built so you don't have to.
  `cargo add axum` = "go get me the 'axum' toolbox."

### Storing values
- **Variable** = a labeled box that holds something. `let name = "ehsan"` puts text in a box
  labeled `name`.
- **Immutable vs mutable** = a box sealed with tape vs a box you can reopen.
  `let` tapes it shut (can't change it). `let mut` lets you swap the contents.
  Rust tapes boxes shut BY DEFAULT so you never change something by accident.
- **Type** = WHAT KIND of thing is in the box: a number, some text, or true/false.
  Rust won't let you put text in a box meant for numbers — it catches the mix-up for you.

### Ownership & borrowing (Rust's signature idea)
Think of a house with exactly ONE physical key:
- **Ownership** = whoever holds the key owns the house. Only one owner at a time.
- **Move** = you hand your only key to a friend. Now THEY have it and you don't —
  you can't use the house anymore. (This is why `let s2 = s1` makes `s1` unusable.)
- **Clone** = you make a photocopy / a second key. Now you BOTH have your own — but it
  cost you the effort of copying.
- **Borrow (`&`)** = lending a book. Your friend reads it and gives it back; you still own it.
- **Mutable borrow (`&mut`)** = lending your car AND letting them adjust the seat.
  Rule: only one person can drive at a time (one `&mut`, OR many read-only `&` — never both).
  This is how Rust stops two people scribbling on the same page at once.

### Shaping data
- **Struct** = a blank form with labeled fields: [ Title: ___ ] [ Done?: ___ ].
  Fill one in and you've got one record (one "Todo").
- **Vec (list)** = a shopping list: you can add items and cross them off.
- **Option (Some / None)** = a lunchbox that might hold a sandwich (`Some`) or be empty (`None`).
  Rust forces you to LOOK before you reach in, so you never grab from an empty box.
- **Result (Ok / Err)** = mailing a letter: it either arrives (`Ok`) or bounces back
  "return to sender" (`Err`). You must deal with both outcomes.
- **match** = a sorting hat / flowchart: "if it's THIS, do that; if it's THAT, do this."
  Rust makes you cover EVERY possibility, so nothing slips through.
- **Function** = a coffee machine: put ingredients in, get a drink out. Same inputs, same output.

### The web app pieces
- **Server** = a vending machine that's always switched on, waiting for someone to press a button.
- **Route** = one button on the machine (e.g. the "/todos" button).
- **curl / browser** = your finger pressing the button to see what comes out.
- **async / await** = a restaurant: you place an order and the kitchen cooks while the waiter
  serves other tables. `await` = "wait here until my dish is ready," without freezing everyone else.
- **JSON** = a standard fill-in-the-blanks form that all computers agree on, so they can
  swap information without confusion.
- **serde** = a translator that converts between YOUR form (a struct) and the standard form (JSON).

### Sharing & safety (the Arc/Mutex stuff)
- **Many requests at once** = a busy café with lots of customers arriving together.
- **Mutex** = the single bathroom key at that café: only one person inside at a time,
  so two people never collide. A request "takes the key" (`.lock()`), does its thing, returns it.
- **Arc** = one shared notebook everyone can reach (like a shared Google Doc) — many people,
  one copy, nobody loses track of it.

### Saving data
- **In-memory storage (the early version)** = a whiteboard. Great while you're working,
  but wiped completely the moment you turn it off (restart).
- **Database / SQLite (the final version)** = a filing cabinet. Write something down and it's
  STILL THERE tomorrow, even after the computer is switched off. `todos.db` is that cabinet (one file).
- **sqlx** = the clerk who files and retrieves folders from the cabinet for you.

### Status codes (the server's replies)
Like parcel-delivery statuses:
- **200 OK** = here's what you asked for.   **201 Created** = your new item was added.
- **204 No Content** = done, nothing to send back (used after deleting).
- **404 Not Found** = "no such thing here."  **500 Server Error** = "something broke on our end."

### Handy shortcuts you'll see in the code
- **The `?` mark** = "if anything went wrong here, stop and report it; otherwise carry on."
  A one-character way to handle problems neatly.
- **A closure `|t| ...`** = a tiny throwaway instruction you hand to a helper, e.g.
  "for each item `t`, check if its id matches." Like a sticky note with a quick rule on it.

---

## Code ↔ everyday meaning (real lines from THIS project)

Match the exact code you typed to what it means in plain words.

### Basics
| Code line | What it means in plain words |
|---|---|
| `let name = "ehsan";` | Put the text "ehsan" in a sealed box labelled `name` (can't change it). |
| `let mut age = 25;` | Put 25 in a box labelled `age` that I'm allowed to reopen and change. |
| `age = age + 1;` | Open the `age` box, add 1, put it back (allowed because of `mut`). |
| `let height: f64 = 1.75;` | Box `height` holds a decimal number; I'm spelling out the kind on purpose. |
| `println!("{name} is {age}");` | Print a line, dropping the box contents into the blanks. |
| `fn add(a: i32, b: i32) -> i32 {` | A machine called `add` that takes two whole numbers and gives one back. |
| `a + b` (last line, no `;`) | The answer this machine hands back is a plus b. |

### Ownership & borrowing
| Code line | What it means in plain words |
|---|---|
| `let s1 = String::from("hello");` | `s1` owns the word "hello" (holds the only key). |
| `let s2 = s1;` | Hand the key to `s2`. Now `s1` is empty — using it is an error. |
| `let s2 = s1.clone();` | Photocopy it. `s1` and `s2` each own their own copy. |
| `fn string_length(s: &String)` | This machine BORROWS the word (reads it, gives it back). |
| `fn make_uppercase(s: &mut String)` | Borrows it AND is allowed to change it (one borrower only). |

### Shaping data (structs, Option, match)
| Code line | What it means in plain words |
|---|---|
| `struct Todo { id, title, done }` | A blank form with three labelled fields. |
| `Todo { id: 1, title: ..., done: false }` | Fill one form in to create one todo. |
| `impl Todo { fn new(...) }` | Where I keep Todo's helpers, like a "make a new Todo" button. |
| `fn mark_done(&mut self)` | A button on a todo that flips its `done` to true. |
| `vec![a, b]` | A list holding a and b (a shopping list). |
| `Option<String>` | "Maybe a title, maybe nothing." A lunchbox that might be empty. |
| `Some(title) => ...` | The lunchbox HAD something — here's how to use it. |
| `None => ...` | The lunchbox was empty — here's what to do instead. |
| `match find_title(...) { ... }` | A flowchart: handle the "found it" and "didn't" cases, both required. |

### The web server
| Code line | What it means in plain words |
|---|---|
| `use axum::{...};` | "Bring the axum tools into this file." |
| `#[tokio::main]` | "Switch on the engine so this program can juggle many tasks at once." |
| `async fn main()` | The starting point, built to handle many requests without freezing. |
| `Router::new()` | Start building the vending machine (no buttons yet). |
| `.route("/todos", get(list_todos))` | Add a button: visiting /todos runs the `list_todos` job. |
| `TcpListener::bind("127.0.0.1:4000")` | Plug the machine into door number 4000 on this computer. |
| `axum::serve(listener, app).await` | Switch the machine on and keep it running, waiting for visitors. |
| `async fn hello() -> &'static str` | A job that hands back a fixed bit of text. |

### JSON + CRUD handlers
| Code line | What it means in plain words |
|---|---|
| `#[derive(Serialize, Deserialize)]` | "Auto-build a translator between this form and standard JSON." |
| `struct CreateTodo { title }` | The shorter form a visitor fills to ADD a todo (just a title). |
| `Json(payload): Json<CreateTodo>` | Read the visitor's JSON and unpack it into a filled form. |
| `State(db): State<Db>` | Hand this job access to the shared storage. |
| `-> (StatusCode::CREATED, Json(todo))` | Reply "201 Created" and send the new todo back as JSON. |
| `Path(id): Path<i64>` | Grab the number from the web address, e.g. the 7 in /todos/7. |
| `-> Result<Json<Todo>, StatusCode>` | This job gives back EITHER a todo OR an error code (like 404). |
| `Err(StatusCode::NOT_FOUND)` | Reply "404 Not Found." |
| `if let Some(title) = payload.title` | "Only if the visitor actually sent a new title, use it." |
| `.retain(\|t\| t.id != id)` | Keep every todo EXCEPT the one with this id (i.e. delete that one). |

### Database (sqlx + SQLite)
| Code line | What it means in plain words |
|---|---|
| `SqlitePool::connect("sqlite:todos.db?mode=rwc")` | Open the filing cabinet file (create it if it's not there yet). |
| `CREATE TABLE IF NOT EXISTS todos (...)` | Set up the "todos" drawer once, if it doesn't already exist. |
| `#[derive(sqlx::FromRow)]` | "Teach the clerk how to turn a filed row back into a Todo form." |
| `sqlx::query_as::<_, Todo>("SELECT ...")` | Ask the cabinet for rows and hand them back as Todo forms. |
| `.bind(payload.title)` | Safely slot the visitor's title into the `?` blank in the query. |
| `.fetch_all(&pool).await` | Wait, then bring back ALL matching rows. |
| `.fetch_optional(&pool).await` | Bring back ONE row, or nothing if there isn't one. |
| `INSERT ... RETURNING id, title, done` | File the new row AND hand it straight back (with its new id). |
| `.map_err(\|_\| StatusCode::INTERNAL_SERVER_ERROR)?` | "If the cabinet jammed, stop and reply 500." |
| `existing.ok_or(StatusCode::NOT_FOUND)?` | "If there was no such row, stop and reply 404." |
| `result.rows_affected() > 0` | Did the delete actually remove a row? (If not → 404.) |

---

## Challenge 6 — Input validation  [DONE]

Manual check inside create_todo and update_todo:
```rust
if payload.title.trim().is_empty() {
    return Err(StatusCode::BAD_REQUEST); // 400
}
```
- `.trim()` strips spaces so "   " counts as empty; `.is_empty()` checks blank.
- In update, wrap in `if let Some(title) = &payload.title { ... }` — only validate
  the title IF the client sent one (it's optional on update). `&` borrows it (peek, don't take).
- `return Err(StatusCode::BAD_REQUEST)` = FastAPI's `raise HTTPException(400)`.
- The `validator` crate automates many rules (`#[validate(length(min=1))]`) — Pydantic-like;
  manual is clearer for a single rule.

---

## Challenge 7 — Logging (tracing)  [DONE]

Deps: tracing, tracing-subscriber (env-filter), tower-http (trace).
- `tracing_subscriber::fmt().with_env_filter("info,tower_http=debug").init();` at start of main.
- `.layer(TraceLayer::new_for_http())` on the router = middleware that logs every request.
- `info!("msg")` for a line; `info!(id = todo.id, title = %todo.title, "created todo")` for
  structured fields (% = format as display). Also warn!/error!/debug!.
- `RUST_LOG=debug cargo run` overrides the filter without recompiling.
- Middleware = code that runs around every request (like FastAPI middleware/Depends).

## Challenge 8 — Tests  [DONE]

Built-in — no pytest. Run with `cargo test`.
- Tests live in `#[cfg(test)] mod tests { use super::*; ... }` (cfg(test) = compiled only for tests).
- `#[tokio::test]` for async tests; `#[test]` for sync.
- Call handlers directly, building extractors by hand: State(pool), Json(CreateTodo{..}), Path(id).
- `assert_eq!`, `assert!`, `assert_ne!`. `.unwrap()` = expect success; `.unwrap_err()` = expect failure.
- Each test makes its OWN `init_db("sqlite::memory:")` → isolated, parallel-safe, never touches todos.db.
- Refactored DB setup into `async fn init_db(url)` so main + tests share it.
  - Uses `SqlitePoolOptions::new().max_connections(1)` so in-memory DB stays alive for the pool.

---

## Challenge 9 — Split into modules  [DONE]

Project layout now:
```
src/
├── main.rs      ← startup + routing only
├── models.rs    ← Todo, CreateTodo, UpdateTodo
├── db.rs        ← init_db
└── handlers.rs  ← 5 handlers + tests
```
- `mod db;` in main.rs = "include src/db.rs as a module" (like Python import).
- Rust is PRIVATE BY DEFAULT: add `pub` to make items visible to other modules.
  - Structs AND their fields each need `pub` to be used from another file.
- `crate::` = from this project's root. `use crate::models::Todo;` = `from models import Todo`.
- Behaviour unchanged — pure reorganization (that's a good refactor).

Two compiler lessons:
- `#[derive(Debug)]` = make a type printable for developers (Rust's __repr__).
  Needed because test asserts/`unwrap_err` must print the value on failure.
- `let _ = expr;` = "deliberately ignore this return value" (silences must-use warnings,
  e.g. ignoring a Result/Json you don't need in a test).

---

## Challenge 10 — Compile-time-checked SQL  [DONE]

Switched runtime queries to compile-time-checked MACROS:
- `sqlx::query_as::<_, Todo>("SELECT ...").bind(id)` → `sqlx::query_as!(Todo, "SELECT ...", id)`
- `sqlx::query("DELETE ...").bind(id)` → `sqlx::query!("DELETE ...", id)`
- Bind params move from `.bind(x)` to macro args after the SQL string.
- The `!` macro CONNECTS to the DB at compile time and verifies tables/columns/types
  against the `Todo` struct. A typo'd column = COMPILE error, e.g.:
  `error: no such column: titel  --> src/handlers.rs:17`. Bug never reaches runtime.

Setup needed:
- `.env` file with `DATABASE_URL=sqlite:todos.db` (the macro reads it at build time).
- todos.db must exist WITH the schema at compile time.
- `.env` is gitignored (machine-specific).

CAVEAT / next: with `.env` and `todos.db` gitignored, a FRESH CLONE or Docker build
can't compile (no DB to check against). Fix = sqlx OFFLINE MODE:
  `cargo install sqlx-cli --no-default-features --features sqlite`
  `cargo sqlx prepare`   # generates a `.sqlx/` cache you COMMIT
Then builds work with no live DB (set SQLX_OFFLINE=true). Needed before Docker.

---

## Challenge 11 — Dockerize  [DONE]

Files added: `Dockerfile`, `.dockerignore`. Final image `rust-crud` = 123 MB.

- Changed bind from `127.0.0.1:4000` to `0.0.0.0:4000` so Docker port-mapping can reach it.
- Multi-stage build:
  - Stage 1 `FROM rust:1` = big image with compiler → `cargo build --release` (with SQLX_OFFLINE=true).
  - Stage 2 `FROM debian:bookworm-slim` = tiny image; COPY only the binary across.
  - Result: ~123 MB final image instead of ~1.5 GB (ships the program, not the toolchain).
- This is why we set up sqlx offline mode first — the build has no live DB.

Commands:
```bash
docker build -t rust-crud .
docker run -p 4000:4000 rust-crud                 # run
docker run -p 4000:4000 -v "$(pwd)/data:/app" rust-crud   # run + persist todos.db to ./data
```
Dockerfile keywords: FROM (base image), WORKDIR, COPY, RUN (build-time cmd),
EXPOSE (doc the port), CMD (start command). `.dockerignore` = like .gitignore for builds.

Data note: todos.db lives inside the container unless you mount a volume (-v).
Deploy = push this image to any cloud (Fly.io/Railway/VPS) and `docker run` it.

---

## Setup

### Installing Rust (if not already installed)
- Linux/macOS: `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh` (choose option 1)
  - Then `source "$HOME/.cargo/env"` or restart terminal.
- Windows: download `rustup-init.exe` from https://rustup.rs
- Verify: `rustc --version` and `cargo --version`

### My machine (already installed)
- rustc 1.95.0
- cargo 1.95.0
- OS: Ubuntu 24.04

The toolchain:
- `rustc` = compiler (ahead-of-time, unlike the Python interpreter)
- `cargo` = build tool + package manager (pip + venv + poetry + task runner)
- `rustup` = version manager (like pyenv)

---

## Step 1 — Hello Cargo  [DONE]

Used `cargo init` in the project root → project name = `rust-learning-crud`, edition 2024.

Generated files explained:
- `Cargo.toml` = manifest (like pyproject.toml). [package] info + [dependencies].
- `src/main.rs` = entry point. `fn main()` runs automatically (no __main__ needed).
- `println!("...")` — the `!` means MACRO (checks format string at compile time).
- Statements end with `;`. Blocks use `{ }`, not indentation.
- `Cargo.lock` = pinned versions (like poetry.lock). Auto-managed.
- `target/` = compiled output (binary at target/debug/rust-learning-crud).
- `.git/` + `.gitignore` created automatically.

`fn` = function (Python's `def`). `main` is the special start function.

### Variables & types
- `let x = ...` immutable by default; `let mut x = ...` to allow reassignment.
- Types are static & inferred but locked in (i32, f64, bool, String, ...).
- `println!("{var}")` = f-string style; `{}` fills from args after.
- Last expression with NO semicolon = the return value of a function.

### Errors vs Warnings
- ERROR (e.g. E0384) = illegal, build FAILS, nothing runs.
- WARNING (e.g. unused_assignments) = legal but suspicious, still runs.
- `rustc --explain E0384` gives a full explanation of any error code.

### Ownership (the big new idea — no garbage collector)
3 rules:
1. Every value has exactly ONE owner (a variable).
2. When the owner goes out of scope, the value is freed automatically.
3. Assigning/passing can MOVE ownership; the old variable becomes invalid.

- `let s2 = s1;` on a String MOVES ownership → using s1 after = error E0382.
- Small types (i32, f64, bool) COPY instead of move (cheap). String/Vec/struct MOVE.
- Fixes for "moved value" errors: `.clone()` (copy the data) or borrowing with `&`.

### Borrowing
- `&x` = read-only borrow (lend, don't give away). `&mut x` = mutable borrow.
- Pass `&thing` to functions to read without taking ownership.
- The rule: many `&` readers OR exactly one `&mut` writer — never both at once.

### Structs, impl, enums (the Pydantic replacement)
- `struct Todo { id: u32, title: String, done: bool }` = data shape (like Pydantic model).
- `impl Todo { ... }` block holds methods/constructors.
  - `Todo::new(...)` = associated fn, convention for a constructor (no built-in one).
  - `fn method(&self)` reads; `fn method(&mut self)` modifies (needs `let mut`).
  - Field shorthand: `Todo { id, title, done: false }` when var name == field name.
- `Option<T>` = maybe-missing value: `Some(v)` or `None` (replaces Python None).
- `Result<T, E>` = maybe-failed: `Ok(v)` or `Err(e)` (replaces try/except).
- `match` forces you to handle ALL variants → no forgotten None/error cases.
- `Vec<T>` = growable list (like Python list).

Python → Rust: Pydantic model→struct, classmethod→Type::new, None→Option, try/except→Result, list→Vec.

### Items vs Statements (common beginner error!)
- Top level of a .rs file = only ITEMS allowed: fn, struct, enum, impl, use, const.
- Executable STATEMENTS (match, let, println!, for, if-calls) must be INSIDE a function.
- Error "expected item, found keyword `match`" = you put a statement at top level.
- Unlike Python, you CAN'T run loose code at file top level — it all goes in main (or another fn).
- `vec![a, b, c]` macro builds a Vec quickly.

### GOTCHA: don't pile snippets — REPLACE the file
- A program has exactly ONE `main`. Each example is a complete standalone file.
- When replacing src/main.rs: select all (Ctrl+A), delete, THEN paste. Don't append.


Commands:
```bash
cd /home/ehsan/Desktop/rust-learning-crud
cargo new hello_rust
cd hello_rust
cargo run
```

- `cargo new hello_rust` — scaffolds project (Cargo.toml + src/main.rs + git init)
- `cargo run` — compiles AND runs

Expected output: `Hello, world!`

Notes:
- `cargo new <name>` creates a NEW subfolder and scaffolds inside it.
- `cargo new` alone errors — it requires a path/name argument.
- `cargo new .` errors because the current folder already exists; cargo new
  always wants to create a fresh directory.
- `cargo init` scaffolds inside the CURRENT folder (use after mkdir + cd).
  - Rule: `cargo new myapp` = make myapp/ subfolder. `cargo init` = use current folder.

---

## Git workflow

Repo: https://github.com/ehsanahmadkhan525/rust-learning-crud

First push (done once):
```bash
git add STEPS.md                 # stage files (cargo staged the rest)
git commit -m "message"          # snapshot locally
gh repo create rust-learning-crud --public --source=. --remote=origin --push
```
- `gh repo create` made the GitHub repo, wired up `origin`, and pushed in one go.

Everyday workflow (after the first push):
```bash
git add -A                       # stage all changes
git commit -m "what I did"       # snapshot
git push                         # send to GitHub
```
Manual alternative to gh: `git remote add origin <url>` then `git push -u origin main`.

---

## Commands cheat-sheet (so far)

| Command | What it does |
|---|---|
| `cargo new <name>` | Create a new project in a NEW subfolder |
| `cargo init` | Scaffold a project in the CURRENT folder |
| `cargo run` | Compile + run |
| `cargo build` | Compile only (debug build in target/debug) |
| `cargo check` | Type-check fast without producing a binary |
| `rustc --version` | Show compiler version |
| `cargo add <crate>` | Add a dependency (fetches latest version) |
| `cargo add tokio --features full` | Add a crate with specific features enabled |

---

## Step 3 — Axum web server  [DONE]

Deps added: axum 0.8.9, tokio 1.52.3 (features = full).
Server runs on http://127.0.0.1:4000 (customized port from 3000).

Key pieces:
- `use axum::{...}` = imports (like `from axum import ...`).
- `#[tokio::main]` macro = sets up async runtime so `main` can be `async`.
- `Router::new().route("/", get(hello))` = define routes (like @app.get("/")).
- `TcpListener::bind(addr).await` = bind a port (like uvicorn).
- `axum::serve(listener, app).await` = start serving.
- Handler = `async fn hello() -> &'static str` returns the response.
- `.await` waits for async ops; `.unwrap()` crashes on error (temporary).
- The server BLOCKS the terminal while running — use a 2nd terminal to curl it. Ctrl+C stops it.

FastAPI → Axum: FastAPI()→Router::new(), @app.get→.route(...,get(fn)), async def→async fn, uvicorn→axum::serve, await→.await.

---

## Step 4 — CRUD with JSON (in-memory)  [IN PROGRESS]

Dep added: `cargo add serde --features derive`.

Concepts:
- `#[derive(Serialize, Deserialize)]` = Pydantic replacement. Serialize=struct→JSON (responses),
  Deserialize=JSON→struct (requests). Add `Clone` to allow copying.
- Separate "create" struct (CreateTodo) with only client-supplied fields (no id).
- Shared state = `Arc<Mutex<Vec<Todo>>>`:
  - Arc = share one DB across many requests (ref-counted pointer).
  - Mutex = lock so only one request writes at a time (prevents data races).
  - `db.lock().unwrap()` to access; `.with_state(db)` registers it on the router.
- Extractors (like FastAPI dependency injection): `State(db)`, `Json(payload)`, `Path(id)`.
- Return `Json<T>` to serialize out; `(StatusCode::CREATED, Json(t))` to set status + body.
- Combine methods on one path: `.route("/todos", get(list).post(create))`.

Full CRUD routes:
- GET    /todos        -> list_todos   (200, all)
- POST   /todos        -> create_todo  (201, created todo)
- GET    /todos/{id}   -> get_todo      (200 or 404)
- PUT    /todos/{id}   -> update_todo   (200 or 404; partial via Option fields)
- DELETE /todos/{id}   -> delete_todo   (204 or 404)

More concepts (part 2):
- `Path(id): Path<u32>` extracts & type-checks a URL segment (like FastAPI todo_id: int).
- Return `Result<Json<Todo>, StatusCode>`: Ok(...)=success, Err(StatusCode::NOT_FOUND)=404.
  No exceptions — you RETURN the error (vs FastAPI raise HTTPException).
- `if let Some(x) = opt { ... }` runs only when the Option has a value (partial update).
- Iterator methods: `.find(|t| ...)`, `.iter_mut()` (edit in place), `.retain(|t| ...)` (delete others).
- `|t| ...` = closure = inline anonymous fn (like Python lambda).
- Combine methods: `.route("/todos/{id}", get(g).put(u).delete(d))`.
- In-memory data resets on restart (fixed in Step 5 with SQLite).

---

## Step 5 — SQLite + sqlx (persistence)  [DONE]

Dep: `cargo add sqlx --features runtime-tokio,sqlite`.
DB file `todos.db` (gitignored). Routes/handlers unchanged — only storage swapped.

Setup:
- `SqlitePool::connect("sqlite:todos.db?mode=rwc")` — mode=rwc creates file if missing.
- Run `CREATE TABLE IF NOT EXISTS todos (...)` once on startup.
- State is now the pool: `.with_state(pool)`, handlers take `State(pool): State<SqlitePool>`.

Concepts (Python/SQLAlchemy → sqlx):
- `#[derive(sqlx::FromRow)]` — map a DB row to a struct (ORM mapping).
- `sqlx::query_as::<_, Todo>(sql)` then `.bind(v)` for each `?` (safe, no SQL injection).
- `.fetch_all` (Vec), `.fetch_optional` (Option), `.fetch_one` (one row).
- `INSERT ... RETURNING id, title, done` to get the created row back with its id.
- id is i64 (SQLite ints are 64-bit).

Error handling:
- `?` operator = if Err, return early; else unwrap Ok and continue (replaces try/except).
- `.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?` = map any DB error to a 500.
- `.ok_or(StatusCode::NOT_FOUND)?` = Option→Result: Some(x) continue, None return 404.

Persistence test: create a todo, Ctrl+C, `cargo run` again, GET /todos → still there.

---

## What I learned / where to go next
Built a full CRUD REST API in Rust: Axum + Tokio + serde + sqlx/SQLite.
Ideas to extend: input validation, logging (tracing crate), tests, PATCH vs PUT,
auth, Docker, deploy. Compile-time checked queries via `sqlx::query!` macro (needs DATABASE_URL).

Test:
```bash
curl http://127.0.0.1:4000/todos
curl -X POST http://127.0.0.1:4000/todos -H "Content-Type: application/json" -d '{"title":"x"}'
```

FastAPI → Axum: BaseModel→derive struct, body param→Json(payload), Depends→State, return dict→Json.
