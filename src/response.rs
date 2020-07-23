use std::fmt;

pub struct Message(String);

impl fmt::Display for Message {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
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

impl Code {
    pub fn from_bytes(s: [u8; 3]) -> Option<Self> {
        Some(match s {
            [b'1', b'1', b'0'] => Code::RestartMarkerReply,
            [b'1', b'2', b'0'] => Code::ServiceReadyInNNNMinutes,
            [b'1', b'2', b'5'] => Code::DataConnectionAlreadyOpen,
            [b'1', b'5', b'0'] => Code::FileStatusOk,
            [b'2', b'0', b'0'] => Code::Ok,
            [b'2', b'0', b'2'] => Code::CommandNotImplementedSuperfluousAtThisSite,
            [b'2', b'1', b'1'] => Code::SystemStatus,
            [b'2', b'1', b'2'] => Code::DirectoryStatus,
            [b'2', b'1', b'3'] => Code::FileStatus,
            [b'2', b'1', b'4'] => Code::HelpMessage,
            [b'2', b'1', b'5'] => Code::SystemTypeName,
            [b'2', b'2', b'0'] => Code::ServiceReadyForNewUser,
            [b'2', b'2', b'1'] => Code::ServiceClosing,
            [b'2', b'2', b'5'] => Code::DataConnectionOpen,
            [b'2', b'2', b'6'] => Code::ClosingDataConnection,
            [b'2', b'2', b'7'] => Code::EnteringPassiveMode,
            [b'2', b'3', b'0'] => Code::UserLoggedIn,
            [b'2', b'5', b'0'] => Code::RequestedFileActionComplete,
            [b'2', b'5', b'7'] => Code::PathNameCreated,
            [b'3', b'3', b'1'] => Code::UserNameOkPasswordNeeded,
            [b'3', b'3', b'2'] => Code::NeedAccountForLogin,
            [b'3', b'5', b'0'] => Code::RequestPendingMoreInformation,
            [b'4', b'2', b'1'] => Code::ServiceNotAvailable,
            [b'4', b'2', b'5'] => Code::CannotOpenDataConnection,
            [b'4', b'2', b'6'] => Code::ConnectionClosed,
            [b'4', b'5', b'0'] => Code::ActionNotTaken,
            [b'4', b'5', b'1'] => Code::ActionAborted,
            [b'4', b'5', b'2'] => Code::ActionNotTakenInsufficientStorage,
            [b'5', b'0', b'0'] => Code::CommandUnrecognized,
            [b'5', b'0', b'1'] => Code::InvalidParametersOrArguments,
            [b'5', b'0', b'2'] => Code::CommandNotImplemented,
            [b'5', b'0', b'3'] => Code::BadSequenceOfCommands,
            [b'5', b'0', b'4'] => Code::CommandNotImplementedForThatParameter,
            [b'5', b'3', b'0'] => Code::NotLoggedIn,
            [b'5', b'3', b'2'] => Code::NeedAccountForStoringFiles,
            [b'5', b'5', b'0'] => Code::FileUnavailable,
            [b'5', b'5', b'1'] => Code::PageTypeUnknown,
            [b'5', b'5', b'2'] => Code::ExceededStorageAllocation,
            [b'5', b'5', b'3'] => Code::FileNameNotAllowed,
            _ => return None,
        })
    }
}

impl fmt::Display for Code {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", *self as u16)
    }
}
