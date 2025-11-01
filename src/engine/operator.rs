use std::ops::Deref;

use anyhow::Context;

use crate::{
  cursor::{scanner::Scanner, value::OwnedValue},
  sql::ast::Expr,
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
  pub fields: Vec<usize>,
  pub scanner: Scanner,
  row_buffer: Vec<OwnedValue>,
}

#[derive(Debug)]
pub struct SeqScanWithPredicate {
  fields: Vec<usize>,
  scanner: Scanner,
  row_buffer: Vec<OwnedValue>,
  pub predicate: Expr,
}

impl SeqScan {
  pub fn new(fields: &[usize], scanner: Scanner) -> Self {
    let row_buffer = vec![OwnedValue::Null; fields.len()];

    Self {
      fields: fields.to_vec(),
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
  pub fn new(fields: &[usize], scanner: Scanner, predicate: Expr) -> SeqScanWithPredicate {
    let row_buffer = vec![OwnedValue::Null; fields.len()];

    SeqScanWithPredicate {
      fields: fields.to_vec(),
      scanner,
      row_buffer,
      predicate,
    }
  }

  fn next_row(&mut self) -> anyhow::Result<Option<&[OwnedValue]>> {
    println!("{:?}", self.predicate);
    let Expr::Comparison(l, op, r) = &self.predicate else {
      anyhow::bail!("Expected a truthy value")
    };
    loop {
      let Some(record) = self.scanner.next_record()? else {
        return Ok(None);
      };
      let v = record.field(l.as_int()?).unwrap();
      if !op.compare(v, r.deref().into()) {
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
