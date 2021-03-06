use std::{io, path::PathBuf};

use ftp::{mock::test_users, Config, Server};

fn main() -> io::Result<()> {
    env_logger::init();

    Server::new(
        "127.0.0.1:21",
        Config::new(test_users()),
        PathBuf::from("/"),
    )
    .run()
}
