/// Use of the AppContext State Managment with Sqlite
/// NOTE: This example requires the SQLite installed on your system.
// Import Our Dependencies
use feather::{App, info, middleware_fn, next, warn};
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use r2d2_sqlite::rusqlite::Result;
use serde_json::json;

fn main() -> Result<()> {
    // Create a new App
    let mut app = App::new();
    let manager = SqliteConnectionManager::file(":memory:");
    let pool: Pool<SqliteConnectionManager> = r2d2::Pool::new(manager).unwrap();

    // Create a person table
    pool.get().unwrap().execute(
        "
    CREATE TABLE person (
        id    INTEGER PRIMARY KEY,
        name  TEXT NOT NULL  
    )",
        [],
    )?;
    app.context().set_state(pool); // Store the connection inside of our context

    // from now on conn is only accesible inside the context
    app.post("/login", login);

    app.get("/user", get_user);

    app.listen("127.0.0.1:5050");
    Ok(())
}
// Post Route for loging in users
#[middleware_fn]
fn login() -> Outcome {
    let data = json!({
        "username": "test_user",
        "password": "test_password"
    });
    info!("Received Json: {data}"); // Log it to see what we got

    let username = match data.get("username") {
        Some(v) => v.as_str().unwrap_or(""),
        None => {
            res.set_status(400).send_text("No username found in the data!");
            return next!();
        }
    };

    let db = ctx.get_state::<Pool<SqliteConnectionManager>>();
    let conn = db.get().unwrap(); // Keep connection alive
    match conn.execute("INSERT INTO person (name) VALUES (?1)", [username]) {
        Ok(rows_changed) => res.set_status(200).send_json(&json!({
            "success": true,
            "rows_changed": rows_changed
        })),
        Err(e) => {
            res.set_status(500).send_json(&json!({"success": false}));
            warn!("{e}")
        }
    };
    next!()
}
// Get Route for listing users
#[middleware_fn]
fn get_user() -> Outcome {
    let db = ctx.get_state::<Pool<SqliteConnectionManager>>();
    let conn = db.get().unwrap(); // Keep connection alive
    let mut stmt = conn.prepare("SELECT name FROM person")?;
    let users = stmt.query_map([], |row| row.get::<_, String>(0))?.filter_map(Result::ok).collect::<Vec<_>>();
    res.set_status(200).send_json(&json!({ "users": users }));
    next!()
}
