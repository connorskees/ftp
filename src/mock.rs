use std::{
    collections::BTreeMap,
    io::{BufRead, BufReader, Read, Write},
    net::TcpStream,
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

        thread::spawn(move || Server::new((LOCALHOST, port), Config::new(test_users())).run());

        let connection = TcpStream::connect((LOCALHOST, port)).unwrap();

        let writer = connection.try_clone().unwrap();
        let reader = BufReader::new(connection);

        MockFtpServer { writer, reader }
    }

    /// Sends all bytes given, panicking if sending failed
    pub fn send_bytes(&mut self, bytes: &[u8]) {
        self.writer.write_all(bytes).unwrap()
    }

    pub fn assert_output(&mut self, output: &[u8]) {
        // let output =
    }
}
