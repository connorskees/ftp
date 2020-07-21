use lazy_static::lazy_static;

use ftp::mock::MockFtpServer;

lazy_static! {
    static ref PWD_SERVER: MockFtpServer = MockFtpServer::new();
}

#[test]
fn it_works() {}
