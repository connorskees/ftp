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

pub struct Config {
    users: BTreeMap<String, String>,
}

impl Config {
    pub fn new() -> Self {
        let mut users = BTreeMap::new();
        users.insert("a".to_owned(), "a".to_owned());
        users.insert("b".to_owned(), "b".to_owned());
        Self { users }
    }
}

pub struct Connection {
    reader: BufReader<TcpStream>,
    writer: BufWriter<TcpStream>,
    path: PathBuf,
    data_port: Option<u16>,
    username: Option<String>,
    config: Config,
}

impl Connection {
    pub fn new(stream: TcpStream, path: PathBuf) -> io::Result<Self> {
        let mut connection = Self {
            reader: BufReader::new(stream.try_clone()?),
            writer: BufWriter::new(stream),
            path,
            data_port: None,
            username: None,
            config: Config::new(),
        };

        connection.write_response(Code::ServiceReadyForNewUser, "Server ready for new user")?;

        connection.write_response(Code::NeedAccountForLogin, "Enter username.")?;

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

    fn read_arg(&mut self) -> io::Result<String> {
        let mut buffer = String::new();
        self.reader.read_line(&mut buffer)?;
        let mut s = buffer.trim_end_matches("\r\n");
        if let Some(stripped) = s.strip_prefix(' ') {
            s = stripped;
        }
        Ok(s.to_owned())
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
            "USER" => {
                if arg.is_empty() {
                    self.write_response(
                        Code::InvalidParametersOrArguments,
                        "Username may not be empty.",
                    )?;
                    return Ok(true);
                }

                println!("Found username: {:?}", arg);

                if !self.config.users.contains_key(&arg) {
                    self.write_response(Code::NotLoggedIn, "User does not exist.")?;
                    return Ok(true);
                }

                self.username = Some(arg);

                self.write_response(
                    Code::UserNameOkPasswordNeeded,
                    "Username Ok. Password needed.",
                )?;
            }
            "PASS" => {
                println!("Found password: {:?}", arg);

                if let Some(username) = &self.username {
                    if self.config.users.get(username) == Some(&arg) {
                        self.write_response(Code::UserLoggedIn, "Logged in.")?;
                    } else {
                        self.write_response(Code::NotLoggedIn, "Incorrect password.")?;
                    }
                } else {
                    self.write_response(Code::BadSequenceOfCommands, "Expected `USER`.")?;
                }
            }
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
            "NOOP" => self.write_response(Code::Ok, "NOOP")?,
            "OPTS" => {
                println!("Found opts: {:?}", arg);

                match arg.to_ascii_lowercase().as_str() {
                    "utf8 on" => self.write_response(Code::Ok, "Ok, UTF-8 enabled.")?,
                    _ => self.write_response(Code::CommandNotImplemented, "Unknown option.")?,
                }
            }
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

    connection.command_loop()?;

    Ok(())
}
