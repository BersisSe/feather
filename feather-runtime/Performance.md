# Feather Runtime Performance

in the latest update see the [changelog](../CHANGELOG.md), we have made significant improvements to the performance of the Feather Runtime.

## Performance Improvements
- **Cross-Beam**: Switched from using the `rusty-pool` crate to `crossbeam` and a Custom `TaskPool` for thread management. This change allows for better handling of threads and improved performance when dealing with concurrent requests.
- **Connection Management**: The connection management system has been revamped to handle connections more efficiently. This includes better handling of idle connections and improved resource management.
- **Error Handling and Logging**: The error handling and logging mechanisms have been improved to provide more detailed information about errors and performance issues. This will help developers identify and resolve issues more quickly.
- **Messages Queue**: A new messages queue has been introduced to manage requests more efficiently. This allows for better handling of concurrent requests and reduces the risk of bottlenecks in the system.
- **Task Pool**: A new task pool has been added to manage concurrent tasks more effectively. This new task pool can dynamicly adjust the number of threads based on the workload, allowing for better resource utilization and improved performance.
- **Concurrency**: The new concurrency model allows for better handling of multiple requests at the same time, improving overall performance and responsiveness of the runtime.


Lets take a look at some of the performance benchmarks to see how these changes have impacted the runtime.
## Performance Benchmarks

This Code snippet is taken from the example folder.
```rust
use feather_runtime::{http::HttpResponse, server::server::Server};
fn main() {
    let config = feather_runtime::server::server::ServerConfig{
        address: "127.0.0.1:5000".to_string(),
    };
    let server = Server::new(config);
    
    server.incoming().for_each(|_r|{
        let mut resp = HttpResponse::default();
        resp.send_text("Hello from Feather!");
        return resp;
    }).unwrap();
}
```
Lets Test It Using wrk
```bash
wrk -t10 -c400 -d5s http://127.0.0.1:5000
```
### Results
```bash
Running 5s test @ http://127.0.0.1:5000
  10 threads and 400 connections
  Thread Stats   Avg      Stdev     Max   +/- Stdev
    Latency   145.64us   57.59us   8.98ms   97.06%
    Req/Sec    45.62k     3.88k   50.42k    80.39%
  231099 requests in 5.10s, 27.55MB read
Requests/sec:  45325.13 
Transfer/sec:   5.40MB
```
**Note**:*This is Bare Feather-Runtime(The Http Engine Behind Feather) Running With the Framework Overhead(Routing,Middlewares etc) it scores About 40000 RPS*.

**The old version had a bug in the writer so it could't even send a response wrk**.

## Comparison
Lets do the same test to **Express**

### Express
```js
const express = require('express')
const app = express()
const port = 3000

app.get('/', (req, res) => {
  res.send('Hello World!')
})

app.listen(port, () => {
  console.log(`Example app listening on port ${port}`)
})
```

Lets See how it performs
```bash
wrk -t10 -c400 -d5s http://127.0.0.1:3000
```
### Results
```bash
Running 5s test @ http://127.0.0.1:3000
  10 threads and 400 connections
  Thread Stats   Avg      Stdev     Max   +/- Stdev
    Latency    62.17ms  151.08ms   1.99s    97.08%
    Req/Sec   690.47    370.71     1.36k    63.10%
  12872 requests in 5.03s, 2.93MB read
  Socket errors: connect 0, read 0, write 0, timeout 67
Requests/sec:   2559.24
Transfer/sec:    597.32KB
```
To be Fair Express is a full fledged framework and Feather-Runtime is just a Http Engine so we can not compare them directly but this is just to show how fast Feather is.
## Conclusion
The Feather Runtime has made significant improvements to its performance with the latest updates. The new concurrency model, connection management system, and task pool have all contributed to better handling of concurrent requests and improved overall performance. The benchmarks show that the Feather Runtime can handle a large number of requests per second, making it a great choice for high-performance applications.