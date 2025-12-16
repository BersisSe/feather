# Server Configuration in Feather
## IMPORTANT!! : Feather's version 0.6.x is not benchmarked yet. The details in this document are **NOT TESTED!** they are estimates.

Fine-tune your Feather server's performance and behavior with configuration options.

## ServerConfig Structure

Feather's `ServerConfig` struct controls server-level behavior:

```rust
pub struct ServerConfig {
    pub max_body_size: usize,       // Maximum request body size in bytes
    pub read_timeout_secs: u64,     // Read timeout in seconds
    pub workers: usize,              // Number of worker threads
    pub stack_size: usize,           // Stack size per coroutine in bytes
}
```

## Creating a Custom Configuration

### With Default Config

Start with defaults and create a new app:

```rust
use feather::App;

fn main() {
    let mut app = App::new();  // Uses default configuration
    app.listen("127.0.0.1:5050");
}
```

### With Custom Config

Create a custom configuration:

```rust
use feather::{App, ServerConfig};

fn main() {
    let config = ServerConfig {
        max_body_size: 10 * 1024 * 1024,  // 10MB
        read_timeout_secs: 60,             // 60 seconds
        workers: 4,                        // 4 worker threads
        stack_size: 128 * 1024,            // 128KB
    };
    
    let mut app = App::with_config(config);
    app.listen("127.0.0.1:5050");
}
```

### Using Convenience Methods

Set configuration after app creation:

```rust
use feather::App;

fn main() {
    let mut app = App::new();
    
    app.max_body(10 * 1024 * 1024)  // 10MB
       .read_timeout(60)            // 60 seconds
       .workers(4)                  // 4 threads
       .stack_size(128 * 1024);     // 128KB
    
    app.listen("127.0.0.1:5050");
}
```

## Configuration Options

### max_body_size

Maximum request body size in bytes.

**Default**: 8192 bytes (8KB)

**Use cases**:
- File uploads: Set higher for large file uploads
- API endpoints: Can be lower for lightweight APIs
- Streaming: Consider your typical request sizes

**Example**:
```rust
// For file uploads
app.max_body(100 * 1024 * 1024);  // 100MB

// For simple APIs
app.max_body(1024);  // 1KB

// For typical REST APIs
app.max_body(5 * 1024 * 1024);  // 5MB
```

**Warning**: Large values increase memory usage per request.

### read_timeout_secs

Read timeout for client connections in seconds.

**Default**: 30 seconds

**Use cases**:
- Slow clients: Increase for slow network
- Long requests: Increase for long-running operations
- DDoS protection: Decrease for quick rejection

**Example**:
```rust
// For slow clients
app.read_timeout(120);  // 2 minutes

// For fast APIs
app.read_timeout(10);   // 10 seconds

// For streaming responses
app.read_timeout(300);  // 5 minutes
```

### workers

Number of worker threads for handling connections.

**Default**: Number of CPU cores

**Use cases**:
- High concurrency: Match or exceed CPU core count
- Shared system: Set lower than core count
- Benchmarking: Experiment with different values

**Example**:
```rust
use num_cpus;

let cpu_count = num_cpus::get();

let config = ServerConfig {
    workers: cpu_count * 2,  // 2x CPU cores
    ..Default::default()
};

let mut app = App::with_config(config);
```

**Note**: More workers use more memory and system resources.

### stack_size

Stack size per coroutine in bytes.

**Default**: 65536 bytes (64KB)

**Important**: Stack sizes below 32KB can cause stack overflow issues with the logger.

**Use cases**:
- Complex functions: Increase for deeply recursive code
- Memory constrained: Decrease if available memory is low
- Logging heavy: Keep at least 32KB

**Example**:
```rust
// For simple operations
app.stack_size(32 * 1024);   // 32KB (minimum safe)

// Standard usage
app.stack_size(64 * 1024);   // 64KB (default)

// Complex operations
app.stack_size(256 * 1024);  // 256KB

// Very memory intensive
app.stack_size(512 * 1024);  // 512KB
```

## Performance Tuning

### For High Traffic

Optimize for handling many concurrent requests:

```rust
use feather::{App, ServerConfig};
use num_cpus;

let config = ServerConfig {
    max_body_size: 1 * 1024 * 1024,        // 1MB
    read_timeout_secs: 30,                  // 30 seconds
    workers: num_cpus::get() * 2,          // 2x CPU cores
    stack_size: 128 * 1024,                 // 128KB
};

let mut app = App::with_config(config);
```

### For File Uploads

Optimize for large file uploads:

```rust
let config = ServerConfig {
    max_body_size: 500 * 1024 * 1024,      // 500MB
    read_timeout_secs: 300,                 // 5 minutes
    workers: num_cpus::get(),              // CPU cores
    stack_size: 256 * 1024,                 // 256KB
};

let mut app = App::with_config(config);
```

### For Low-Resource Environments

Optimize for limited memory/CPU:

```rust
let config = ServerConfig {
    max_body_size: 256 * 1024,              // 256KB
    read_timeout_secs: 15,                  // 15 seconds
    workers: 2,                             // 2 threads
    stack_size: 32 * 1024,                  // 32KB minimum
};

let mut app = App::with_config(config);
```

### For Real-time APIs

Optimize for fast response times:

```rust
let config = ServerConfig {
    max_body_size: 64 * 1024,               // 64KB
    read_timeout_secs: 5,                   // 5 seconds
    workers: num_cpus::get() * 2,          // 2x cores
    stack_size: 96 * 1024,                  // 96KB
};

let mut app = App::with_config(config);
```

## Monitoring Configuration Impact

### Memory Usage

Stack size × worker threads × concurrent requests = memory usage

```text
Example:
128KB stack × 8 workers × 100 requests = ~102MB
```

### CPU Utilization

More workers improve throughput but use more CPU:

```rust
// Conservative - use less CPU
app.workers(num_cpus::get());

// Aggressive - use more CPU for more throughput
app.workers(num_cpus::get() * 2);
```

## Default Configuration

The default `ServerConfig`:

```rust
impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            max_body_size: 8192,                    // 8KB
            read_timeout_secs: 30,                  // 30 seconds
            workers: num_cpus::get(),              // CPU cores
            stack_size: 65536,                      // 64KB
        }
    }
}
```

## Listening on Different Addresses

The `listen()` method accepts any address format that implements `ToSocketAddrs`:

```rust
use feather::App;

let mut app = App::new();

// Listen on localhost port 5050
app.listen("127.0.0.1:5050");

// Listen on all interfaces
app.listen("0.0.0.0:8080");

// Listen on IPv6
app.listen("[::1]:5050");

// Listen on all IPv6 interfaces
app.listen("[::]:8080");

// Listen on hostname
app.listen("localhost:5050");

// Multiple addresses with string
app.listen("127.0.0.1:5050");
```

## Example: Production Server

Complete example for a production server:

```rust
use feather::{App, ServerConfig};
use std::env;
use num_cpus;

fn main() {
    // Get configuration from environment
    let port = env::var("PORT")
        .unwrap_or_default()
        .parse::<u16>()
        .unwrap_or(5050);
    
    let host = env::var("HOST")
        .unwrap_or_default()
        .unwrap_or_else(|_| "0.0.0.0".to_string());
    
    // Create production config
    let config = ServerConfig {
        max_body_size: 10 * 1024 * 1024,    // 10MB
        read_timeout_secs: 60,               // 60 seconds
        workers: num_cpus::get() * 2,       // 2x cores
        stack_size: 256 * 1024,              // 256KB
    };
    
    let mut app = App::with_config(config);
    
    // Setup logging
    #[cfg(feature = "log")]
    {
        feather::info!("Starting server on {}:{}", host, port);
    }
    
    // Add routes
    app.get("/health", middleware!(|_req, res, _ctx| {
        res.send_text(r#"{"status":"ok"}"#);
        next!()
    }));
    
    // Start server
    let address = format!("{}:{}", host, port);
    app.listen(address);
}
```
