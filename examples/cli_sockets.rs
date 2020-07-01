/// Connect to the HAProxy Unix socket located at the default address. Once connected, enumerate
/// all CLI sockets via a `show cli sockets` command. Display all sockets and attempt to connect
/// to any supported sockets, reporting any errors.
use haptik::responses::CliSocketAddr;
use haptik::{ConnectionBuilder, UnixSocketBuilder};

fn main() {
    println!("attempting connection to default Unix socket");
    let connection = UnixSocketBuilder::default()
        .connect()
        .expect("Unix socket connection failed");

    let sockets = connection.cli_sockets().unwrap();
    for socket in sockets {
        println!("attempting connection to socket: {:?}", socket);
        match socket.address {
            CliSocketAddr::Unix(path) => {
                let connection_builder: UnixSocketBuilder = path.into();
                match connection_builder.connect() {
                    Ok(_) => println!(" - connected successfully"),
                    Err(err) => println!(" - failed to connect: {}", err),
                };
            }
            _ => println!(" - socket type not supported"),
        }
    }
}
