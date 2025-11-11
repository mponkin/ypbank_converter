#![deny(unreachable_pub)]
use std::{fmt::Display, str::FromStr};

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
    Binary,
    Csv,
    Text,
}

impl FileFormat {
    /// Get reader corresponding to format
    pub fn get_format_reader(&self) -> Box<dyn RecordReader> {
        match self {
            FileFormat::Binary => Box::new(BinRecordReader::new()),
            FileFormat::Csv => Box::new(CsvRecordReader::new()),
            FileFormat::Text => Box::new(TextRecordReader::new()),
        }
    }

    /// Get writer corresponding to format
    pub fn get_format_writer(&self) -> Box<dyn RecordWriter> {
        match self {
            FileFormat::Binary => Box::new(BinRecordWriter::new()),
            FileFormat::Csv => Box::new(CsvRecordWriter::new()),
            FileFormat::Text => Box::new(TextRecordWriter::new()),
        }
    }
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
    pub id: u64,
    record_type: RecordType,
    amount: u64,
    timestamp: u64,
    status: RecordStatus,
    description: String,
}

impl Record {
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

#[derive(Debug, PartialEq, Eq)]
pub enum RecordType {
    Deposit { to_user_id: u64 },
    Withdrawal { from_user_id: u64 },
    Transfer { from_user_id: u64, to_user_id: u64 },
}

#[derive(Debug, PartialEq, Eq)]
pub enum RecordStatus {
    Success,
    Failure,
    Pending,
}

pub trait RecordReader {
    /// Read all records from given reader
    fn read_all(&self, r: &mut dyn std::io::Read) -> Result<Vec<Record>, YpbankError>;
}

pub trait RecordWriter {
    /// Write all records to privided writer
    fn write_all(&self, w: &mut dyn std::io::Write, records: &[Record]) -> Result<(), YpbankError>;
}
