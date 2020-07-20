pub enum Command {
    /// The argument field is a Telnet string identifying the user.
    /// The user identification is that which is required by the
    /// server for access to its file system.  This command will
    /// normally be the first command transmitted by the user after
    /// the control connections are made (some servers may require
    /// this).  Additional identification information in the form of
    /// a password and/or an account command may also be required by
    /// some servers.  Servers may allow a new USER command to be
    /// entered at any point in order to change the access control
    /// and/or accounting information.  This has the effect of
    /// flushing any user, password, and account information already
    /// supplied and beginning the login sequence again.  All
    /// transfer parameters are unchanged and any file transfer in
    /// progress is completed under the old access control
    /// parameters.
    UserName,

    /// The argument field is a Telnet string specifying the user's
    /// password.  This command must be immediately preceded by the
    /// user name command, and, for some sites, completes the user's
    /// identification for access control.  Since password
    /// information is quite sensitive, it is desirable in general
    /// to "mask" it or suppress typeout.  It appears that the
    /// server has no foolproof way to achieve this.  It is
    /// therefore the responsibility of the user-FTP process to hide
    /// the sensitive password information.
    Password,

    /// The argument field is a Telnet string identifying the user's
    /// account.  The command is not necessarily related to the USER
    /// command, as some sites may require an account for login and
    /// others only for specific access, such as storing files.  In
    /// the latter case the command may arrive at any time.
    ///
    /// There are reply codes to differentiate these cases for the
    /// automation: when account information is required for login,
    /// the response to a successful PASSword command is reply code
    /// 332.  On the other hand, if account information is NOT
    /// required for login, the reply to a successful PASSword
    /// command is 230; and if the account information is needed for
    /// a command issued later in the dialogue, the server should
    /// return a 332 or 532 reply depending on whether it stores
    /// (pending receipt of the ACCounT command) or discards the
    /// command, respectively.
    Account,

    /// This command allows the user to work with a different
    /// directory or dataset for file storage or retrieval without
    /// altering his login or accounting information.  Transfer
    /// parameters are similarly unchanged.  The argument is a
    /// pathname specifying a directory or other system dependent
    /// file group designator.
    ChangeWorkingDirectory,

    /// This command is a special case of CWD, and is included to
    /// simplify the implementation of programs for transferring
    /// directory trees between operating systems having different
    /// syntaxes for naming the parent directory.  The reply codes
    /// shall be identical to the reply codes of CWD.  See
    /// Appendix II for further details.
    ChangeToParentDirectory,

    /// This command allows the user to mount a different file
    /// system data structure without altering his login or
    /// accounting information.  Transfer parameters are similarly
    /// unchanged.  The argument is a pathname specifying a
    /// directory or other system dependent file group designator.
    StructureMount,

    /// This command terminates a USER, flushing all I/O and account
    /// information, except to allow any transfer in progress to be
    /// completed.  All parameters are reset to the default settings
    /// and the control connection is left open.  This is identical
    /// to the state in which a user finds himself immediately after
    /// the control connection is opened.  A USER command may be
    /// expected to follow.
    Reinitialize,

    /// This command terminates a USER and if file transfer is not
    /// in progress, the server closes the control connection.  If
    /// file transfer is in progress, the connection will remain
    /// open for result response and the server will then close it.
    /// If the user-process is transferring files for several USERs
    /// but does not wish to close and then reopen connections for
    /// each, then the REIN command should be used instead of QUIT.
    ///
    /// An unexpected close on the control connection will cause the
    /// server to take the effective action of an abort (ABOR) and a
    /// logout (QUIT).
    Logout,
}
