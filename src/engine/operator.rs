use anyhow::{Context, Ok};

use crate::{
  cursor::{
    cursor::Cursor,
    scanner::Scanner,
    value::{OwnedValue, Value},
  },
  sql::ast::{Comparison, Expr},
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
    loop {
      let Some(record) = self.scanner.next_record()? else {
        return Ok(None);
      };
      if !apply_where(&record, self.predicate.as_comparison()?)? {
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

fn apply_where(record: &Cursor, predicate: Comparison) -> anyhow::Result<bool> {
  match predicate.l {
    Expr::Alias(_) => {
      let v = record.field(predicate.l.as_int()?).unwrap();
      let r = predicate.r;
      Ok(predicate.op.compare(v, Value::from(&r)))
    }
    Expr::Comparison(_, _, _) => {
      let left = apply_where(record, predicate.l.as_comparison()?)?;
      let right = apply_where(record, predicate.r.as_comparison()?)?;
      Ok(predicate.op.compare(left.into(), right.into()))
    }
    _ => anyhow::bail!("Expected a truthy value"),
  }
}
