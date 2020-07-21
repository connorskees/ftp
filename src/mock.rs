use std::{
    collections::BTreeMap,
    io::{BufReader, Read, Write},
    net::TcpStream,
    path::PathBuf,
    sync::atomic::{AtomicU16, Ordering},
    thread,
};

use crate::{Config, Server, Users};

const LOCALHOST: &str = "127.0.0.1";

/// The total number of mock servers, plus 60,000
///
/// We use this in order to bind on unique ports
static MOCK_COUNT: AtomicU16 = AtomicU16::new(60_000);

pub struct MockFtpServer {
    reader: BufReader<TcpStream>,
    writer: TcpStream,
}

pub fn test_users() -> Users {
    let mut users = BTreeMap::new();
    users.insert("a".to_owned(), "a".to_owned());
    users.insert("b".to_owned(), "b".to_owned());
    users
}

impl MockFtpServer {
    /// Creates a new server bound to localhost on a unique port
    pub fn new() -> Self {
        let port = MOCK_COUNT.fetch_add(1, Ordering::Relaxed);

        thread::spawn(move || {
            Server::new(
                (LOCALHOST, port),
                Config::new(test_users()),
                PathBuf::from("."),
            )
            .run()
        });

        let connection = TcpStream::connect((LOCALHOST, port)).unwrap();

        let writer = connection.try_clone().unwrap();
        let reader = BufReader::new(connection);

        let mut server = MockFtpServer { writer, reader };

        server.assert_output(b"220 Server ready for new user.\r\n");
        server.assert_output(b"332 Enter username.\r\n");

        server.send_bytes(b"USER a\r\n");
        server.assert_output(b"331 Username Ok. Password needed.\r\n");

        server.send_bytes(b"PASS a\r\n");
        server.assert_output(b"230 Logged in.\r\n");

        server
    }

    /// Sends all bytes given, panicking if sending failed
    pub fn send_bytes(&mut self, bytes: &[u8]) {
        self.writer.write_all(bytes).unwrap()
    }

    pub fn assert_output(&mut self, output: &[u8]) {
        let mut output_buf = vec![0; output.len()];

        self.reader.read_exact(&mut output_buf).unwrap();

        assert_eq!(output, output_buf.as_slice())
    }

    pub fn quit(mut self) {
        self.send_bytes(b"QUIT\r\n")
    }
}
