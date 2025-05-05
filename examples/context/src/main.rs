/// Use of the AppContext State Managment with Sqlite
// Import Our Dependencies
use feather::{App, AppContext, MiddlewareResult, Request, Response};
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
    )
    .unwrap(); // For the sake of this example we are going to unwrap where we can but on production code error should be handled
    app.context().set_state(conn); // Store the conn inside of our context
    // from now on context is only accesible inside the context

    app.post("/login", login);

    app.get("/user", get_user);

    app.listen("127.0.0.1:5050");
    Ok(())
}
// Post Route for loging in users
fn login(req: &mut Request, res: &mut Response, ctx: &mut AppContext) -> MiddlewareResult {
    let data = match req.json() {
        Ok(json) => json,
        Err(_) => {
            res.status(400).send_json(json!({"error": "Invalid JSON"}));
            return MiddlewareResult::Next;
        }
    };
    println!("Received Json: {data}"); // Log it to see what we got
    let db = ctx.get_state::<Connection>().unwrap(); // Get the connection from the context, remember we are not opening new connections here
    let username = data.get("username").unwrap().to_string(); // Get the username
    // Now Lets put it inside of our DB
    match db.execute("INSERT INTO person (name) VALUES (?1)", [username]) {
        //If it succeeds we send a successs message with 200 Code
        Ok(rows_changed) => res.status(200).send_json(json!
        (
            {
                "success":true,
                "rows_changed":rows_changed
            }
        )),
        //If it fails we send a successs message with 500 Code
        Err(e) => {
            res.status(500).send_json(json!
            (
                {
                    "success":false,
                }
            ));
            println!("{e}")
        }
    };
    feather::MiddlewareResult::Next
}
// Get Route for listing users
fn get_user(_req: &mut Request, res: &mut Response, ctx: &mut AppContext) -> MiddlewareResult {
    let db = ctx.get_state::<Connection>().unwrap(); // Again Take our Connection from the context. that is still a single connection
    let mut stmt = db.prepare("SELECT name FROM person").unwrap();
    let users = stmt
        .query_map([], |row| row.get::<_, String>(0))
        .unwrap()
        .filter_map(Result::ok)
        .collect::<Vec<_>>();

    res.status(200).send_json(json!({ "users": users }));
    MiddlewareResult::Next
}
