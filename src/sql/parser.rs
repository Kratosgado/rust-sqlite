use anyhow::{bail, Context};

use super::{
  ast::{
    ColumnDef, CreateTableStatement, Expr, ExprResultColumn, ResultColumn, SelectCore, SelectFrom,
    SelectStatement, Statement, Type,
  },
  tokenizer::{self, Ops, Token},
};

#[derive(Debug)]
struct ParserState {
  tokens: Vec<Token>,
  pos: usize,
}

impl ParserState {
  fn new(tokens: Vec<Token>) -> Self {
    Self { tokens, pos: 0 }
  }

  fn parse_statement(&mut self) -> anyhow::Result<Statement> {
    match self.peak_next_token().context("unexpected end of input")? {
      Token::Create => self.parse_create_table().map(Statement::CreateTable),
      Token::Select => self.parse_select().map(Statement::Select),
      token => bail!("unexpected token: {token:?}"),
    }
  }

  fn parse_select(&mut self) -> anyhow::Result<SelectStatement> {
    self.advance();
    let result_columns = self.parse_result_columns()?;
    self.expect_eq(Token::From)?;
    let from = self.parse_select_from()?;

    let mut where_clause = None;
    match self.peak_next_token()? {
      Token::Where => {
        where_clause = Some(self.parse_where_clause()?);
      }
      _ => {}
    }

    Ok(SelectStatement {
      core: SelectCore {
        result_columns,
        from,
        where_clause,
      },
    })
  }

  fn parse_select_from(&mut self) -> anyhow::Result<SelectFrom> {
    let table = self.expected_identifier()?;
    Ok(SelectFrom::Table(table.to_string()))
  }

  fn parse_where_clause(&mut self) -> anyhow::Result<Expr> {
    self.advance();
    let mut expr = Expr::Comparison(
      Box::new(self.parse_expr()?),
      *(self.expect_operator()?),
      Box::new(self.expect_literal()?),
    );
    match self.peak_next_token() {
      Ok(Token::Op(new_op)) => {
        expr = Expr::Comparison(
          Box::new(expr),
          *new_op,
          Box::new(self.parse_where_clause()?),
        )
      }
      Err(e) => bail!("Error parsing where: {e:?}"),
      _ => {}
    }

    Ok(expr)
  }

  fn parse_result_columns(&mut self) -> anyhow::Result<Vec<ResultColumn>> {
    let mut result_columns = vec![self.parse_result_column()?];
    while self.next_token_is(Token::Comma) {
      self.advance();
      result_columns.push(self.parse_result_column()?);
    }
    Ok(result_columns)
  }

  fn parse_result_column(&mut self) -> anyhow::Result<ResultColumn> {
    if self.peak_next_token()? == &Token::Star {
      self.advance();
      return Ok(ResultColumn::Star);
    }

    Ok(ResultColumn::Expr(self.parse_expr_result_column()?))
  }

  fn parse_expr(&mut self) -> anyhow::Result<Expr> {
    Ok(Expr::Column(self.expected_identifier()?.to_string()))
  }

  fn parse_expr_result_column(&mut self) -> anyhow::Result<ExprResultColumn> {
    let expr = self.parse_expr()?;
    let alias = if self.next_token_is(Token::As) {
      self.advance();
      Some(self.expected_identifier()?.to_string())
    } else {
      None
    };
    Ok(ExprResultColumn { expr, alias })
  }

  fn next_token_is(&self, expected: Token) -> bool {
    self.tokens.get(self.pos) == Some(&expected)
  }

  fn expected_identifier(&mut self) -> anyhow::Result<&str> {
    self
      .expect_matching(|t| matches!(t, Token::Identifier(_)))
      .map(|t| t.as_identifier().unwrap())
  }

  fn expect_operator(&mut self) -> anyhow::Result<&Ops> {
    self
      .expect_matching(|t| matches!(t, Token::Op(_)))
      .map(|t| t.as_op().unwrap())
  }
  fn expect_literal(&mut self) -> anyhow::Result<Expr> {
    self
      .expect_matching(|t| {
        matches!(
          t,
          Token::Null | Token::Int(_) | Token::Real(_) | Token::String(_)
        )
      })
      .map(|t| t.as_literal().unwrap())
  }

  fn expect_eq(&mut self, expected: Token) -> anyhow::Result<&Token> {
    self.expect_matching(|t| *t == expected)
  }

  fn expect_matching(&mut self, f: impl Fn(&Token) -> bool) -> anyhow::Result<&Token> {
    match self.next_token() {
      Some(token) if f(token) => Ok(token),
      Some(token) => anyhow::bail!("unexpected token: {:?}", token),
      None => anyhow::bail!("unexpected end of input"),
    }
  }

  fn peak_next_token(&self) -> anyhow::Result<&Token> {
    self.tokens.get(self.pos).context("unexpected end of input")
  }

  fn next_token(&mut self) -> Option<&Token> {
    let token = self.tokens.get(self.pos);
    if token.is_some() {
      self.pos += 1;
      // self.advance();
    }
    token
  }

  fn parse_create_table(&mut self) -> anyhow::Result<CreateTableStatement> {
    self.expect_eq(Token::Create)?;
    self.expect_eq(Token::Table)?;
    let name = self.expected_identifier()?.to_string();
    self.expect_eq(Token::LPar)?;
    let mut columns = vec![self.parse_column_def()?];
    while self.next_token_is(Token::Comma) {
      self.advance();
      columns.push(self.parse_column_def()?);
    }
    self.expect_eq(Token::RPar)?;
    Ok(CreateTableStatement { name, columns })
  }
  fn parse_column_def(&mut self) -> anyhow::Result<ColumnDef> {
    Ok(ColumnDef {
      name: self.expected_identifier()?.to_string(),
      col_type: self.parse_type()?,
    })
  }

  fn parse_type(&mut self) -> anyhow::Result<Type> {
    let type_name = self.expected_identifier()?;
    let t = match type_name.to_lowercase().as_str() {
      "integer" => Type::Integer,
      "real" => Type::Real,
      "blob" => Type::Blob,
      "text" | "string" => Type::Text,
      _ => bail!("unsupported type: {type_name}"),
    };
    Ok(t)
  }

  fn advance(&mut self) {
    self.pos += 1;
  }
}

pub fn parse_statement(input: &str, trailing_semicolon: bool) -> anyhow::Result<Statement> {
  let tokens = tokenizer::tokenize(input)?;
  println!("tokens: {tokens:?}");
  let mut state = ParserState::new(tokens);
  let statements = state.parse_statement()?;
  if trailing_semicolon {
    state.expect_eq(Token::SemiColon)?;
  }

  println!("parsed: {statements:?}");
  Ok(statements)
}

pub fn parse_create_statement(input: &str) -> anyhow::Result<CreateTableStatement> {
  match parse_statement(input, false)? {
    Statement::CreateTable(c) => Ok(c),
    Statement::Select(_) => bail!("expected a create statement"),
  }
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use crate::sql::ast::{ColumnDef, ResultColumn, SelectFrom, Type, Expr, Statement};
//
//     #[test]
//     fn test_parse_simple_select() {
//         let query = "SELECT * FROM users";
//         let result = parse_statement(query, false);
//         assert!(result.is_ok());
//
//         if let Ok(Statement::Select(select_stmt)) = result {
//             assert_eq!(select_stmt.core.result_columns.len(), 1);
//             if let ResultColumn::Star = &select_stmt.core.result_columns[0] {}
//             else {
//                 panic!("Expected Star result column");
//             }
//
//             if let SelectFrom::Table(table_name) = &select_stmt.core.from {
//                 assert_eq!(table_name, "users");
//             } else {
//                 panic!("Expected table from clause");
//             }
//         } else {
//             panic!("Expected SELECT statement");
//         }
//     }
//
//     #[test]
//     fn test_parse_select_with_columns() {
//         let query = "SELECT id, name FROM users";
//         let result = parse_statement(query, false);
//         assert!(result.is_ok());
//
//         if let Ok(Statement::Select(select_stmt)) = result {
//             assert_eq!(select_stmt.core.result_columns.len(), 2);
//             match &select_stmt.core.result_columns[0] {
//                 ResultColumn::Expr(expr_col) => {
//                     if let Expr::Column(col_name) = &expr_col.expr {
//                         assert_eq!(col_name, "id");
//                     } else {
//                         panic!("Expected column expression");
//                     }
//                 },
//                 _ => panic!("Expected expression result column"),
//             }
//         } else {
//             panic!("Expected SELECT statement");
//         }
//     }
//
//     #[test]
//     fn test_parse_create_table() {
//         let query = "CREATE TABLE users (id INTEGER, name TEXT)";
//         let result = parse_statement(query, false);
//         assert!(result.is_ok());
//
//         if let Ok(Statement::CreateTable(create_stmt)) = result {
//             assert_eq!(create_stmt.name, "users");
//             assert_eq!(create_stmt.columns.len(), 2);
//
//             assert_eq!(create_stmt.columns[0], ColumnDef {
//                 name: "id".to_string(),
//                 col_type: Type::Integer,
//             });
//
//             assert_eq!(create_stmt.columns[1], ColumnDef {
//                 name: "name".to_string(),
//                 col_type: Type::Text,
//             });
//         } else {
//             panic!("Expected CREATE TABLE statement");
//         }
//     }
//
//     #[test]
//     fn test_parse_create_table_with_various_types() {
//         let query = "CREATE TABLE test (id INTEGER, value REAL, data TEXT, raw BLOB)";
//         let result = parse_statement(query, false);
//         assert!(result.is_ok());
//
//         if let Ok(Statement::CreateTable(create_stmt)) = result {
//             assert_eq!(create_stmt.columns[0].col_type, Type::Integer);
//             assert_eq!(create_stmt.columns[1].col_type, Type::Real);
//             assert_eq!(create_stmt.columns[2].col_type, Type::Text);
//             assert_eq!(create_stmt.columns[3].col_type, Type::Blob);
//         } else {
//             panic!("Expected CREATE TABLE statement");
//         }
//     }
//
//     #[test]
//     fn test_parse_select_with_where_clause() {
//         let query = "SELECT * FROM users WHERE id = 10";
//         let result = parse_statement(query, false);
//         assert!(result.is_ok());
//
//         if let Ok(Statement::Select(select_stmt)) = result {
//             if let Some(Expr::Comparison(left, op, right)) = select_stmt.core.where_clause {
//                 if let Expr::Column(field) = left.as_ref() {
//                     assert_eq!(field, "id");
//                 } else {
//                     panic!("Expected column in where clause");
//                 }
//
//                 match op {
//                     Ops::Eq => {}, // Expected
//                     _ => panic!("Expected equality operator"),
//                 }
//
//                 if let Expr::Int(val) = right.as_ref() {
//                     assert_eq!(*val, 10);
//                 } else {
//                     panic!("Expected integer in where clause");
//                 }
//             } else {
//                 panic!("Expected where clause with comparison");
//             }
//         } else {
//             panic!("Expected SELECT statement");
//         }
//     }
//
//     #[test]
//     fn test_parse_create_statement_function() {
//         let query = "CREATE TABLE users (id INTEGER)";
//         let result = parse_create_statement(query);
//         assert!(result.is_ok());
//
//         let create_stmt = result.unwrap();
//         assert_eq!(create_stmt.name, "users");
//         assert_eq!(create_stmt.columns[0].name, "id");
//         assert_eq!(create_stmt.columns[0].col_type, Type::Integer);
//     }
//
//     #[test]
//     fn test_parse_invalid_statement() {
//         let query = "INVALID statement";
//         let result = parse_statement(query, false);
//         assert!(result.is_err());
//     }
//
//     #[test]
//     fn test_parse_create_with_unsupported_type() {
//         let query = "CREATE TABLE test (data INVALID_TYPE)";
//         let result = parse_statement(query, false);
//         assert!(result.is_err());
//         assert!(result.unwrap_err().to_string().contains("unsupported type"));
//     }
// }
