use anyhow::Context;

use crate::{
    cursor::{scanner::Scanner, value::OwnedValue},
    sql::ast::WhereClause,
};

#[derive(Debug)]
pub enum Operator {
    SeqScan(SeqScan),
}

impl Operator {
    pub fn next_row(&mut self) -> anyhow::Result<Option<&[OwnedValue]>> {
        match self {
            Operator::SeqScan(s) => s.next_row(),
        }
    }
}

#[derive(Debug)]
pub struct SeqScan {
    fields: Vec<usize>,
    aliases: Option<Vec<String>>,
    scanner: Scanner,
    row_buffer: Vec<OwnedValue>,
    where_clause: Option<WhereClause>,
}

impl SeqScan {
    pub fn new(
        fields: Vec<usize>,
        aliases: Option<Vec<String>>,
        scanner: Scanner,
        where_clause: Option<WhereClause>,
    ) -> Self {
        let row_buffer = vec![OwnedValue::Null; fields.len()];

        Self {
            fields,
            aliases,
            scanner,
            row_buffer,
            where_clause,
        }
    }

    fn next_row(&mut self) -> anyhow::Result<Option<&[OwnedValue]>> {
        let Some(record) = self.scanner.next_record()? else {
            return Ok(None);
        };

        // println!("record: {record:?}");
        for (i, &n) in self.fields.iter().enumerate() {
            self.row_buffer[i] = record.owned_field(n).context("missing record field")?;
        }

        Ok(Some(&self.row_buffer))
    }
}
