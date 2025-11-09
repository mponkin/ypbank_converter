use crate::{Record, RecordReader, RecordStatus, RecordType, RecordWriter, error::YpbankError};

pub struct BinRecordReader;

impl BinRecordReader {
    pub fn new() -> Self {
        Self
    }
}

#[macro_export]
macro_rules! read_n_bytes {
    ($reader:expr, $count:expr) => {{
        let mut buffer = [0u8; $count];

        match $reader.read_exact(&mut buffer) {
            Ok(_) => Ok(buffer),
            Err(_) => Err($crate::YpbankError::BinaryReadError),
        }
    }};
}

impl RecordReader for BinRecordReader {
    fn read_all(&self, r: &mut dyn std::io::Read) -> Result<Vec<Record>, YpbankError> {
        let mut bin_records: Vec<BinRecord> = vec![];
        loop {
            let header_res = read_n_bytes!(r, 4);

            match header_res {
                Ok(header) if &header == BinRecord::HEADER => (),
                Ok(_) => return Err(YpbankError::BinaryUnexpectedValue),
                Err(_) => break,
            }

            let _record_length = read_n_bytes!(r, 4)?;

            let id = read_n_bytes!(r, 8)?;
            let record_type = read_n_bytes!(r, 1)?[0];
            let from_user_id = read_n_bytes!(r, 8)?;
            let to_user_id = read_n_bytes!(r, 8)?;
            let amount = read_n_bytes!(r, 8)?;
            let timestamp = read_n_bytes!(r, 8)?;
            let status = read_n_bytes!(r, 1)?[0];
            let description_length = u32::from_be_bytes(read_n_bytes!(r, 4)?);
            let mut description = vec![0u8; description_length as usize];
            if r.read_exact(&mut description).is_err() {
                return Err(YpbankError::BinaryReadError);
            }

            bin_records.push(BinRecord {
                id,
                record_type,
                from_user_id,
                to_user_id,
                amount,
                timestamp,
                status,
                description,
            });
        }

        bin_records.into_iter().map(|br| br.try_into()).collect()
    }
}

pub struct BinRecordWriter;

impl BinRecordWriter {
    pub fn new() -> Self {
        Self
    }
}

impl RecordWriter for BinRecordWriter {
    fn write_all(&self, w: &mut dyn std::io::Write, records: &[Record]) -> Result<(), YpbankError> {
        for record in records {
            let bin_record = BinRecord::from(record);

            let mut buffer = vec![];

            buffer.extend_from_slice(&bin_record.id);
            buffer.push(bin_record.record_type);
            buffer.extend_from_slice(&bin_record.from_user_id);
            buffer.extend_from_slice(&bin_record.to_user_id);
            buffer.extend_from_slice(&bin_record.amount);
            buffer.extend_from_slice(&bin_record.timestamp);
            buffer.push(bin_record.status);
            buffer.extend_from_slice(&(bin_record.description.len() as u32).to_be_bytes());
            buffer.extend_from_slice(&bin_record.description);

            w.write_all(BinRecord::HEADER)
                .map_err(|_| YpbankError::WriteError)?;
            w.write_all(&(buffer.len() as u32).to_be_bytes())
                .map_err(|_| YpbankError::WriteError)?;
            w.write_all(&buffer).map_err(|_| YpbankError::WriteError)?;
        }

        Ok(())
    }
}

struct BinRecord {
    id: [u8; 8],
    record_type: u8,
    from_user_id: [u8; 8],
    to_user_id: [u8; 8],
    amount: [u8; 8],
    timestamp: [u8; 8],
    status: u8,
    description: Vec<u8>,
}

impl BinRecord {
    const HEADER: &[u8; 4] = b"YPBN";
}

impl TryInto<Record> for BinRecord {
    type Error = YpbankError;

    fn try_into(self) -> Result<Record, Self::Error> {
        let id = u64::from_be_bytes(self.id);
        let from_user_id = u64::from_be_bytes(self.from_user_id);
        let to_user_id = u64::from_be_bytes(self.to_user_id);
        let record_type = match self.record_type {
            0 => RecordType::Deposit { to_user_id },
            1 => RecordType::Transfer {
                from_user_id,
                to_user_id,
            },
            2 => RecordType::Withdrawal { from_user_id },
            _ => return Err(YpbankError::BinaryUnexpectedValue),
        };
        let amount = u64::from_be_bytes(self.amount);
        let timestamp = u64::from_be_bytes(self.timestamp);
        let status = match self.status {
            0 => RecordStatus::Success,
            1 => RecordStatus::Failure,
            2 => RecordStatus::Pending,
            _ => return Err(YpbankError::BinaryUnexpectedValue),
        };
        let description = if let Ok(str) = String::from_utf8(self.description) {
            str
        } else {
            return Err(YpbankError::BinaryUnexpectedValue);
        };
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

impl From<&Record> for BinRecord {
    fn from(value: &Record) -> Self {
        let (record_type, from_user_id, to_user_id) = match value.record_type {
            RecordType::Deposit { to_user_id } => (0, 0, to_user_id),
            RecordType::Withdrawal { from_user_id } => (2, from_user_id, 0),
            RecordType::Transfer {
                from_user_id,
                to_user_id,
            } => (1, from_user_id, to_user_id),
        };

        Self {
            id: value.id.to_be_bytes(),
            record_type: record_type,
            from_user_id: from_user_id.to_be_bytes(),
            to_user_id: to_user_id.to_be_bytes(),
            amount: value.amount.to_be_bytes(),
            timestamp: value.timestamp.to_be_bytes(),
            status: match value.status {
                RecordStatus::Success => 0,
                RecordStatus::Failure => 1,
                RecordStatus::Pending => 2,
            },
            description: value.description.as_bytes().to_vec(),
        }
    }
}
