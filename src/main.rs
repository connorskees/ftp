use std::{
    collections::BTreeMap,
    fs,
    io::{self, BufRead, BufReader, BufWriter, Read, Write},
    net::{TcpListener, TcpStream},
    path::PathBuf,
};

use crate::response::Code;

mod command;
mod data;
mod response;

struct Options {
    users: BTreeMap<String, String>,
}

pub struct Server {
    reader: BufReader<TcpStream>,
    writer: BufWriter<TcpStream>,
    path: PathBuf,
    data_port: Option<String>,
}

impl Server {
    pub async fn new(stream: TcpStream, path: PathBuf) -> io::Result<Self> {
        let mut server = Self {
            reader: BufReader::new(stream.try_clone()?),
            writer: BufWriter::new(stream),
            path,
            data_port: None,
        };

        server
            .write_response(Code::ServiceReadyForNewUser, "Server ready for new user")
            .await?;

        server.write_response(Code::Ok, "Ok").await?;

        server.read_opts().await?;

        Ok(server)
    }

    pub async fn write_response(&mut self, code: Code, message: &str) -> io::Result<()> {
        if message.contains('\n') {
            write!(self.writer, "{}-", code)?;

            let mut lines = message.split('\n').peekable();

            while let Some(line) = lines.next() {
                if lines.peek().is_some() {
                    if line.starts_with(|c: char| c.is_ascii_digit()) {
                        self.writer.write(b"  ")?;
                    }
                    write!(self.writer, "{}\r\n", line)?;
                } else {
                    write!(self.writer, "{} {}\r\n", code, line)?;
                }
            }
        } else {
            write!(self.writer, "{} {}\r\n", code, message)?;
        }
        self.writer.flush()?;
        Ok(())
    }

    pub async fn read_opts(&mut self) -> io::Result<()> {
        if !self.expect_command(b"OPTS ").await? {
            return Ok(());
        }

        let opts = self.read_arg().await?;

        println!("Found opts: {:?}", opts);

        Ok(())
    }

    async fn read_arg(&mut self) -> io::Result<String> {
        let mut buffer = String::new();
        self.reader.read_line(&mut buffer)?;
        let mut s = buffer.trim_end_matches("\r\n");
        if let Some(stripped) = s.strip_prefix(' ') {
            s = stripped;
        }
        Ok(s.to_owned())
    }

    /// Returns true if command was found
    async fn expect_command(&mut self, command: &[u8]) -> io::Result<bool> {
        let mut command_buf = vec![0; command.len()];

        let len = self.reader.read(&mut command_buf)?;

        if len > 0 && command_buf != command {
            self.write_response(Code::BadSequenceOfCommands, "Bad sequence of commands.")
                .await?;
            println!("Found {:?}.", command);
            return Ok(false);
        }
        Ok(true)
    }

    pub async fn login(&mut self) -> io::Result<()> {
        self.write_response(Code::NeedAccountForLogin, "Enter username.")
            .await?;

        if !self.expect_command(b"USER ").await? {
            return Ok(());
        }

        let username = self.read_arg().await?;

        if username.is_empty() {
            self.write_response(
                Code::InvalidParametersOrArguments,
                "Username may not be empty.",
            )
            .await?;
            return Ok(());
        }

        println!("Found username: {:?}", username);

        self.write_response(
            Code::UserNameOkPasswordNeeded,
            "Username Ok. Password needed.",
        )
        .await?;

        if !self.expect_command(b"PASS ").await? {
            return Ok(());
        }

        let password = self.read_arg().await?;

        println!("Found password: {:?}", password);

        self.write_response(Code::UserLoggedIn, "Logged in.")
            .await?;

        Ok(())
    }

    async fn read_cmd(&mut self) -> io::Result<bool> {
        let mut command = vec![0; 4];

        self.reader.read_exact(&mut command)?;

        let command = match String::from_utf8(command) {
            Ok(mut cmd) => {
                cmd.make_ascii_uppercase();
                cmd
            }
            Err(..) => {
                self.write_response(Code::CommandNotImplemented, "Command was not valid UTF-8.")
                    .await?;
                return Ok(true);
            }
        };

        let arg = self.read_arg().await?;

        match command.as_str() {
            "USER" => todo!(),
            "PASS" => todo!(),
            "ACCT" => todo!(),
            "XCWD" | "CWD " => {
                let path = self.path.join(arg);
                if !path.is_dir() {
                    self.write_response(
                        Code::InvalidParametersOrArguments,
                        "Path is not a directory.",
                    )
                    .await?;
                    return Ok(true);
                }
                self.path = path;
                self.write_response(Code::Ok, "Changed directory.").await?
            }
            "CDUP" => todo!(),
            "SMNT" => todo!(),
            "QUIT" => {
                self.write_response(Code::ServiceClosing, "Goodbye!")
                    .await?;
                return Ok(false);
            }
            "REIN" => todo!(),
            "PORT" => {
                self.data_port = Some(arg);
                self.write_response(Code::Ok, "Changed port.").await?;
            }
            "PASV" => todo!(),
            "TYPE" => todo!(),
            "STRU" => todo!(),
            "MODE" => todo!(),
            "RETR" => todo!(),
            "STOR" => todo!(),
            "STOU" => todo!(),
            "APPE" => todo!(),
            "ALLO" => todo!(),
            "REST" => todo!(),
            "RNFR" => todo!(),
            "RNTO" => todo!(),
            "ABOR" => todo!(),
            "DELE" => todo!(),
            "XRMD" => todo!(),
            "XMKD" => todo!(),
            "XPWD" => {
                let path: String = self.path.to_string_lossy().into();
                self.write_response(Code::Ok, &path).await?
            }
            "LIST" => todo!(),
            "NLST" => {
                let path = self.path.join(arg);
                let dirs = fs::read_dir(path)?
                    .map(|entry| Ok(format!("{:?}", entry?.file_name())))
                    .collect::<io::Result<Vec<String>>>()?
                    .join("\n");
                self.write_response(Code::Ok, &dirs).await?;
            }
            "SITE" => todo!(),
            "SYST" => todo!(),
            "STAT" => todo!(),
            "HELP" => todo!(),
            "NOOP" => todo!(),
            cmd => todo!("command not recognized: {:?}", cmd),
        }

        Ok(true)
    }

    pub async fn command_loop(&mut self) -> io::Result<()> {
        loop {
            if !self.read_cmd().await? {
                break;
            }
        }
        Ok(())
    }
}

#[tokio::main]
async fn main() -> io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:21").unwrap();

    for stream in listener.incoming() {
        let stream = stream?;

        handle_connection(stream).await?;
    }

    Ok(())
}

async fn handle_connection(stream: TcpStream) -> io::Result<()> {
    let mut server = Server::new(stream, PathBuf::from("/")).await?;

    server.login().await?;

    server.command_loop().await?;

    Ok(())
}
