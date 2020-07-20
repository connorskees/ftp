use std::fmt;

pub struct Message(String);

impl fmt::Display for Message {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

#[derive(Debug, Copy, Clone)]
#[repr(u16)]
pub enum Code {
    RestartMarkerReply = 110,
    ServiceReadyInNNNMinutes = 120,
    DataConnectionAlreadyOpen = 125,
    FileStatusOk = 150,
    Ok = 200,
    CommandNotImplementedSuperfluousAtThisSite = 202,
    SystemStatus = 211,
    DirectoryStatus = 212,
    FileStatus = 213,
    HelpMessage = 214,
    SystemTypeName = 215,
    ServiceReadyForNewUser = 220,
    ServiceClosing = 221,
    DataConnectionOpen = 225,
    ClosingDataConnection = 226,
    EnteringPassiveMode = 227,
    UserLoggedIn = 230,
    RequestedFileActionComplete = 250,
    PathNameCreated = 257,
    UserNameOkPasswordNeeded = 331,
    NeedAccountForLogin = 332,
    RequestPendingMoreInformation = 350,
    ServiceNotAvailable = 421,
    CannotOpenDataConnection = 425,
    ConnectionClosed = 426,
    ActionNotTaken = 450,
    ActionAborted = 451,
    ActionNotTakenInsufficientStorage = 452,
    CommandUnrecognized = 500,
    InvalidParametersOrArguments = 501,
    CommandNotImplemented = 502,
    BadSequenceOfCommands = 503,
    CommandNotImplementedForThatParameter = 504,
    NotLoggedIn = 530,
    NeedAccountForStoringFiles = 532,
    FileUnavailable = 550,
    PageTypeUnknown = 551,
    ExceededStorageAllocation = 552,
    FileNameNotAllowed = 553,
}

impl fmt::Display for Code {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", *self as u16)
    }
}
