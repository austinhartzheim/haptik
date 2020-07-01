# haptik
HAProxy control via Unix sockets.

Issue [commands](https://cbonte.github.io/haproxy-dconv/2.2/management.html#9.3) to HAProxy.

## Examples
Get the connection permission level:
```rust
use haptik::{ConnectionBuilder, UnixSocketBuilder};
let connection = UnixSocketBuilder::default().connect().expect("Failed to connect");
println!("Current permission level: {:?}", connection.level());
```

Get the list of CLI sockets:
```rust
use haptik::{ConnectionBuilder, UnixSocketBuilder};
let connection = UnixSocketBuilder::default().connect().expect("Failed to connect");
println!("Sockets: {:?}", connection.cli_sockets());
```