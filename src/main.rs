use std::{
    collections::BTreeMap,
    fs,
    io::{self, BufRead, BufReader, BufWriter, Read, Write},
    net::{Ipv4Addr, TcpListener, TcpStream},
    path::PathBuf,
    str::FromStr,
    thread,
};

use crate::response::Code;

mod command;
mod data;
mod response;

struct Options {
    users: BTreeMap<String, String>,
}

pub struct Connection {
    reader: BufReader<TcpStream>,
    writer: BufWriter<TcpStream>,
    path: PathBuf,
    data_port: Option<u16>,
}

impl Connection {
    pub fn new(stream: TcpStream, path: PathBuf) -> io::Result<Self> {
        let mut connection = Self {
            reader: BufReader::new(stream.try_clone()?),
            writer: BufWriter::new(stream),
            path,
            data_port: None,
        };

        connection.write_response(Code::ServiceReadyForNewUser, "Server ready for new user")?;

        connection.write_response(Code::Ok, "Ok")?;

        connection.read_opts()?;

        Ok(connection)
    }

    pub fn write_response(&mut self, code: Code, message: &str) -> io::Result<()> {
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

    pub fn read_opts(&mut self) -> io::Result<()> {
        if !self.expect_command(b"OPTS ")? {
            return Ok(());
        }

        let opts = self.read_arg()?;

        println!("Found opts: {:?}", opts);

        Ok(())
    }

    fn read_arg(&mut self) -> io::Result<String> {
        let mut buffer = String::new();
        self.reader.read_line(&mut buffer)?;
        let mut s = buffer.trim_end_matches("\r\n");
        if let Some(stripped) = s.strip_prefix(' ') {
            s = stripped;
        }
        Ok(s.to_owned())
    }

    /// Returns true if command was found
    fn expect_command(&mut self, command: &[u8]) -> io::Result<bool> {
        let mut command_buf = vec![0; command.len()];

        let len = self.reader.read(&mut command_buf)?;

        if len > 0 && command_buf != command {
            self.write_response(Code::BadSequenceOfCommands, "Bad sequence of commands.")?;
            println!("Found {:?}.", command);
            return Ok(false);
        }
        Ok(true)
    }

    pub fn login(&mut self) -> io::Result<()> {
        self.write_response(Code::NeedAccountForLogin, "Enter username.")?;

        if !self.expect_command(b"USER ")? {
            return Ok(());
        }

        let username = self.read_arg()?;

        if username.is_empty() {
            self.write_response(
                Code::InvalidParametersOrArguments,
                "Username may not be empty.",
            )?;
            return Ok(());
        }

        println!("Found username: {:?}", username);

        self.write_response(
            Code::UserNameOkPasswordNeeded,
            "Username Ok. Password needed.",
        )?;

        if !self.expect_command(b"PASS ")? {
            return Ok(());
        }

        let password = self.read_arg()?;

        println!("Found password: {:?}", password);

        self.write_response(Code::UserLoggedIn, "Logged in.")?;

        Ok(())
    }

    fn read_cmd(&mut self) -> io::Result<bool> {
        let mut command = vec![0; 4];

        self.reader.read_exact(&mut command)?;

        let command = match String::from_utf8(command) {
            Ok(mut cmd) => {
                cmd.make_ascii_uppercase();
                cmd
            }
            Err(..) => {
                self.write_response(Code::CommandNotImplemented, "Command was not valid UTF-8.")?;
                return Ok(true);
            }
        };

        let arg = self.read_arg()?;

        println!("Command: {:?}", command);
        println!("Arg: {:?}", arg);

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
                    )?;
                    return Ok(true);
                }
                self.path = path;
                self.write_response(Code::Ok, "Changed directory.")?
            }
            "CDUP" => todo!(),
            "SMNT" => todo!(),
            "QUIT" => {
                self.write_response(Code::ServiceClosing, "Goodbye!")?;
                return Ok(false);
            }
            "REIN" => todo!(),
            "PORT" => {
                let mut vals: Vec<&str> = arg.split(',').collect();
                let port = vals.pop().unwrap().parse::<u16>().unwrap()
                    + (vals.pop().unwrap().parse::<u16>().unwrap() << 8);
                self.data_port = Some(port);
                let _ip = match Ipv4Addr::from_str(&vals.join(".")) {
                    Ok(addr) => addr,
                    Err(..) => {
                        self.write_response(
                            Code::InvalidParametersOrArguments,
                            "IP not in valid format.",
                        )?;
                        return Ok(true);
                    }
                };
                self.write_response(Code::Ok, "Changed port.")?;
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
            "XPWD" | "PWD\r" | "PWD " => {
                let path: String = self.path.to_string_lossy().into();
                self.write_response(Code::Ok, &path)?
            }
            "LIST" => todo!(),
            "NLST" => {
                let path = self.path.join(arg);
                let dirs = fs::read_dir(path)?
                    .map(|entry| {
                        Ok(entry?
                            .file_name()
                            .to_str()
                            .unwrap_or("Invalid UTF-8.")
                            .to_owned())
                    })
                    .collect::<io::Result<Vec<String>>>()?
                    .join("\n");
                self.write_response(Code::Ok, &dirs)?;
            }
            "SITE" => todo!(),
            "SYST" => todo!(),
            "STAT" => todo!(),
            "HELP" => todo!(),
            "NOOP" => todo!(),
            cmd => {
                println!("Command not recognized: {:?}", cmd);
                self.write_response(Code::CommandUnrecognized, "Command not recognized.")?;
            }
        }

        Ok(true)
    }

    pub fn command_loop(&mut self) -> io::Result<()> {
        loop {
            if !self.read_cmd()? {
                break;
            }
        }
        Ok(())
    }
}

fn main() -> io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:21").unwrap();

    for stream in listener.incoming() {
        let stream = stream?;

        thread::spawn(|| handle_connection(stream));
    }

    Ok(())
}

fn handle_connection(stream: TcpStream) -> io::Result<()> {
    let mut connection = Connection::new(stream, PathBuf::from("/"))?;

    connection.login()?;

    connection.command_loop()?;

    Ok(())
}
