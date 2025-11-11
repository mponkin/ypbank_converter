use std::{
    fs::File,
    io::{self, BufReader, BufWriter},
    path::PathBuf,
};

use clap::{Parser, arg};
use ypbank_converter::{FileFormat, error::YpbankError, read_all_records, write_all_records};

#[derive(Parser, Debug)]
pub struct ConverterCli {
    #[arg(long, value_name = "FILE")]
    pub input: PathBuf,

    #[arg(long, value_name = "FORMAT")]
    pub input_format: FileFormat,

    #[arg(long, value_name = "FORMAT")]
    pub output_format: FileFormat,
}

fn main() -> Result<(), YpbankError> {
    let args = ConverterCli::parse();

    let file = File::open(&args.input).map_err(|e| YpbankError::FileOpenError(e.to_string()))?;

    let mut file_reader = BufReader::new(file);

    let stdout_handle = io::stdout();
    let mut stdout_writer = BufWriter::new(stdout_handle);

    let records = read_all_records(&mut file_reader, args.input_format)?;

    write_all_records(&mut stdout_writer, args.output_format, &records)
}
