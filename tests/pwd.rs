use ftp::mock::MockFtpServer;

#[test]
fn simple_pwd() {
    let mut server = MockFtpServer::new();
    server.send_bytes(b"PWD\r\n");
    server.assert_output(b"200 .\r\n");
    server.quit();
}

#[test]
fn ignores_args() {
    let mut server = MockFtpServer::new();
    server.send_bytes(b"PWD abc123\r\n");
    server.assert_output(b"200 .\r\n");
    server.quit();
}
