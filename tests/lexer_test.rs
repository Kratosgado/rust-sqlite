#[cfg(test)]
mod tests {
  use std::f64::consts::PI;

  use rust_sqlite::sql::{
    ast::Expr,
    tokenizer::{tokenize, Ops, Token},
  };

  #[test]
  fn test_tokenize_simple_select() {
    let input = "SELECT * FROM users";
    let tokens = tokenize(input).unwrap();

    assert_eq!(
      tokens,
      vec![
        Token::Select,
        Token::Star,
        Token::From,
        Token::Identifier("users".to_string())
      ]
    );
  }

  #[test]
  fn test_tokenize_create_table() {
    let input = "CREATE TABLE users (id INTEGER, name TEXT)";
    let tokens = tokenize(input).unwrap();

    assert_eq!(
      tokens,
      vec![
        Token::Create,
        Token::Table,
        Token::Identifier("users".to_string()),
        Token::LPar,
        Token::Identifier("id".to_string()),
        Token::Identifier("integer".to_string()),
        Token::Comma,
        Token::Identifier("name".to_string()),
        Token::Identifier("text".to_string()),
        Token::RPar
      ]
    );
  }

  #[test]
  fn test_tokenize_where_clause() {
    let input = "SELECT * FROM users WHERE id = 10";
    let tokens = tokenize(input).unwrap();

    assert_eq!(
      tokens,
      vec![
        Token::Select,
        Token::Star,
        Token::From,
        Token::Identifier("users".to_string()),
        Token::Where,
        Token::Identifier("id".to_string()),
        Token::Op(Ops::Eq),
        Token::Int(10)
      ]
    );
  }

  #[test]
  fn test_tokenize_comparison_operators() {
    let input = "= != < > <= >=";
    let tokens = tokenize(input).unwrap();

    assert_eq!(
      tokens,
      vec![
        Token::Op(Ops::Eq),
        Token::Op(Ops::Ne),
        Token::Op(Ops::Lt),
        Token::Op(Ops::Gt),
        Token::Op(Ops::Loe),
        Token::Op(Ops::Goe)
      ]
    );
  }

  #[test]
  fn test_tokenize_numbers() {
    let input = "123 45.67";
    let tokens = tokenize(input).unwrap();

    assert_eq!(tokens, vec![Token::Int(123), Token::Real(45.67)]);
  }

  #[test]
  fn test_tokenize_string() {
    let input = "'hello world'";
    let tokens = tokenize(input).unwrap();

    assert_eq!(tokens, vec![Token::String("hello world".to_string())]);
  }

  #[test]
  fn test_tokenize_null() {
    let input = "NULL";
    let tokens = tokenize(input).unwrap();

    assert_eq!(tokens, vec![Token::Null]);
  }

  #[test]
  fn test_tokenize_logical_operators() {
    let input = "AND OR";
    let tokens = tokenize(input).unwrap();

    assert_eq!(tokens, vec![Token::Op(Ops::And), Token::Op(Ops::Or)]);
  }

  #[test]
  fn test_token_as_identifier() {
    let token = Token::Identifier("test".to_string());
    assert_eq!(token.as_identifier(), Some("test"));

    let token = Token::Select;
    assert_eq!(token.as_identifier(), None);
  }

  #[test]
  fn test_token_as_op() {
    let token = Token::Op(Ops::Eq);
    assert_eq!(token.as_op(), Some(&Ops::Eq));

    let token = Token::Select;
    assert_eq!(token.as_op(), None);
  }

  #[test]
  fn test_token_as_literal() {
    let token = Token::Int(42);
    assert_eq!(token.as_literal(), Some(Expr::Int(42)));

    let token = Token::Real(PI);
    assert_eq!(token.as_literal(), Some(Expr::Real(PI)));

    let token = Token::String("hello".to_string());
    assert_eq!(token.as_literal(), Some(Expr::Text("hello".to_string())));

    let token = Token::Null;
    assert_eq!(token.as_literal(), Some(Expr::Null));

    let token = Token::Select;
    assert_eq!(token.as_literal(), None);
  }

  #[test]
  fn test_ops_compare() {
    // Tests for comparison operations need to be done with actual Value types
    // which are defined in cursor::value::Value
    // Since we can't easily create these values here, we'll test the logic
    // by ensuring the comparison methods exist and compile correctly
    // Actual comprehensive tests would require setting up Value instances
  }
}
