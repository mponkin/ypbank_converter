use std::{error::Error, fmt::Display};

#[derive(Debug, PartialEq, Eq)]
pub enum YpbankError {
    FileNotFound { file: String },
    UnknownFormat(String),
    CsvParseError(String),
    CsvUnexpectedValue(String),
    TextFieldNotFound(String),
    TextUnexpectedFieldValue(String, String),
    TextUnableToParse(String),
    TextDuplicateField(String),
    TextReadError(String),
    BinaryUnexpectedValue,
    BinaryReadError(String),
    BinaryDescriptionTooLong,
    BinaryRecordTooShort,
    WriteError(String),
}

impl Display for YpbankError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            YpbankError::FileNotFound { file } => write!(f, "Unable to open '{file}'"),
            YpbankError::UnknownFormat(format) => write!(
                f,
                "Unknown file format '{format}', available options are 'binary', 'csv' and 'text'"
            ),
            YpbankError::CsvParseError(error) => write!(f, "Parsing CSV error: {error}"),
            YpbankError::CsvUnexpectedValue(value) => write!(f, "Csv unexpected value: {value}"),
            YpbankError::TextFieldNotFound(field) => write!(f, "Text field not found: {field}"),
            YpbankError::TextUnexpectedFieldValue(field, value) => {
                write!(f, "Text field {field} unexpected value: {value}")
            }
            YpbankError::TextUnableToParse(line) => write!(f, "Unable to parse txt line: {line}"),
            YpbankError::TextDuplicateField(field) => {
                write!(f, "Text duplicate field found: {field}")
            }
            YpbankError::TextReadError(reason) => {
                write!(f, "Error while reading text file: {reason}")
            }
            YpbankError::BinaryUnexpectedValue => {
                write!(f, "Unable to read binary format, unexpected value")
            }
            YpbankError::BinaryReadError(err) => {
                write!(f, "Unable to read binary format, read error: {err}")
            }
            YpbankError::BinaryDescriptionTooLong => {
                write!(f, "Binary description length exceeds record length")
            }
            YpbankError::BinaryRecordTooShort => {
                write!(
                    f,
                    "Binary record is too shord and does not contain all required fields"
                )
            }
            YpbankError::WriteError(reason) => {
                write!(f, "Unable to write output: {reason}")
            }
        }
    }
}

impl Error for YpbankError {}

impl From<csv::Error> for YpbankError {
    fn from(value: csv::Error) -> Self {
        YpbankError::CsvParseError(value.to_string())
    }
}
