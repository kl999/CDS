use std::{
    io::Error,
    net::{SocketAddr, UdpSocket},
    rc::Rc,
};

use crate::{message::Message, socket_worker::SocketWorker};

/// Sets up a UDP server that waits for client handshake requests.
///
/// Creates a socket on the specified address, waits for a "Hello" message,
/// then creates a dedicated communication channel with the client.
///
/// # Arguments
///
/// * `address` - Server bind address (e.g., "127.0.0.1:8080")
/// * `notify` - Callback function for received messages
///
/// # Examples
///
/// ```rust,no_run
/// # use udp_connection::socket_worker_handshake::receive_handshake;
/// let mut worker = receive_handshake(
///     "127.0.0.1:8080".to_string(),
///     |msg| println!("Received: {}", msg)
/// ).expect("Failed to start server");
/// ```
pub fn receive_handshake(
    address: String,
    notify: fn(Rc<Message>),
) -> std::io::Result<SocketWorker> {
    let sock = UdpSocket::bind(&address)?;

    let (new_sock, new_adr) = expect_handshake(sock)?;

    new_sock.set_nonblocking(true)?;

    Ok(SocketWorker::new(new_sock, new_adr, notify))
}

/// Initiates a handshake with a UDP server and establishes connection.
///
/// Sends "Hello" to the server, receives connection details, and creates
/// a SocketWorker for reliable message exchange.
///
/// # Arguments
///
/// * `address` - Server address to connect to (e.g., "127.0.0.1:8080")
/// * `notify` - Callback function for received messages
///
/// # Examples
///
/// ```rust,no_run
/// # use udp_connection::socket_worker_handshake::send_handshake;
/// let mut worker = send_handshake(
///     "127.0.0.1:8080".to_string(),
///     |msg| println!("Received: {}", msg)
/// ).expect("Failed to connect");
/// ```
pub fn send_handshake(address: String, notify: fn(Rc<Message>)) -> std::io::Result<SocketWorker> {
    let sock = UdpSocket::bind("127.0.0.1:0")?;

    let buf = "Hello".as_bytes();
    sock.send_to(buf, address)?;

    let mut buf = [0; 20];

    let (number_of_bytes, server_address) = sock.recv_from(&mut buf)?;
    let msg = String::from_utf8_lossy(&buf[..number_of_bytes]).to_string();

    if !msg.starts_with("Connect port ") {
        return Err(Error::new(
            std::io::ErrorKind::Unsupported,
            format!("Unknown message '{}'", msg),
        ));
    }

    let num_str = &msg[13..].trim();

    let port = match num_str.parse::<u16>() {
        Ok(num) => num,
        Err(_) => {
            return Err(Error::new(
                std::io::ErrorKind::Unsupported,
                format!("Unknown port format '{}'", msg),
            ))
        }
    };

    let socket_addr = SocketAddr::new(server_address.ip(), port);

    sock.set_nonblocking(true)?;

    Ok(SocketWorker::new(
        sock,
        socket_addr.to_string(),
        notify,
    ))
}

/// Handles server-side handshake protocol.
///
/// Waits for "Hello" message, creates a new dedicated socket,
/// and sends "Connect port {port}" response to the client.
///
/// # Returns
///
/// Tuple of (dedicated_socket, client_address_string)
///
/// # Examples
///
/// ```rust,no_run
/// # use std::net::UdpSocket;
/// let server_socket = UdpSocket::bind("127.0.0.1:8080").unwrap();
/// // expect_handshake waits for "Hello" and creates dedicated channel
/// ```
fn expect_handshake(sock: UdpSocket) -> std::io::Result<(UdpSocket, String)> {
    let mut buf = [0; 5];

    let (number_of_bytes, src_addr) = sock.recv_from(&mut buf)?;
    let msg = String::from_utf8_lossy(&buf[..number_of_bytes]).to_string();
    println!(
        "Received {} bytes from {}: '{}'",
        number_of_bytes, src_addr, msg
    );
    if msg == "Hello" {
        let con = UdpSocket::bind("127.0.0.1:0")?;
        let port = con.local_addr()?.port();
        let buf = format!("Connect port {}", port);
        sock.send_to(buf.as_bytes(), src_addr)?;
        //echo "Hello" | nc -u -w1 127.0.0.1 8080

        return Ok((con, src_addr.to_string()));
    } else {
        Err(Error::new(
            std::io::ErrorKind::Unsupported,
            format!("Unknown message '{}'", msg),
        ))
    }
}
