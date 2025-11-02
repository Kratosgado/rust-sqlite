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
    if let Token::Where = self.peak_next_token()? {
      where_clause = Some(self.parse_where_clause()?);
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
      "bool" => Type::Bool,
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
  let mut state = ParserState::new(tokens);
  let statements = state.parse_statement()?;
  if trailing_semicolon {
    state.expect_eq(Token::SemiColon)?;
  }
  Ok(statements)
}

pub fn parse_create_statement(input: &str) -> anyhow::Result<CreateTableStatement> {
  match parse_statement(input, false)? {
    Statement::CreateTable(c) => Ok(c),
    Statement::Select(_) => bail!("expected a create statement"),
  }
}
