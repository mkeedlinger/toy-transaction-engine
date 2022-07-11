use crate::prelude::*;
use crate::transaction::Transaction;
use csv::{DeserializeRecordsIntoIter, ReaderBuilder, Trim};
use std::collections::VecDeque;

pub struct LikelyValidLine(String);

pub struct CsvParser {
    reader: DeserializeRecordsIntoIter<VecDeque<u8>, Transaction>,
}

impl CsvParser {
    pub fn line_to_transaction(&mut self, line: LikelyValidLine) -> Result<Transaction> {
        {
            let reader = self.reader.reader_mut().get_mut();

            reader.append(&mut VecDeque::from(line.0.into_bytes()));
            reader.push_back(b'\n');
        }

        let transaction = self
            .reader
            .next()
            .expect("Line just added, there should be lines left!")?;

        Ok(transaction)
    }

    pub fn new(line_with_headers: LikelyValidLine) -> Result<Self> {
        let mut reader = VecDeque::from(line_with_headers.0.into_bytes());
        reader.push_back(b'\n');

        let reader = ReaderBuilder::new()
            .trim(Trim::All)
            .from_reader(reader)
            .into_deserialize();

        Ok(Self { reader })
    }

    pub fn valid_line(line: String) -> Option<LikelyValidLine> {
        if line.is_empty() || !line.is_ascii() {
            None
        } else {
            Some(LikelyValidLine(line))
        }
    }
}

#[test]
fn parse_with_dispute() {
    let input = r#"
type,         client,   tx,   amount
dispute,      1,        1,
"#;
    let mut lines = input.lines();
    lines.next();
    let mut parser =
        CsvParser::new(CsvParser::valid_line(String::from(lines.next().unwrap())).unwrap())
            .unwrap();
    let _transaction = parser
        .line_to_transaction(CsvParser::valid_line(String::from(lines.next().unwrap())).unwrap())
        .unwrap();
}
