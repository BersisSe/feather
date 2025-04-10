# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).


## [Feather[Runtime Second Update]] - 2025-04-04

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