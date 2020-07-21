use std::io;

use ftp::{mock::test_users, Config, Server};

fn main() -> io::Result<()> {
    Server::new("127.0.0.1:21", Config::new(test_users())).run()
}
