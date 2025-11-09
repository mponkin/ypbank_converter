use crate::{Record, RecordReader, RecordStatus, RecordType, RecordWriter, error::YpbankError};
use serde::{Deserialize, Serialize};

pub struct CsvRecordReader;

impl CsvRecordReader {
    pub fn new() -> Self {
        Self
    }
}

impl RecordReader for CsvRecordReader {
    fn read_all(&self, r: &mut dyn std::io::Read) -> Result<Vec<Record>, YpbankError> {
        let mut rdr = csv::Reader::from_reader(r);
        rdr.deserialize::<CsvRecord>()
            .map(|res| {
                res.map_err(YpbankError::from)
                    .and_then(|csv_record| csv_record.try_into())
            })
            .collect::<Result<Vec<Record>, YpbankError>>()
    }
}

pub struct CsvRecordWriter;

impl CsvRecordWriter {
    pub fn new() -> Self {
        Self
    }
}

impl RecordWriter for CsvRecordWriter {
    fn write_all(&self, w: &mut dyn std::io::Write, records: &[Record]) -> Result<(), YpbankError> {
        let mut writer = csv::Writer::from_writer(w);

        for record in records {
            let csv_record = CsvRecord::from(record);
            if writer.serialize(csv_record).is_err() {
                return Err(YpbankError::WriteError);
            }
        }

        Ok(())
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct CsvRecord {
    #[serde(rename = "TX_ID")]
    id: u64,
    #[serde(rename = "TX_TYPE")]
    record_type: String,
    #[serde(rename = "FROM_USER_ID")]
    from_user_id: u64,
    #[serde(rename = "TO_USER_ID")]
    to_user_id: u64,
    #[serde(rename = "AMOUNT")]
    amount: u64,
    #[serde(rename = "TIMESTAMP")]
    timestamp: u64,
    #[serde(rename = "STATUS")]
    status: String,
    #[serde(rename = "DESCRIPTION")]
    description: String,
}

impl TryInto<Record> for CsvRecord {
    type Error = YpbankError;

    fn try_into(self) -> Result<Record, Self::Error> {
        let record_type = match self.record_type.as_str() {
            "DEPOSIT" => Ok(RecordType::Deposit {
                to_user_id: self.to_user_id,
            }),
            "WITHDRAWAL" => Ok(RecordType::Withdrawal {
                from_user_id: self.from_user_id,
            }),
            "TRANSFER" => Ok(RecordType::Transfer {
                from_user_id: self.from_user_id,
                to_user_id: self.to_user_id,
            }),
            other => Err(YpbankError::CsvUnexpectedValue(other.to_string())),
        }?;

        let status = match self.status.as_str() {
            "SUCCESS" => Ok(RecordStatus::Success),
            "PENDING" => Ok(RecordStatus::Pending),
            "FAILURE" => Ok(RecordStatus::Failure),
            other => Err(YpbankError::CsvUnexpectedValue(other.to_string())),
        }?;

        Ok(Record::new(
            self.id,
            record_type,
            self.amount,
            self.timestamp,
            status,
            self.description,
        ))
    }
}

impl From<&Record> for CsvRecord {
    fn from(value: &Record) -> Self {
        let (record_type, from_user_id, to_user_id) = match value.record_type {
            RecordType::Deposit { to_user_id } => ("DEPOSIT".to_string(), 0, to_user_id),
            RecordType::Withdrawal { from_user_id } => ("WITHDRAWAL".to_string(), from_user_id, 0),
            RecordType::Transfer {
                from_user_id,
                to_user_id,
            } => ("TRANSFER".to_string(), from_user_id, to_user_id),
        };
        Self {
            id: value.id,
            record_type,
            from_user_id,
            to_user_id,
            amount: value.amount,
            timestamp: value.timestamp,
            status: match value.status {
                RecordStatus::Success => "SUCCESS",
                RecordStatus::Failure => "FAILURE",
                RecordStatus::Pending => "PENDING",
            }
            .to_string(),
            description: format!("{}", value.description),
        }
    }
}

mod tests {
    #![allow(unused_imports)]
    use std::io::Cursor;

    use crate::bin_format::BinRecordWriter;

    use super::*;

    #[test]
    fn test_csv_deposit_success() {
        let deposit = CsvRecord {
            id: 1001,
            record_type: "DEPOSIT".to_string(),
            from_user_id: 0,
            to_user_id: 501,
            amount: 50000,
            timestamp: 1672531200000,
            status: "SUCCESS".to_string(),
            description: "Initial account funding".to_string(),
        };
        assert_eq!(
            deposit.try_into(),
            Ok(Record::new(
                1001,
                RecordType::Deposit { to_user_id: 501 },
                50000,
                1672531200000,
                RecordStatus::Success,
                "Initial account funding".to_string(),
            ))
        )
    }

    #[test]
    fn test_csv_transfer_success() {
        let withdrawal = CsvRecord {
            id: 1002,
            record_type: "TRANSFER".to_string(),
            from_user_id: 501,
            to_user_id: 502,
            amount: 15000,
            timestamp: 1672534800000,
            status: "FAILURE".to_string(),
            description: "Payment for services, invoice #123".to_string(),
        };
        assert_eq!(
            withdrawal.try_into(),
            Ok(Record::new(
                1002,
                RecordType::Transfer {
                    from_user_id: 501,
                    to_user_id: 502
                },
                15000,
                1672534800000,
                RecordStatus::Failure,
                "Payment for services, invoice #123".to_string(),
            ))
        )
    }

    #[test]
    fn test_csv_withdrawal_success() {
        let withdrawal = CsvRecord {
            id: 1003,
            record_type: "WITHDRAWAL".to_string(),
            from_user_id: 502,
            to_user_id: 0,
            amount: 1000,
            timestamp: 1672538400000,
            status: "PENDING".to_string(),
            description: "ATM withdrawal".to_string(),
        };
        assert_eq!(
            withdrawal.try_into(),
            Ok(Record::new(
                1003,
                RecordType::Withdrawal { from_user_id: 502 },
                1000,
                1672538400000,
                RecordStatus::Pending,
                "ATM withdrawal".to_string(),
            ))
        )
    }

    #[test]
    fn test_csv_record_type_error() {
        let withdrawal = CsvRecord {
            id: 1003,
            record_type: "something".to_string(),
            from_user_id: 502,
            to_user_id: 0,
            amount: 1000,
            timestamp: 1672538400000,
            status: "PENDING".to_string(),
            description: "ATM withdrawal".to_string(),
        };

        let result: Result<Record, YpbankError> = withdrawal.try_into();
        assert_eq!(
            result,
            Err(YpbankError::CsvUnexpectedValue("something".to_string()))
        )
    }

    #[test]
    fn test_csv_status_error() {
        let withdrawal = CsvRecord {
            id: 1003,
            record_type: "WITHDRAWAL".to_string(),
            from_user_id: 502,
            to_user_id: 0,
            amount: 1000,
            timestamp: 1672538400000,
            status: "INITIAL".to_string(),
            description: "ATM withdrawal".to_string(),
        };

        let result: Result<Record, YpbankError> = withdrawal.try_into();
        assert_eq!(
            result,
            Err(YpbankError::CsvUnexpectedValue("INITIAL".to_string()))
        )
    }

    #[test]
    fn test_read_all() {
        let csv_data = r#"TX_ID,TX_TYPE,FROM_USER_ID,TO_USER_ID,AMOUNT,TIMESTAMP,STATUS,DESCRIPTION
1001,DEPOSIT,0,501,50000,1672531200000,SUCCESS,"Initial account funding"
1002,TRANSFER,501,502,15000,1672534800000,FAILURE,"Payment for services, invoice #123"
1003,WITHDRAWAL,502,0,1000,1672538400000,PENDING,"ATM withdrawal""#;

        let mut cursor = Cursor::new(csv_data);

        let reader = CsvRecordReader::new();

        let records = reader.read_all(&mut cursor);

        assert_eq!(
            records,
            Ok(vec![
                Record::new(
                    1001,
                    RecordType::Deposit { to_user_id: 501 },
                    50000,
                    1672531200000,
                    RecordStatus::Success,
                    "Initial account funding".to_string(),
                ),
                Record::new(
                    1002,
                    RecordType::Transfer {
                        from_user_id: 501,
                        to_user_id: 502
                    },
                    15000,
                    1672534800000,
                    RecordStatus::Failure,
                    "Payment for services, invoice #123".to_string(),
                ),
                Record::new(
                    1003,
                    RecordType::Withdrawal { from_user_id: 502 },
                    1000,
                    1672538400000,
                    RecordStatus::Pending,
                    "ATM withdrawal".to_string(),
                ),
            ])
        )
    }

    #[test]
    fn test_write_all() {
        let records = vec![
            Record::new(
                1001,
                RecordType::Deposit { to_user_id: 501 },
                50000,
                1672531200000,
                RecordStatus::Success,
                "Initial account funding".to_string(),
            ),
            Record::new(
                1002,
                RecordType::Transfer {
                    from_user_id: 501,
                    to_user_id: 502,
                },
                15000,
                1672534800000,
                RecordStatus::Failure,
                "Payment for services, invoice #123".to_string(),
            ),
            Record::new(
                1003,
                RecordType::Withdrawal { from_user_id: 502 },
                1000,
                1672538400000,
                RecordStatus::Pending,
                "ATM withdrawal".to_string(),
            ),
        ];

        let buffer: Vec<u8> = Vec::new();

        let mut writer = Cursor::new(buffer);

        let bin_writer = CsvRecordWriter::new();
        bin_writer
            .write_all(&mut writer, &records)
            .expect("Should write successfully");

        assert_eq!(
            String::from_utf8(writer.into_inner()).expect("Should be correct string"),
            r#"TX_ID,TX_TYPE,FROM_USER_ID,TO_USER_ID,AMOUNT,TIMESTAMP,STATUS,DESCRIPTION
1001,DEPOSIT,0,501,50000,1672531200000,SUCCESS,Initial account funding
1002,TRANSFER,501,502,15000,1672534800000,FAILURE,"Payment for services, invoice #123"
1003,WITHDRAWAL,502,0,1000,1672538400000,PENDING,ATM withdrawal
"#
        )
    }
}
