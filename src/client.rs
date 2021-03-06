use std::{
    io::{self, stdin, stdout, BufRead, BufReader, Read, Stdin, Stdout, Write},
    net::TcpStream,
};

use ftp::Code;

struct FtpConnection {
    reader: BufReader<TcpStream>,
    writer: TcpStream,
    code: [u8; 3],
    message: String,
    stdout: Stdout,
}

impl FtpConnection {
    pub fn new(connection: TcpStream) -> io::Result<Self> {
        let reader = BufReader::new(connection.try_clone()?);
        let writer = connection;

        let stdout = stdout();

        Ok(Self {
            reader,
            writer,
            code: [0; 3],
            message: String::new(),
            stdout,
        })
    }

    pub fn write(&mut self, bytes: &[u8]) -> io::Result<()> {
        self.writer.write_all(bytes)
    }

    pub fn wait_until_code(&mut self, response_code: Code) -> io::Result<()> {
        while self.read_cmd()? {
            if Code::from_bytes(self.code) == Some(response_code) {
                break;
            }
        }

        Ok(())
    }

    /// Returns true if it did not quit
    pub fn read_cmd(&mut self) -> io::Result<bool> {
        self.reader.read_exact(&mut self.code)?;

        let mut space_or_dash = [0];

        self.reader.read_exact(&mut space_or_dash)?;

        self.reader.read_line(&mut self.message)?;

        if space_or_dash == [b'-'] {
            let prefix = &self
                .code
                .iter()
                .map(|b| char::from(*b))
                .chain(std::iter::once(' '))
                .collect::<String>();

            loop {
                let message_len = self.message.len();
                self.reader.read_line(&mut self.message)?;
                if self.message[message_len..].starts_with(prefix) {
                    break;
                }
            }
        }

        self.stdout.write(&self.code)?;
        self.stdout.write(&space_or_dash)?;
        self.stdout.write(self.message.as_bytes())?;

        self.stdout.flush()?;

        self.message.clear();

        let code = match Code::from_bytes(self.code) {
            Some(c) => c,
            None => return Ok(true),
        };

        Ok(match code {
            Code::ServiceClosing => false,
            _ => true,
        })
    }

    pub fn write_stdout(&mut self, bytes: &[u8]) -> io::Result<()> {
        self.stdout.write(bytes)?;
        self.stdout.flush()
    }

    pub fn prompt_login(&mut self, stdin: &mut BufReader<Stdin>) -> io::Result<()> {
        loop {
            self.write_stdout(b"User (127.0.0.1:(none)): ")?;

            let mut username = String::new();
            stdin.read_line(&mut username)?;

            self.write(b"USER ")?;
            self.write(username.trim().as_bytes())?;
            self.write(b"\r\n")?;

            self.read_cmd()?;

            match Code::from_bytes(self.code) {
                Some(Code::UserNameOkPasswordNeeded) => {}
                _ => continue,
            }

            self.write_stdout(b"Password: ")?;

            let mut password = String::new();
            stdin.read_line(&mut password)?;
            self.write(b"PASS ")?;
            self.write(password.trim().as_bytes())?;
            self.write(b"\r\n")?;

            self.read_cmd()?;

            match Code::from_bytes(self.code) {
                Some(Code::UserLoggedIn) => break,
                _ => continue,
            }
        }

        Ok(())
    }
}

fn main() -> io::Result<()> {
    let addr = match std::env::args().skip(1).next() {
        Some(addr) => addr,
        None => {
            eprintln!("Missing argument: IP");
            std::process::exit(1);
        }
    };
    let mut connection = FtpConnection::new(TcpStream::connect((addr, 21))?)?;

    connection.wait_until_code(Code::ServiceReadyForNewUser)?;

    let mut stdin = BufReader::new(stdin());

    connection.prompt_login(&mut stdin)?;

    let mut line = String::new();

    loop {
        connection.write_stdout(b"ftp> ")?;
        stdin.read_line(&mut line)?;

        if line.trim().len() < 3 {
            line.clear();
            continue;
        }

        connection.write(line.as_bytes())?;

        line.clear();

        if !connection.read_cmd()? {
            break;
        }
    }

    Ok(())
}
