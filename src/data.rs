/// Data representations are handled in FTP by a user specifying a
/// representation type.  This type may implicitly (as in ASCII or
/// EBCDIC) or explicitly (as in Local byte) define a byte size for
/// interpretation which is referred to as the "logical byte size."
/// Note that this has nothing to do with the byte size used for
/// transmission over the data connection, called the "transfer
/// byte size", and the two should not be confused.  For example,
/// NVT-ASCII has a logical byte size of 8 bits.  If the type is
/// Local byte, then the TYPE command has an obligatory second
/// parameter specifying the logical byte size.  The transfer byte
/// size is always 8 bits.
pub enum DataType {
    /// This is the default type and must be accepted by all FTP
    /// implementations.  It is intended primarily for the transfer
    /// of text files, except when both hosts would find the EBCDIC
    /// type more convenient.
    ///
    /// The sender converts the data from an internal character
    /// representation to the standard 8-bit NVT-ASCII
    /// representation (see the Telnet specification).  The receiver
    /// will convert the data from the standard form to his own
    /// internal form.
    ///
    /// In accordance with the NVT standard, the <CRLF> sequence
    /// should be used where necessary to denote the end of a line
    /// of text.  (See the discussion of file structure at the end
    /// of the Section on Data Representation and Storage.)
    ///
    /// Using the standard NVT-ASCII representation means that data
    /// must be interpreted as 8-bit bytes.
    Ascii,

    /// This type is intended for efficient transfer between hosts
    /// which use EBCDIC for their internal character
    /// representation.
    ///
    /// For transmission, the data are represented as 8-bit EBCDIC
    /// characters.  The character code is the only difference
    /// between the functional specifications of EBCDIC and ASCII
    /// types.
    ///
    /// End-of-line (as opposed to end-of-record--see the discussion
    /// of structure) will probably be rarely used with EBCDIC type
    /// for purposes of denoting structure, but where it is
    /// necessary the <NL> character should be used.
    Ebcdic,

    /// The data are sent as contiguous bits which, for transfer,
    /// are packed into the 8-bit transfer bytes.  The receiving
    /// site must store the data as contiguous bits.  The structure
    /// of the storage system might necessitate the padding of the
    /// file (or of each record, for a record-structured file) to
    /// some convenient boundary (byte, word or block).  This
    /// padding, which must be all zeros, may occur only at the end
    /// of the file (or at the end of each record) and there must be
    /// a way of identifying the padding bits so that they may be
    /// stripped off if the file is retrieved.  The padding
    /// transformation should be well publicized to enable a user to
    /// process a file at the storage site.
    ///
    /// Image type is intended for the efficient storage and
    /// retrieval of files and for the transfer of binary data.  It
    /// is recommended that this type be accepted by all FTP
    /// implementations.
    Image,

    /// The data is transferred in logical bytes of the size
    /// specified by the obligatory second parameter, Byte size.
    /// The value of Byte size must be a decimal integer; there is
    /// no default value.  The logical byte size is not necessarily
    /// the same as the transfer byte size.  If there is a
    /// difference in byte sizes, then the logical bytes should be
    /// packed contiguously, disregarding transfer byte boundaries
    /// and with any necessary padding at the end.
    ///
    /// When the data reaches the receiving host, it will be
    /// transformed in a manner dependent on the logical byte size
    /// and the particular host.  This transformation must be
    /// invertible (i.e., an identical file can be retrieved if the
    /// same parameters are used) and should be well publicized by
    /// the FTP implementors.
    ///
    /// For example, a user sending 36-bit floating-point numbers to
    /// a host with a 32-bit word could send that data as Local byte
    /// with a logical byte size of 36.  The receiving host would
    /// then be expected to store the logical bytes so that they
    /// could be easily manipulated; in this example putting the
    /// 36-bit logical bytes into 64-bit double words should
    /// suffice.
    ///
    /// In another example, a pair of hosts with a 36-bit word size
    /// may send data to one another in words by using TYPE L 36.
    /// The data would be sent in the 8-bit transmission bytes
    /// packed so that 9 transmission bytes carried two host words.
    LocalType,

    /// The types ASCII and EBCDIC also take a second (optional)
    /// parameter; this is to indicate what kind of vertical format
    /// control, if any, is associated with a file.  The following
    /// data representation types are defined in FTP:
    ///
    /// A character file may be transferred to a host for one of
    /// three purposes: for printing, for storage and later
    /// retrieval, or for processing.  If a file is sent for
    /// printing, the receiving host must know how the vertical
    /// format control is represented.  In the second case, it must
    /// be possible to store a file at a host and then retrieve it
    /// later in exactly the same form.  Finally, it should be
    /// possible to move a file from one host to another and process
    /// the file at the second host without undue trouble.  A single
    /// ASCII or EBCDIC format does not satisfy all these
    /// conditions.  Therefore, these types have a second parameter
    /// specifying one of the following three formats:
    FormatControl,
}

pub enum DataStructure {
    /// File structure is the default to be assumed if the STRUcture
    /// command has not been used.
    ///
    /// In file-structure there is no internal structure and the
    /// file is considered to be a continuous sequence of data
    File(Vec<u8>),

    // Record structures must be accepted for "text" files (i.e.,
    // files with TYPE ASCII or EBCDIC) by all FTP implementations.
    //
    // In record-structure the file is made up of sequential
    // records.
    Record,

    /// To transmit files that are discontinuous, FTP defines a page
    /// structure.  Files of this type are sometimes known as
    /// "random access files" or even as "holey files".  In these
    /// files there is sometimes other information associated with
    /// the file as a whole (e.g., a file descriptor), or with a
    /// section of the file (e.g., page access controls), or both.
    /// In FTP, the sections of the file are called pages.
    Page {
        /// The number of logical bytes in the page header
        /// including this byte.  The minimum header length is 4.
        header_length: usize,

        /// The logical page number of this section of the file.
        /// This is not the transmission sequence number of this
        /// page, but the index used to identify this page of the
        /// file.
        page_index: usize,

        /// The number of logical bytes in the page data.  The
        /// minimum data length is 0.
        data_length: usize,
    },
}

#[repr(u8)]
pub enum PageType {
    /// This is used to indicate the end of a paged
    /// structured transmission.  The header length must
    /// be 4, and the data length must be 0.
    Last = 0,

    /// This is the normal type for simple paged files
    /// with no page level associated control
    /// information.  The header length must be 4.
    Simple = 1,

    /// This type is used to transmit the descriptive
    /// information for the file as a whole.
    Descriptor = 2,

    /// This type includes an additional header field
    /// for paged files with page level access control
    /// information.  The header length must be 5.
    AccessControlled = 3,
}

/// Number of bits long a byte is (for now we assume every byte is 8 bits)
pub struct LogicalByteLength(u8);