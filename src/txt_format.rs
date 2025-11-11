use std::collections::HashMap;
use std::io::{BufRead, BufReader};

use crate::error::YpbankError;
use crate::{Record, RecordReader, RecordStatus, RecordType, RecordWriter};

pub struct TextRecordReader;

impl TextRecordReader {
    pub fn new() -> Self {
        Self
    }
}

impl RecordReader for TextRecordReader {
    fn read_all(&self, r: &mut dyn std::io::Read) -> Result<Vec<Record>, YpbankError> {
        let reader = BufReader::new(r);

        const DELIMITER: &str = ": ";
        let mut map = HashMap::new();
        let mut records = vec![];
        for line in reader.lines() {
            match line {
                Ok(line) => {
                    if line.is_empty() {
                        let fields = map.clone();
                        map.clear();
                        let text_record = TextRecord { fields };
                        records.push(text_record.try_into()?);
                        continue;
                    }
                    if line.starts_with("#") {
                        continue;
                    }
                    match line.split_once(DELIMITER) {
                        Some((key, value)) => {
                            if map.contains_key(key) {
                                return Err(YpbankError::TextDuplicateField(key.to_string()));
                            }

                            map.insert(key.to_string(), value.to_string());
                        }
                        None => {
                            return Err(YpbankError::TextUnableToParse(line));
                        }
                    }
                }
                Err(e) => return Err(YpbankError::TextReadError(e.to_string())),
            }
        }

        if !map.is_empty() {
            let text_record = TextRecord { fields: map };
            records.push(text_record.try_into()?);
        }

        Ok(records)
    }
}

pub struct TextRecordWriter;

impl TextRecordWriter {
    pub fn new() -> Self {
        Self
    }
}

impl RecordWriter for TextRecordWriter {
    fn write_all(&self, w: &mut dyn std::io::Write, records: &[Record]) -> Result<(), YpbankError> {
        for record in records {
            let text_record = TextRecord::from(record);

            for (k, v) in text_record.fields {
                if let Err(e) = w.write(format!("{k}: {v}\n").as_bytes()) {
                    return Err(YpbankError::WriteError(e.to_string()));
                }
            }
            if let Err(e) = w.write("\n".as_bytes()) {
                return Err(YpbankError::WriteError(e.to_string()));
            }
        }

        Ok(())
    }
}

struct TextRecord {
    fields: HashMap<String, String>,
}

impl TryInto<Record> for TextRecord {
    type Error = YpbankError;

    fn try_into(self) -> Result<Record, Self::Error> {
        fn field_value(map: &HashMap<String, String>, key: &str) -> Result<String, YpbankError> {
            map.get(key)
                .ok_or_else(|| YpbankError::TextFieldNotFound(key.to_string()))
                .cloned()
        }

        let id = field_value(&self.fields, "TX_ID").and_then(|v| {
            v.parse::<u64>()
                .map_err(|_| YpbankError::TextUnexpectedFieldValue("TX_ID".to_string(), v))
        })?;

        let from_user_id = field_value(&self.fields, "FROM_USER_ID").and_then(|v| {
            v.parse::<u64>()
                .map_err(|_| YpbankError::TextUnexpectedFieldValue("FROM_USER_ID".to_string(), v))
        })?;
        let to_user_id = field_value(&self.fields, "TO_USER_ID").and_then(|v| {
            v.parse::<u64>()
                .map_err(|_| YpbankError::TextUnexpectedFieldValue("TO_USER_ID".to_string(), v))
        })?;
        let record_type = match field_value(&self.fields, "TX_TYPE")?.as_str() {
            "DEPOSIT" => Ok(RecordType::Deposit { to_user_id }),
            "WITHDRAWAL" => Ok(RecordType::Withdrawal { from_user_id }),
            "TRANSFER" => Ok(RecordType::Transfer {
                from_user_id,
                to_user_id,
            }),
            other => Err(YpbankError::TextUnexpectedFieldValue(
                "TX_TYPE".to_string(),
                other.to_string(),
            )),
        }?;
        let amount = field_value(&self.fields, "AMOUNT").and_then(|v| {
            v.parse::<u64>()
                .map_err(|_| YpbankError::TextUnexpectedFieldValue("AMOUNT".to_string(), v))
        })?;
        let timestamp = field_value(&self.fields, "TIMESTAMP").and_then(|v| {
            v.parse::<u64>()
                .map_err(|_| YpbankError::TextUnexpectedFieldValue("TIMESTAMP".to_string(), v))
        })?;
        let status = match field_value(&self.fields, "STATUS")?.as_str() {
            "SUCCESS" => Ok(RecordStatus::Success),
            "PENDING" => Ok(RecordStatus::Pending),
            "FAILURE" => Ok(RecordStatus::Failure),
            other => Err(YpbankError::TextUnexpectedFieldValue(
                "STATUS".to_string(),
                other.to_string(),
            )),
        }?;
        let description = field_value(&self.fields, "DESCRIPTION").and_then(|v| {
            if v.len() >= 2 && v.starts_with("\"") && v.ends_with("\"") {
                let slice = &v[1..v.len() - 1];
                Ok(slice.to_string())
            } else {
                Err(YpbankError::TextUnexpectedFieldValue(
                    "DESCRIPTION".to_string(),
                    v,
                ))
            }
        })?;
        Ok(Record::new(
            id,
            record_type,
            amount,
            timestamp,
            status,
            description,
        ))
    }
}

impl From<&Record> for TextRecord {
    fn from(value: &Record) -> Self {
        let (tx_type, from_user_id, to_user_id) = match value.record_type {
            RecordType::Deposit { to_user_id } => ("DEPOSIT", 0, to_user_id),
            RecordType::Withdrawal { from_user_id } => ("WITHDRAWAL", from_user_id, 0),
            RecordType::Transfer {
                from_user_id,
                to_user_id,
            } => ("TRANSFER", from_user_id, to_user_id),
        };
        let status = match value.status {
            RecordStatus::Success => "SUCCESS",
            RecordStatus::Failure => "FAILURE",
            RecordStatus::Pending => "PENDING",
        };

        Self {
            fields: HashMap::from_iter(
                vec![
                    ("TX_ID", value.id.to_string()),
                    ("TX_TYPE", tx_type.to_string()),
                    ("FROM_USER_ID", from_user_id.to_string()),
                    ("TO_USER_ID", to_user_id.to_string()),
                    ("AMOUNT", value.amount.to_string()),
                    ("TIMESTAMP", value.timestamp.to_string()),
                    ("STATUS", status.to_string()),
                    ("DESCRIPTION", format!("\"{}\"", value.description)),
                ]
                .into_iter()
                .map(|(k, v)| (k.to_string(), v)),
            ),
        }
    }
}

mod tests {
    #![allow(unused_imports)]
    use std::io::Cursor;

    use super::*;

    #[test]
    fn test_text_deposit_success() {
        let deposit = TextRecord {
            fields: HashMap::from_iter(
                vec![
                    ("TX_ID", "1234567890123456"),
                    ("TX_TYPE", "DEPOSIT"),
                    ("FROM_USER_ID", "0"),
                    ("TO_USER_ID", "9876543210987654"),
                    ("AMOUNT", "10000"),
                    ("TIMESTAMP", "1633036800000"),
                    ("STATUS", "SUCCESS"),
                    ("DESCRIPTION", "\"Terminal deposit\""),
                ]
                .into_iter()
                .map(|(k, v)| (k.to_string(), v.to_string())),
            ),
        };
        let result: Result<Record, YpbankError> = deposit.try_into();
        assert_eq!(
            result,
            Ok(Record::new(
                1234567890123456,
                RecordType::Deposit {
                    to_user_id: 9876543210987654
                },
                10000,
                1633036800000,
                RecordStatus::Success,
                "Terminal deposit".to_string(),
            ))
        )
    }

    #[test]
    fn test_text_transfer_success() {
        let deposit = TextRecord {
            fields: HashMap::from_iter(
                vec![
                    ("TX_ID", "2312321321321321"),
                    ("TX_TYPE", "TRANSFER"),
                    ("FROM_USER_ID", "1231231231231231"),
                    ("TO_USER_ID", "9876543210987654"),
                    ("AMOUNT", "1000"),
                    ("TIMESTAMP", "1633056800000"),
                    ("STATUS", "FAILURE"),
                    ("DESCRIPTION", "\"User transfer\""),
                ]
                .into_iter()
                .map(|(k, v)| (k.to_string(), v.to_string())),
            ),
        };
        let result: Result<Record, YpbankError> = deposit.try_into();
        assert_eq!(
            result,
            Ok(Record::new(
                2312321321321321,
                RecordType::Transfer {
                    to_user_id: 9876543210987654,
                    from_user_id: 1231231231231231,
                },
                1000,
                1633056800000,
                RecordStatus::Failure,
                "User transfer".to_string(),
            ))
        )
    }

    #[test]
    fn test_text_withdrawal_success() {
        let deposit = TextRecord {
            fields: HashMap::from_iter(
                vec![
                    ("TX_ID", "3213213213213213"),
                    ("TX_TYPE", "WITHDRAWAL"),
                    ("FROM_USER_ID", "9876543210987654"),
                    ("TO_USER_ID", "0"),
                    ("AMOUNT", "100"),
                    ("TIMESTAMP", "1633066800000"),
                    ("STATUS", "SUCCESS"),
                    ("DESCRIPTION", "\"User withdrawal\""),
                ]
                .into_iter()
                .map(|(k, v)| (k.to_string(), v.to_string())),
            ),
        };
        let result: Result<Record, YpbankError> = deposit.try_into();
        assert_eq!(
            result,
            Ok(Record::new(
                3213213213213213,
                RecordType::Withdrawal {
                    from_user_id: 9876543210987654,
                },
                100,
                1633066800000,
                RecordStatus::Success,
                "User withdrawal".to_string(),
            ))
        )
    }

    #[test]
    fn test_text_missing_field_error() {
        let deposit = TextRecord {
            fields: HashMap::from_iter(
                vec![
                    ("TX_TYPE", "WITHDRAWAL"),
                    ("FROM_USER_ID", "9876543210987654"),
                    ("TO_USER_ID", "0"),
                    ("AMOUNT", "100"),
                    ("TIMESTAMP", "1633066800000"),
                    ("STATUS", "SUCCESS"),
                    ("DESCRIPTION", "\"User withdrawal\""),
                ]
                .into_iter()
                .map(|(k, v)| (k.to_string(), v.to_string())),
            ),
        };
        let result: Result<Record, YpbankError> = deposit.try_into();
        assert_eq!(
            result,
            Err(YpbankError::TextFieldNotFound("TX_ID".to_string()))
        )
    }

    #[test]
    fn test_text_field_value_error() {
        let deposit = TextRecord {
            fields: HashMap::from_iter(
                vec![
                    ("TX_ID", "incorrect"),
                    ("TX_TYPE", "WITHDRAWAL"),
                    ("FROM_USER_ID", "9876543210987654"),
                    ("TO_USER_ID", "0"),
                    ("AMOUNT", "100"),
                    ("TIMESTAMP", "1633066800000"),
                    ("STATUS", "SUCCESS"),
                    ("DESCRIPTION", "\"User withdrawal\""),
                ]
                .into_iter()
                .map(|(k, v)| (k.to_string(), v.to_string())),
            ),
        };
        let result: Result<Record, YpbankError> = deposit.try_into();
        assert_eq!(
            result,
            Err(YpbankError::TextUnexpectedFieldValue(
                "TX_ID".to_string(),
                "incorrect".to_string()
            ))
        )
    }

    #[test]
    fn test_read_all_success() {
        let text_data = r#"# Record 1 (Deposit)
TX_ID: 1234567890123456
TX_TYPE: DEPOSIT
FROM_USER_ID: 0
TO_USER_ID: 9876543210987654
AMOUNT: 10000
TIMESTAMP: 1633036800000
STATUS: SUCCESS
DESCRIPTION: "Terminal deposit"

# Record 2 (Transfer)
TX_ID: 2312321321321321
TIMESTAMP: 1633056800000
STATUS: FAILURE
TX_TYPE: TRANSFER
FROM_USER_ID: 1231231231231231
TO_USER_ID: 9876543210987654
AMOUNT: 1000
DESCRIPTION: "User transfer"

# Record 3 (Withdrawal)
TX_ID: 3213213213213213
AMOUNT: 100
TX_TYPE: WITHDRAWAL
FROM_USER_ID: 9876543210987654
TO_USER_ID: 0
TIMESTAMP: 1633066800000
STATUS: SUCCESS
DESCRIPTION: "User withdrawal""#;

        let mut cursor = Cursor::new(text_data);

        let reader = TextRecordReader::new();

        let records = reader.read_all(&mut cursor);

        assert_eq!(
            records,
            Ok(vec![
                Record::new(
                    1234567890123456,
                    RecordType::Deposit {
                        to_user_id: 9876543210987654
                    },
                    10000,
                    1633036800000,
                    RecordStatus::Success,
                    "Terminal deposit".to_string(),
                ),
                Record::new(
                    2312321321321321,
                    RecordType::Transfer {
                        to_user_id: 9876543210987654,
                        from_user_id: 1231231231231231,
                    },
                    1000,
                    1633056800000,
                    RecordStatus::Failure,
                    "User transfer".to_string(),
                ),
                Record::new(
                    3213213213213213,
                    RecordType::Withdrawal {
                        from_user_id: 9876543210987654,
                    },
                    100,
                    1633066800000,
                    RecordStatus::Success,
                    "User withdrawal".to_string(),
                )
            ])
        )
    }

    #[test]
    fn test_read_all_duplicate_field() {
        let text_data = r#"# Record 1 (Deposit)
TX_ID: 1234567890123456
TX_ID: 1234567890123456
TX_TYPE: DEPOSIT
FROM_USER_ID: 0
TO_USER_ID: 9876543210987654
AMOUNT: 10000
TIMESTAMP: 1633036800000
STATUS: SUCCESS
DESCRIPTION: "Terminal deposit""#;

        let mut cursor = Cursor::new(text_data);

        let reader = TextRecordReader::new();

        let records = reader.read_all(&mut cursor);

        assert_eq!(
            records,
            Err(YpbankError::TextDuplicateField("TX_ID".to_string()))
        )
    }
}
