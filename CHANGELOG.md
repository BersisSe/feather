# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).


## Feather[Runtime Third Update] - 2025-04-20


### Added
**Feather Framework**
- New Json methods for the `Request` object to make it easier to get the json body
- Doc comments for almost all methods and structs
**Feather Runtime**
- The internals has been rewitten to be more readable and understandable
- Added TaskPool to manage concurrent tasks(Essentially Concurrent Requests)
- Added a new MessagesQueue to manage the requests efficiently
- Added a new Connection Managment system to manage the connections efficiently
- Added Proper error handling and logging


### Removed
**Feather Framework**
- The `AppConfig` struct has been removed.
- The `App` struct's `with_config` method has been removed.

**Feather Runtime**
- Rusty-pool Dependency has been removed


### Fixed
**Feather Framework**
- Improved the General Performance of the framework(More on that topic in the runtime section).

**Feather Runtime**
- Fixed the bug if the client shutsdown the connection server will not send a response back
- Fixed the bug server would not shutdown properly
- Improved the performance of the Runtime by changing the internals to be more efficient [Details](feather-runtime/Performance.md)








## Feather[Runtime Second Update] - 2025-04-07

### Added
- A new `ServeStatic` middleware to serve static files from a directory
- New `Response` methods `Send`, `SendJson`, and `SendHtml` so you dont have to dereference the response object to change the value

### Changed
- The `MiddlewareResult` enum now is inculuded in the prelude file for easier access


## [Feather[Runtime First Update]] - 2025-04-04

### Added
- Easier to understand api for Response and Request objects
- App Struct now has a `with_config` method to create a app with a config 

### Changed
- Some of the internal code has been changed to be more readable and understandable
- The App struct's `new` method does not take a config anymore.

### Deprecated
- N/A

### Removed
- N/A

### Fixed
- N/A

### Security
- N/A

## [Feather[Runtime Change]]

### Changed
- Moved To `Feather-Runtime` from `Tiny-http` See Feather Runtime's Readme for more [Feather-Runtime](feather-runtime/README.md)


## [0.1.1] - 2025-03-21
### Added
- First Update of the framework
- Simple Express Style Routing and Middlewares
- Configurable thread pool for handling concurrent requests.