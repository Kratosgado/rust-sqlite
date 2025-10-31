
use anyhow::{bail, Context};

use crate::{
  db::{Db, TableMetadata},
  engine::operator::SeqScanWithPredicate,
  sql::{
    ast::{self, Expr, ResultColumn, SelectFrom},
    tokenizer::Ops,
  },
};

use super::operator::{Operator, SeqScan};

pub struct Planner<'d> {
  db: &'d Db,
}

impl<'d> Planner<'d> {
  pub fn new(db: &'d Db) -> Self {
    Self { db }
  }

  pub fn compile(self, statement: &ast::Statement) -> anyhow::Result<Operator> {
    match statement {
      ast::Statement::Select(s) => self.compile_select(s),
      stmt => bail!("unsupported statement: {stmt:?}"),
    }
  }

  fn compile_select(self, select: &ast::SelectStatement) -> anyhow::Result<Operator> {
    let SelectFrom::Table(table_name) = &select.core.from;

    let table = self
      .db
      .tables_metadata
      .iter()
      .find(|m| &m.name == table_name)
      .with_context(|| format!("invalid table name: {table_name}"))?;

    let mut columns = vec![];
    let mut col_names = vec![];

    for res_col in &select.core.result_columns {
      match res_col {
        ResultColumn::Star => {
          for (i, col) in table.columns.iter().enumerate() {
            columns.push(i);
            col_names.push(col.name.clone());
          }
        }
        ResultColumn::Expr(e) => {
          let Expr::Column(col) = &e.expr else {
            anyhow::bail!("Expecting a column name")
          };
          let (index, _) = table
            .columns
            .iter()
            .enumerate()
            .find(|(_, c)| c.name == *col)
            .with_context(|| format!("invalid column name: {}", col))?;
          columns.push(index);
          col_names.push(if let Some(alias) = &e.alias {
            alias.clone()
          } else {
            col.clone()
          });
        }
      }
    }
    let formatted = col_names.join("\t| ");
    println!("{formatted}");
    println!("-----------------------------------------------------------------------------------------------------------------------");

    let operator = if let Some(Expr::Comparison(l, op, r)) = &select.core.where_clause {
      let predicate = self.compile_expr(*l.clone(), *op, r.clone(), table)?;
      Operator::SeqScanWithPredicate(SeqScanWithPredicate::new(
        &columns,
        self.db.scanner(table.first_page),
        predicate,
      ))
    } else {
      Operator::SeqScan(SeqScan::new(&columns, self.db.scanner(table.first_page)))
    };

    Ok(operator)
  }

  fn compile_expr(
    &self,
    l: Expr,
    op: Ops,
    r: Box<Expr>,
    table: &TableMetadata,
  ) -> anyhow::Result<Expr> {
    match l {
      Expr::Column(field) | Expr::Text(field) => {
        let idx = table
          .columns
          .iter()
          .enumerate()
          .find(|(_, c)| c.name == *field)
          .with_context(|| format!("invalid where field: {}", field))?
          .0;
        Ok(Expr::Comparison(Box::new(Expr::Alias(idx as i64)), op, r))
      }
      Expr::Comparison(l, ops, r1) => self.compile_expr(*l, ops, r1, table),
      Expr::Alias(_) => unimplemented!(),
      expr => anyhow::bail!("Expected a column or an alias. Got a value {expr:?}"),
    }
  }
}
