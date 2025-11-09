use std::{
    collections::{HashMap, HashSet},
    fs::File,
    io::BufReader,
};

use clap::Parser;
use ypbank_converter::{FileFormat, Record, error::YpbankError};

#[derive(Parser, Debug)]
pub struct ParserCli {
    #[arg(long, value_name = "FILE")]
    pub file1: String,

    #[arg(long, value_name = "FORMAT")]
    pub format1: FileFormat,

    #[arg(long, value_name = "FILE")]
    pub file2: String,

    #[arg(long, value_name = "FORMAT")]
    pub format2: FileFormat,
}

fn main() -> Result<(), YpbankError> {
    let args = ParserCli::parse();

    let file1 = File::open(&args.file1).map_err(|_| YpbankError::FileNotFound {
        file: args.file1.clone(),
    })?;

    let file2 = File::open(&args.file2).map_err(|_| YpbankError::FileNotFound {
        file: args.file2.clone(),
    })?;

    let reader1 = args.format1.get_format_reader();
    let reader2 = args.format2.get_format_reader();

    let records1 = records_to_map(reader1.read_all(&mut BufReader::new(file1))?);
    let records2 = records_to_map(reader2.read_all(&mut BufReader::new(file2))?);

    let keys1 = records1.keys().collect::<HashSet<_>>();
    let keys2 = records2.keys().collect::<HashSet<_>>();

    let only_in_1 = keys1.difference(&keys2).collect::<Vec<_>>();
    let only_in_2 = keys2.difference(&keys1).collect::<Vec<_>>();
    let in_both = keys1.intersection(&keys2).collect::<Vec<_>>();

    let different_values = in_both
        .into_iter()
        .filter(|id| records1.get(id) != records2.get(id))
        .collect::<Vec<_>>();

    if only_in_1.is_empty() && only_in_2.is_empty() && different_values.is_empty() {
        println!("Transactions are the same");
    } else {
        if !only_in_1.is_empty() {
            println!(
                "Transactions only in file 1: {}",
                only_in_1
                    .iter()
                    .map(|v| v.to_string())
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        }
        if !only_in_2.is_empty() {
            println!(
                "Transactions only in file 2: {}",
                only_in_2
                    .iter()
                    .map(|v| v.to_string())
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        }
        if !different_values.is_empty() {
            println!(
                "Transactions that differs in file1 and file2: {}",
                different_values
                    .iter()
                    .map(|v| v.to_string())
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        }
    }

    Ok(())
}

fn records_to_map(records: Vec<Record>) -> HashMap<u64, Record> {
    HashMap::from_iter(records.into_iter().map(|r| (r.id, r)))
}
