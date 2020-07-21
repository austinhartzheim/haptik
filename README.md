# haptik

[![Build Status](https://travis-ci.org/austinhartzheim/haptik.svg?branch=master)](https://travis-ci.org/austinhartzheim/haptik)
[![Coverage Status](https://coveralls.io/repos/github/austinhartzheim/haptik/badge.svg?branch=master)](https://coveralls.io/github/austinhartzheim/haptik?branch=master)


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

## Developing
1. Start HAProxy via Docker by running this command at the base directory for this project:
```sh
docker run -d -v $(pwd)/examples/haproxy.cfg:/usr/local/etc/haproxy/haproxy.cfg -v /tmp/socket:/var/run --entrypoint haproxy -p 9999:9999 haproxy:latest -db -f /usr/local/etc/haproxy/haproxy.cfg
```
2. Run the test suite. The ignored tests require a running HAProxy instance (configured in step 1).
```sh
cargo test
cargo test -- --ignored
```