use std::{
    collections::BTreeMap,
    fs,
    io::{self, BufRead, BufReader, Read, Write},
    net::{Ipv4Addr, Shutdown, TcpListener, TcpStream, ToSocketAddrs},
    path::PathBuf,
    str::FromStr,
    sync::Arc,
    thread,
};

pub type Users = BTreeMap<String, String>;

use log::debug;

use crate::data::{DataStructure, DataType, TransferMode};
pub use crate::response::Code;

mod data;
pub mod mock;
mod response;

pub struct Config {
    users: Users,
}

impl Config {
    pub fn new(users: Users) -> Self {
        Self { users }
    }
}

pub struct Connection {
    reader: BufReader<TcpStream>,
    writer: TcpStream,
    path: PathBuf,
    username: Option<String>,
    config: Arc<Config>,
    data_type: DataType,
    data_structure: DataStructure,
    transfer_mode: TransferMode,
    data_connection: Option<TcpStream>,
}

impl Connection {
    pub fn new(stream: TcpStream, path: PathBuf, config: Arc<Config>) -> io::Result<Self> {
        let mut connection = Self {
            reader: BufReader::new(stream.try_clone()?),
            writer: stream,
            path,
            username: None,
            config,
            data_type: DataType::default(),
            data_structure: DataStructure::default(),
            transfer_mode: TransferMode::default(),
            data_connection: None,
        };

        debug!("Beginning new connection.");

        connection.write_response(Code::ServiceReadyForNewUser, "Server ready for new user.")?;

        connection.write_response(Code::NeedAccountForLogin, "Enter username.")?;

        Ok(connection)
    }

    pub fn write_response(&mut self, code: Code, message: &str) -> io::Result<()> {
        debug!("Writing response: {:?} {:?}", code, message);

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

        Ok(())
    }

    pub fn write_to_data_connection(&mut self, bytes: &[u8]) -> io::Result<()> {
        self.write_response(Code::FileStatusOk, "Connecting to data port.")?;
        if let Some(connection) = self.data_connection.take().as_mut() {
            connection.write_all(bytes)?;

            if !bytes.ends_with(b"\r\n") {
                connection.write_all(b"\r\n")?;
            }

            connection.flush()?;
            connection.shutdown(Shutdown::Both)?;
        } else {
            self.write_response(Code::CannotOpenDataConnection, "No data connection")?;
            return Ok(());
        }

        self.write_response(Code::ClosingDataConnection, "Closing connection")?;

        Ok(())
    }

    fn read_arg(&mut self) -> io::Result<String> {
        let mut buffer = String::new();
        self.reader.read_line(&mut buffer)?;
        Ok(buffer.trim().to_owned())
    }

    fn unrecognized_command(&mut self, cmd: &str) -> io::Result<()> {
        debug!("Command not recognized: {:?}", cmd);
        self.write_response(Code::CommandUnrecognized, "Command not recognized.")?;
        Ok(())
    }

    fn read_cmd(&mut self) -> io::Result<bool> {
        let mut command = vec![0; 4];

        let cmd_len = self.reader.read(&mut command)?;

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

        debug!("Command: {:?}", command);

        if cmd_len < 4 || command.trim().len() < 3 {
            self.unrecognized_command(&command)?;
            return Ok(true);
        }

        let arg = self.read_arg()?;

        debug!("Arg: {:?}", arg);

        match command.as_str() {
            "USER" => {
                if arg.is_empty() {
                    self.write_response(
                        Code::InvalidParametersOrArguments,
                        "Username may not be empty.",
                    )?;
                    return Ok(true);
                }

                debug!("Found username: {:?}", arg);

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
                debug!("Found password: {:?}", arg);

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

                let ip = match Ipv4Addr::from_str(&vals.join(".")) {
                    Ok(addr) => addr,
                    Err(..) => {
                        self.write_response(
                            Code::InvalidParametersOrArguments,
                            "IP not in valid format.",
                        )?;
                        return Ok(true);
                    }
                };

                debug!("Opening data port on {}:{}", ip, port);

                self.data_connection = Some(TcpStream::connect((ip, port))?);

                self.write_response(Code::Ok, "Changed port.")?;
            }
            "PASV" => todo!(),
            "TYPE" => self.type_cmd(arg)?,
            "STRU" => self.stru(arg)?,
            "MODE" => self.mode(arg)?,
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
            "XRMD" | "RMD " | "RMD\r" => self.rmd(arg)?,
            "XMKD" | "MKD " | "MKD\r" => {
                let path = self.path.join(arg);

                if !path.exists() {
                    fs::create_dir(&path)?;
                }

                self.write_response(
                    Code::PathNameCreated,
                    &format!("Successfully created {:?}.", path),
                )?;
            }
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
                    .join("\r\n");

                self.write_to_data_connection(dirs.as_bytes())?;
            }
            "SITE" => todo!(),
            "SYST" => todo!(),
            "STAT" => todo!(),
            "HELP" => todo!(),
            "NOOP" => self.write_response(Code::Ok, "NOOP")?,
            "OPTS" => self.opts(arg)?,
            cmd => self.unrecognized_command(cmd)?,
        }

        Ok(true)
    }

    fn opts(&mut self, arg: String) -> io::Result<()> {
        debug!("Found opts: {:?}", arg);

        match arg.to_ascii_lowercase().as_str() {
            "utf8 on" => self.write_response(Code::Ok, "Ok, UTF-8 enabled.")?,
            _ => self.write_response(Code::CommandNotImplemented, "Unknown option.")?,
        }

        Ok(())
    }

    fn rmd(&mut self, arg: String) -> io::Result<()> {
        let path = self.path.join(arg);

        if !path.exists() {
            self.write_response(
                Code::FileUnavailable,
                &format!("Error removing {:?}: No such file or directory.", path),
            )?;
            return Ok(());
        }

        match fs::remove_dir(&path) {
            Ok(()) => self.write_response(
                Code::RequestedFileActionComplete,
                &format!("Successfully deleted {:?}.", path),
            )?,
            Err(e) => self.write_response(
                Code::ActionNotTaken,
                &format!("Error deleting {:?}: {}.", path, e),
            )?,
        };

        Ok(())
    }

    fn mode(&mut self, arg: String) -> io::Result<()> {
        let transfer_mode = match arg.chars().next() {
            Some('S') | Some('s') => TransferMode::Stream,
            Some('B') | Some('b') => TransferMode::Block,
            Some('C') | Some('c') => TransferMode::Compressed,
            Some(c) => {
                self.write_response(
                    Code::CommandNotImplementedForThatParameter,
                    &format!("Unknown transfer mode: {}.", c),
                )?;
                return Ok(());
            }
            None => {
                self.write_response(Code::InvalidParametersOrArguments, "Missing argument.")?;
                return Ok(());
            }
        };

        self.transfer_mode = transfer_mode;

        self.write_response(
            Code::Ok,
            &format!("Transfer mode is now {}.", transfer_mode),
        )?;

        Ok(())
    }

    fn stru(&mut self, arg: String) -> io::Result<()> {
        let data_structure = match arg.chars().next() {
            Some('F') | Some('f') => DataStructure::Files,
            Some('R') | Some('r') => DataStructure::Record,
            Some('P') | Some('p') => DataStructure::Page,
            Some(c) => {
                self.write_response(
                    Code::CommandNotImplementedForThatParameter,
                    &format!("Unknown STRUcture: {}.", c),
                )?;
                return Ok(());
            }
            None => {
                self.write_response(Code::InvalidParametersOrArguments, "Missing argument.")?;
                return Ok(());
            }
        };

        self.data_structure = data_structure;

        self.write_response(Code::Ok, &format!("Structure is now {}.", data_structure))?;

        Ok(())
    }

    fn type_cmd(&mut self, arg: String) -> io::Result<()> {
        let data_type = match arg.chars().next() {
            Some('A') | Some('a') => DataType::Ascii,
            Some('E') | Some('e') => DataType::Ebcdic,
            Some('I') | Some('i') => DataType::Image,
            Some('L') => {
                if arg[1..].trim() != "8" {
                    self.write_response(
                        Code::CommandNotImplementedForThatParameter,
                        "Only 8-bit bytes are supported.",
                    )?;
                    return Ok(());
                }
                DataType::LocalType
            }
            Some(c) => {
                self.write_response(
                    Code::CommandNotImplementedForThatParameter,
                    &format!("Unknown TYPE: {}.", c),
                )?;
                return Ok(());
            }
            None => {
                self.write_response(Code::InvalidParametersOrArguments, "Missing argument.")?;
                return Ok(());
            }
        };

        self.data_type = data_type;
        self.write_response(Code::Ok, &format!("Type is now {}.", data_type))?;

        Ok(())
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

pub struct Server {
    listener: TcpListener,
    config: Arc<Config>,
    root_path: PathBuf,
}

impl Server {
    pub fn new<A: ToSocketAddrs>(addr: A, config: Config, root_path: PathBuf) -> Self {
        Server {
            listener: TcpListener::bind(addr).unwrap(),
            config: Arc::new(config),
            root_path,
        }
    }

    pub fn run(self) -> io::Result<()> {
        for stream in self.listener.incoming() {
            let stream = stream?;

            let config = self.config.clone();
            let root_path = self.root_path.clone();

            thread::spawn(move || Self::handle_connection(stream, config, root_path));
        }

        Ok(())
    }

    fn handle_connection(
        stream: TcpStream,
        config: Arc<Config>,
        root_path: PathBuf,
    ) -> io::Result<()> {
        let mut connection = Connection::new(stream, root_path, config)?;

        connection.command_loop()?;

        Ok(())
    }
}
