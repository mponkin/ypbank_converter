//! Library for reading, parsing and writing YPBank transaction records

#![deny(unreachable_pub)]
#![warn(missing_docs)]

use std::{
    fmt::Display,
    io::{Read, Write},
    str::FromStr,
};

use crate::{
    bin_format::{BinRecordReader, BinRecordWriter},
    csv_format::{CsvRecordReader, CsvRecordWriter},
    error::YpbankError,
    txt_format::{TextRecordReader, TextRecordWriter},
};

mod bin_format;
mod csv_format;
pub mod error;
mod txt_format;

/// Available file formats
#[derive(Debug, Clone)]
pub enum FileFormat {
    /// Binary format for effective storage usage
    Binary,

    /// CSV table format
    Csv,

    /// Human-readable text format
    Text,
}

impl Display for FileFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                FileFormat::Binary => "Binary",
                FileFormat::Csv => "Csv",
                FileFormat::Text => "Text",
            }
        )
    }
}

impl FromStr for FileFormat {
    type Err = YpbankError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "binary" => Ok(FileFormat::Binary),
            "csv" => Ok(FileFormat::Csv),
            "text" => Ok(FileFormat::Text),
            _ => Err(YpbankError::UnknownFormat(s.to_string())),
        }
    }
}

/// Format-independent Record structure
#[derive(Debug, PartialEq, Eq)]
pub struct Record {
    /// Id of record
    pub id: u64,
    record_type: RecordType,
    amount: u64,
    timestamp: u64,
    status: RecordStatus,
    description: String,
}

impl Record {
    /// Create new record
    pub fn new(
        id: u64,
        record_type: RecordType,
        amount: u64,
        timestamp: u64,
        status: RecordStatus,
        description: String,
    ) -> Self {
        Self {
            id,
            record_type,
            amount,
            timestamp,
            status,
            description,
        }
    }
}

/// Supported record types
#[derive(Debug, PartialEq, Eq)]
pub enum RecordType {
    /// Deposit money to some account
    Deposit {
        /// Id of user account for money deposit
        to_user_id: u64,
    },
    /// Withdraw money from some account
    Withdrawal {
        /// Id of user account for money withdraw
        from_user_id: u64,
    },
    /// Transfer money between users
    Transfer {
        /// Id of user account for money withdraw
        from_user_id: u64,
        /// Id of user account for money deposit
        to_user_id: u64,
    },
}

/// Status of record
#[derive(Debug, PartialEq, Eq)]
pub enum RecordStatus {
    /// Successfull operation
    Success,
    /// Failed operation
    Failure,
    /// Pending operation
    Pending,
}

/// Trait for reading some format to unified records list
trait RecordReader {
    /// Read all records from given reader
    fn read_all<R: Read>(&self, r: &mut R) -> Result<Vec<Record>, YpbankError>;
}

/// Trait for writing some format from unified records list
trait RecordWriter {
    /// Write all records to privided writer
    fn write_all<W: Write>(&self, w: &mut W, records: &[Record]) -> Result<(), YpbankError>;
}

/// Read all records in given format from reader
pub fn read_all_records<R: Read>(
    reader: &mut R,
    input_format: FileFormat,
) -> Result<Vec<Record>, YpbankError> {
    match input_format {
        FileFormat::Binary => BinRecordReader::new().read_all(reader),
        FileFormat::Csv => CsvRecordReader::new().read_all(reader),
        FileFormat::Text => TextRecordReader::new().read_all(reader),
    }
}

/// Write all records in given format to writer
pub fn write_all_records<W: Write>(
    writer: &mut W,
    output_format: FileFormat,
    records: &[Record],
) -> Result<(), YpbankError> {
    match output_format {
        FileFormat::Binary => BinRecordWriter::new().write_all(writer, records),
        FileFormat::Csv => CsvRecordWriter::new().write_all(writer, records),
        FileFormat::Text => TextRecordWriter::new().write_all(writer, records),
    }
}
