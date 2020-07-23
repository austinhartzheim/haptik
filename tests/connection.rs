use std::net::{Ipv4Addr, SocketAddrV4};

use haptik::models::AclId;
use haptik::requests::{BackendId, ErrorFlag};
use haptik::responses;
use haptik::{ConnectionBuilder, TcpSocketBuilder, UnixSocketBuilder};

#[test]
#[ignore]
fn unix_socket_builder_connects() {
    let builder = UnixSocketBuilder::new("/tmp/socket/haproxy.sock");
    assert!(
        builder.connect().is_ok(),
        "Failed to connect to the HAProxy Unix socket"
    );
}

#[test]
#[ignore]
fn tcp_socket_builder_connects() {
    let builder =
        TcpSocketBuilder::new(SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), 9999).into());
    assert!(
        builder.connect().is_ok(),
        "Failed to connect to the HAProxy TCP socket"
    );
}

#[test]
#[ignore]
fn connection_acl_add() {
    let ip = std::net::Ipv4Addr::new(255, 255, 255, 255);
    let acl_id = AclId::Id(1);

    let builder = UnixSocketBuilder::new("/tmp/socket/haproxy.sock");
    let connection = builder.connect().unwrap();
    connection.acl_add(acl_id, &ip).unwrap();

    // Check that the ACL contains the new entry
    let connection = builder.connect().unwrap();
    let acl_data = connection.acl_data::<std::net::Ipv4Addr>(acl_id).unwrap();
    assert!(acl_data.iter().any(|entry| entry.value == ip))
}

#[test]
#[ignore]
fn connection_acl_data() {
    let builder = UnixSocketBuilder::new("/tmp/socket/haproxy.sock");
    let connection = builder.connect().unwrap();
    let acl_data = connection
        .acl_data::<std::net::IpAddr>(AclId::Id(0))
        .unwrap();
    assert_eq!(acl_data.len(), 2);
    assert_eq!(
        acl_data[0].value,
        std::net::IpAddr::V4("127.0.0.1".parse().unwrap())
    );
    assert_eq!(
        acl_data[1].value,
        std::net::IpAddr::V4("127.0.0.2".parse().unwrap())
    );
}

#[test]
#[ignore]
fn connection_acl_data_str() {
    let builder = UnixSocketBuilder::new("/tmp/socket/haproxy.sock");
    let connection = builder.connect().unwrap();
    let acl_data = connection.acl_data::<String>(AclId::Id(0)).unwrap();
    assert_eq!(acl_data.len(), 2);
    assert_eq!(acl_data[0].value, "127.0.0.1",);
    assert_eq!(acl_data[1].value, "127.0.0.2",);
}

#[test]
#[ignore]
fn connection_acl_list() {
    let builder = UnixSocketBuilder::new("/tmp/socket/haproxy.sock");
    let connection = builder.connect().unwrap();
    let acls = connection.acl_list().unwrap();

    assert_eq!(acls.len(), 2);

    assert_eq!(acls[0].id, 0);
    assert_eq!(acls[0].reference, None);
    assert_eq!(
        acls[0].description,
        "acl 'src' file '/usr/local/etc/haproxy/haproxy.cfg' line 20"
    );
    assert_eq!(acls[1].id, 1);
    assert_eq!(acls[1].reference, None);
    assert_eq!(
        acls[1].description,
        "acl 'src' file '/usr/local/etc/haproxy/haproxy.cfg' line 21"
    );
}

#[test]
#[ignore]
fn connection_cli_sockets() {
    let builder = UnixSocketBuilder::new("/tmp/socket/haproxy.sock");
    let connection = builder.connect().unwrap();
    let sockets = connection.cli_sockets().unwrap();

    assert_eq!(sockets.len(), 3);
    assert_eq!(
        sockets[0],
        responses::CliSocket {
            address: responses::CliSocketAddr::Unix("/var/run/haproxy.sock".into()),
            level: responses::Level::Admin,
            processes: responses::CliSocketProcesses::All
        }
    );
    assert_eq!(
        sockets[1],
        responses::CliSocket {
            address: responses::CliSocketAddr::Ip("127.0.0.1:9999".parse().unwrap()),
            level: responses::Level::Admin,
            processes: responses::CliSocketProcesses::All
        }
    );
    assert_eq!(
        sockets[2],
        responses::CliSocket {
            address: responses::CliSocketAddr::Ip("[::]:9999".parse().unwrap()),
            level: responses::Level::Admin,
            processes: responses::CliSocketProcesses::All
        }
    );
}

#[test]
#[ignore]
fn connection_level() {
    let builder = UnixSocketBuilder::new("/tmp/socket/haproxy.sock");
    let connection = builder.connect().unwrap();
    assert_eq!(connection.level().unwrap(), responses::Level::Admin);
}

#[test]
#[ignore]
fn connection_errors() {
    let builder = UnixSocketBuilder::new("/tmp/socket/haproxy.sock");
    let connection = builder.connect().unwrap();
    assert_eq!(connection.errors().unwrap(), 0);
}

#[test]
#[ignore]
fn connection_errors_backend_all() {
    let builder = UnixSocketBuilder::new("/tmp/socket/haproxy.sock");
    let connection = builder.connect().unwrap();
    assert_eq!(
        connection
            .errors_backend(BackendId::All, ErrorFlag::All)
            .unwrap(),
        0
    );
}

#[test]
#[ignore]
fn connection_errors_backend_id() {
    let builder = UnixSocketBuilder::new("/tmp/socket/haproxy.sock");
    let connection = builder.connect().unwrap();
    assert_eq!(
        connection
            .errors_backend(BackendId::Id(1), ErrorFlag::Request)
            .unwrap(),
        0
    );
}
