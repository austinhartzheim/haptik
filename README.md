# haptik
HAProxy control via Unix sockets.

Issue [commands](https://cbonte.github.io/haproxy-dconv/2.2/management.html#9.3) to HAProxy.

Example:
```rust
use haptik::{ConnectionBuilder, UnixSocketBuilder};
let connection = UnixSocketBuilder::default().connect();
println!("Current permission level: {:?}", connection.level());
```
