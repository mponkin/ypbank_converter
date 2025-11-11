use std::{
    fs::File,
    io::{self, BufReader, BufWriter},
    path::PathBuf,
};

use clap::{Parser, arg};
use ypbank_converter::{FileFormat, error::YpbankError};

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

    read_and_convert(
        &mut file_reader,
        args.input_format,
        &mut stdout_writer,
        args.output_format,
    )
}

fn read_and_convert(
    reader: &mut dyn std::io::Read,
    input_format: FileFormat,
    writer: &mut dyn std::io::Write,
    output_format: FileFormat,
) -> Result<(), YpbankError> {
    let input_reader = input_format.get_format_reader();
    let records = input_reader.read_all(reader)?;
    let output_writer = output_format.get_format_writer();
    output_writer.write_all(writer, &records)
}
