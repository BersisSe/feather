/// Use of the AppContext State Managment with Sqlite
/// NOTE: This example requires the SQLite installed on your system.
// Import Our Dependencies
use feather::{App, warn, info, next, middleware_fn};
use rusqlite::{Connection, Result};
use serde_json::json;

fn main() -> Result<()> {
    // Create a new App
    let mut app = App::new();
    // Open a Connection to Sqlite
    let conn = Connection::open_in_memory()?;
    // Create a person table
    conn.execute(
        "
    CREATE TABLE person (
        id    INTEGER PRIMARY KEY,
        name  TEXT NOT NULL  
    )",
        [],
    )?;
    app.context().set_state(conn); // Store the connection inside of our context
    // from now on conn is only accesible inside the context

    app.post("/login", login);

    app.get("/user", get_user);

    app.listen("127.0.0.1:5050");
    Ok(())
}
// Post Route for loging in users
#[middleware_fn]
fn login() -> Outcome {
    let data = match req.json() {
        Ok(json) => json,
        Err(_) => {
            res.set_status(400).send_json(json!({"error": "Invalid JSON"}));
            return next!();
        }
    };
    info!("Received Json: {data}"); // Log it to see what we got
    let db = ctx.get_state::<Connection>().unwrap(); // Get the Connection from the context. Unwrap is safe here because we know we set it before
    let username = match data.get("username") {
        Some(v) => v.to_string(),
        None => {
            res.set_status(400).send_text("No username found in the data!");
            return next!();
        }
    };

    // Now Lets put it inside of our DB
    match db.execute("INSERT INTO person (name) VALUES (?1)", [username]) {
        //If it succeeds we send a successs message with 200 Code
        Ok(rows_changed) => res.set_status(200).send_json(json!
        (
            {
                "success":true,
                "rows_changed":rows_changed
            }
        )),
        //If it fails we send a successs message with 500 Code
        Err(e) => {
            res.set_status(500).send_json(json!
            (
                {
                    "success":false,
                }
            ));
            warn!("{e}")
        }
    };
    next!()
}
// Get Route for listing users
#[middleware_fn]
fn get_user() -> Outcome {
    let db = ctx.get_state::<Connection>().unwrap(); // Again Take our Connection from the context. that is still a single connection

    // We can use the ? operator here because we are inside of a function that returns a Result
    // We prepare a statement to select all names from the person table
    let mut stmt = db.prepare("SELECT name FROM person")?;
    let users = stmt.query_map([], |row| row.get::<_, String>(0))?.filter_map(Result::ok).collect::<Vec<_>>();

    res.set_status(200).send_json(json!({ "users": users }));
    next!()
}
