# haptik
HAProxy control via Unix sockets.

Issue [commands](https://cbonte.github.io/haproxy-dconv/2.2/management.html#9.3) to HA Proxy.

Example:
```rust
let client = haptik::Client::default();
let info = client.show_info();
println!("{:?}", info);
``
