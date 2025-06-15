use anyhow::Context;

use crate::{
    cursor::{scanner::Scanner, value::OwnedValue},
    sql::ast::Predicate,
};

#[derive(Debug)]
pub enum Operator {
    SeqScan(SeqScan),
    SeqScanWithPredicate(SeqScanWithPredicate),
}

impl Operator {
    pub fn next_row(&mut self) -> anyhow::Result<Option<&[OwnedValue]>> {
        match self {
            Operator::SeqScan(s) => s.next_row(),
            Operator::SeqScanWithPredicate(s) => s.next_row(),
        }
    }
}

/// Sequencial scan
#[derive(Debug)]
pub struct SeqScan {
    fields: Vec<usize>,
    scanner: Scanner,
    row_buffer: Vec<OwnedValue>,
}

#[derive(Debug)]
pub struct SeqScanWithPredicate {
    fields: Vec<usize>,
    scanner: Scanner,
    row_buffer: Vec<OwnedValue>,
    predicate: Predicate,
}

impl SeqScan {
    pub fn new(fields: Vec<usize>, scanner: Scanner) -> Self {
        let row_buffer = vec![OwnedValue::Null; fields.len()];

        Self {
            fields,
            scanner,
            row_buffer,
        }
    }

    fn next_row(&mut self) -> anyhow::Result<Option<&[OwnedValue]>> {
        let Some(record) = self.scanner.next_record()? else {
            return Ok(None);
        };

        for (i, &n) in self.fields.iter().enumerate() {
            self.row_buffer[i] = record.owned_field(n).context("missing record field")?;
        }

        Ok(Some(&self.row_buffer))
    }
}

impl SeqScanWithPredicate {
    pub fn new(fields: Vec<usize>, scanner: Scanner, predicate: Predicate) -> SeqScanWithPredicate {
        let row_buffer = vec![OwnedValue::Null; fields.len()];

        SeqScanWithPredicate {
            fields,
            scanner,
            row_buffer,
            predicate,
        }
    }

    fn next_row(&mut self) -> anyhow::Result<Option<&[OwnedValue]>> {
        let pred = &self.predicate;
        loop {
            let Some(record) = self.scanner.next_record()? else {
                return Ok(None);
            };
            if !record.field(pred.field).unwrap().compare(&pred.value) {
                continue;
            }

            for (i, &n) in self.fields.iter().enumerate() {
                self.row_buffer[i] = record.owned_field(n).context("missing record field")?;
            }
            break;
        }
        Ok(Some(&self.row_buffer))
    }
}
