# Changelog

All notable changes to this project will be documented in this file.
The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/).
---
## [0.4.6] - 2025-06-17

### Notes
This is a pretty minor Quality of life update to Feather.

### Added
**Feather Framework**
- The new `middleware!` macro makes it easier to create middlewares.  
  This macro allows you to define a middleware in a more concise way.


## Changed
**Feather Framework**
- middleware module name has been changed to `middlewares` for better clarity.

---

## [0.4.5] - 2025-06-13

### Notes
This is a Minor Update to Feather.
It includes some bug fixes and quality of life improvements.


### Added
**Feather Framework**
- Dynamic Route Parameters!  
  You can now use dynamic route parameters in your routes.  
  For example:  
  ```rust
  app.get("/user/:id", |req, res| {
      let id = req.param("id").unwrap();
      res.send_text(format!("User ID: {}", id));
  });
  ```
  Express style dynamic route parameters are now supported!

- The `debug`, `info`, `warn`, and `error` macros are re-exported from the `log` crate. also the new `log` feature is added to the framework.  
  This allows you to use the `log` crate's macros directly in your Feather app without needing to import them separately.
  
**Feather Runtime**
- Request now has a `param` method to get dynamic route parameters.


## Changed
**Feather Framework**
- The `App` struct now initializes the logging facade using `simple_logger` crate with the `info` level by default.  
  This means that you can use the `log` macros without needing to set up a logger.


---
## [0.4.4] - 2025-05-24

### Notes
This is a Major Update not to Feather but rather Feather Runtime.  
This update brings more modularity to Feather lets take a look at the changes!  

### Added
**Feather Framework**
- N/A
**Feather Runtime**
- Request has a new Method named `take_stream` it takes the underlying TcpStream Out of the Request use this Method wisely.  

## Fixed
**General**
- The `send_bytes` method on the Response mangling the input spesificly this [issue by timwedde ](https://github.com/BersisSe/feather/issues/12)

## Changed
**Feather Framework**  
- N/A  
**Feather Runtime**  
- Some of the internals has been renamed for clarity 

---


## [0.4.3] - 2025-05-24

### Notes
This update is a minor update to Feather. It includes some bug fixes & some quality of life improvements.

### Added
**Feather Framework**
- N/A
**Feather Runtime**
- New `send_file` method on the `Response` object to send files as a response.
- New `path` method on the `Request` object to get the request path as percent encoded.
## Fixed
**Feather Framework**  
- Fixed a bug where the routes were not percent encoded.
**Feather Runtime**
- N/A

---

## [0.4.1] - 2025-05-11

### Notes
No Notable changes to the framework. Only the Readme file has been symlinked.

---

## [0.4.0] - 2025-05-08

### Notes
This update is a major update. it solves the Error Handling Issue in Feather. With the new Error-Pipeline System.  

Now Every middleware Returns a `Outcome`  
This allows you to handle errors using the `?` operator. That will just pass the error to the next middleware.  
If there is no Error handler in the pipeline it will be passed to the default error handler.  
Default error handler will log the message and return a 500 Internal Server Error with the error message.

### Added
**Feather Framework**
- New Error-Pipeline System to handle errors in middlewares.  
- New `set_handler` method to set a custom error handler for the app.  
- New `next()!` macro for better readability and less boilerplate code.  
- New `Error-pipeline` Example to show how to use the new error handling system.
**Feather Runtime**
- N/A
## Fixed
**Feather Framework**  
- `ServeStatic` middleware's Security problems and use excessive of `unwrap`&`expect` has been fixed.  
- The non-existing route now returns a 404 not found error instead of just freazing the client.  
## Changed
**Feather Framework**  
- Now every middleware returns a `Result<MiddlewareResult, Error>`(We Call it `Outcome` for simplicty) instead of `MiddlewareResult`.  
- File Structure has been changed for better scalability.  
- Middleware example has been rewritten to match the latest changes.
**Feather Runtime**  
- Response's `status` method's name is changed to `set_status` for better clarity.  
---

## [0.3.2] - 2025-05-04

### Notes
No Changes to the framework Only the Readme file has been rewritten.  

---

## [0.3.1] - 2025-05-04

### Notes
This Update includes some bugs fixes in the Runtime and some Quality of life additions  
### Added
**Feather Framework**
- Context now has `get_mut_state` method to access mutable state without Mutexes  
- New Counter Example!  
**Feather Runtime**
- Request Now has `query` method.  
### Fixed
**Feather Framework**  
- N/A  
**Feather Runtime**  
- Now Puts The Correct `connection` HTTP headers  
## Changed
**Feather Framework**  
- N/A
**Feather Runtime**  
- Response's `to_string` method is renamed to `to_raw` for better clarity  
---

## [0.3.0] - 2025-05-01
### Notes

This update is a major update. it adds a Solid State management system on top of Feather called Context API.  
Every App now has a Context from that Context you can add State or Retrieve State this is especially usefull when using databases or file accesses.  
App Context is also reserved for future use for things like event system ,html rendering and more!

### Added

**Feather Framework**

- New Context Api to manage app state without extractors or macros
- New context.rs example to show how context works with a database
  **Feather Runtime**

- N/A

### Removed

**Feather Framework**

- _BREAKING CHANGE_: The old routes now require a `context` parameter.

**Feather Runtime**

- N/A

### Fixed

**Feather Framework**

- N/A

**Feather Runtime**

- Response's status method now returns a referance to the response so you can chain other methods like send_text etc

### Changed

**Feather Framework**

- Changed the file structure for better readablity.
- Middlewares are no longer needs to implement `Clone`.  
  **Feather Runtime**
- N/A

---

## [0.2.1] - 2025-04-24

### Notes

This update is a minor update to the Feather Framework and Feather Runtime. It includes new features and bug fixes. The JWT module allows you to create and verify JWT tokens when the `jwt` feature is enabled. The new `chain` macro allows chaining multiple middlewares for better organization and readability.

### Added

**Feather Framework**

- New JWT module to create and verify JWT tokens.
- New JWT auth helper to protect routes with JWT.
- New `generate_jwt` function.
- New `chain` macro to chain middlewares together.

**Feather Runtime**

- N/A

### Removed

**Feather Framework**

- N/A

**Feather Runtime**

- N/A

### Fixed

**Feather Framework**

- N/A

**Feather Runtime**

- Fixed a bug where the `Response` object's status could not be changed.

### Changed

**Feather Framework**

- Middleware module has been split into multiple files for better organization. This might break some code that uses the old module path.

**Feather Runtime**

- N/A

---

## [0.2.0] - 2025-04-20

### Added

**Feather Framework**

- New JSON methods for the `Request` object to simplify retrieving the JSON body.
- Doc comments for most methods and structs.

**Feather Runtime**

- Internals rewritten for better readability and maintainability.
- Added `TaskPool` to manage concurrent tasks (essentially concurrent requests).
- Added `MessageQueue` to manage requests efficiently.
- Added a new connection management system.
- Added proper error handling and logging.

### Removed

**Feather Framework**

- Removed the `AppConfig` struct.
- Removed the `App` struct's `with_config` method.

**Feather Runtime**

- Removed the `rusty-pool` dependency.

### Fixed

**Feather Framework**

- Improved general performance.

**Feather Runtime**

- Fixed a bug where the server would not send a response if the client shut down the connection.
- Fixed a bug where the server would not shut down properly.
- Improved runtime performance by optimizing internals. See [details](feather-runtime/Performance.md).

---

## [0.1.2] - 2025-04-07

### Added

- New `ServeStatic` middleware to serve static files from a directory.
- New `Response` methods: `send`, `send_json`, and `send_html` for easier response handling.

### Changed

- The `MiddlewareResult` enum is now included in the prelude file for easier access.

---

## [0.1.1] - 2025-04-04

### Added

- Simplified API for `Response` and `Request` objects.
- `App` struct now has a `with_config` method to create an app with a configuration.

### Changed

- Internal code refactored for better readability and maintainability.
- The `App` struct's `new` method no longer takes a configuration.

---

## [0.1.0] - 2025-03-21

### Added

- Initial release of the framework.
- Simple Express-style routing and middlewares.
- Configurable thread pool for handling concurrent requests.

---

## [0.0.1] - 2025-03-15

### Changed

- Migrated to `Feather-Runtime` from `Tiny-HTTP`. See [Feather Runtime README](feather-runtime/README.md) for details.
