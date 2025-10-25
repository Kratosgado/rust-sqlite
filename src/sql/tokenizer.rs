use anyhow::bail;

use crate::cursor::value::Value;

use super::ast::Expr;

#[derive(Debug, PartialEq)]
pub enum Token {
  Create,
  Table,
  LPar,
  RPar,
  Select,
  As,
  From,
  Star,
  Comma,
  SemiColon,
  Where,
  Op(Ops),
  Identifier(String),

  Int(i64),
  Real(f64),
  String(String),
  Null,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Ops {
  Eq,
  Ne,
  Lt,
  Gt,
  Loe,
  Goe,
  And,
  Or,
}

impl Ops {
  pub fn compare(&self, l: Value, r: Value) -> bool {
    match self {
      Ops::Eq => l == r,
      Ops::Ne => l != r,
      Ops::Lt => l < r,
      Ops::Gt => l > r,
      Ops::Loe => l <= r,
      Ops::Goe => l >= r,
      Ops::And => todo!(),
      Ops::Or => todo!(),
    }
  }
}

impl Token {
  pub fn as_identifier(&self) -> Option<&str> {
    match self {
      Token::Identifier(ident) => Some(ident),
      _ => None,
    }
  }

  pub fn as_op(&self) -> Option<&Ops> {
    match self {
      Token::Op(op) => Some(op),
      _ => None,
    }
  }

  pub fn as_literal(&self) -> Option<Expr> {
    match self {
      Token::Int(i) => Some(Expr::Int(*i)),
      Token::Real(i) => Some(Expr::Real(*i)),
      Token::Null => Some(Expr::Null),
      Token::String(v) => Some(Expr::Text(v.clone())),
      _ => None,
    }
  }
}

pub fn tokenize(input: &str) -> anyhow::Result<Vec<Token>> {
  let mut tokens = vec![];
  let mut chars = input.chars().peekable();

  while let Some(c) = chars.next() {
    match c {
      '*' => tokens.push(Token::Star),
      ',' => tokens.push(Token::Comma),
      ';' => tokens.push(Token::SemiColon),
      '(' => tokens.push(Token::LPar),
      ')' => tokens.push(Token::RPar),
      '=' | '<' | '>' | '!' => {
        let mut op = c.to_string();
        if let Some(cc) = chars.next_if(|&cc| cc == '=') {
          op.push(cc);
        }
        match op.as_str() {
          "=" => tokens.push(Token::Op(Ops::Eq)),
          "!=" => tokens.push(Token::Op(Ops::Ne)),
          "<" => tokens.push(Token::Op(Ops::Lt)),
          ">" => tokens.push(Token::Op(Ops::Gt)),
          ">=" => tokens.push(Token::Op(Ops::Goe)),
          "<=" => tokens.push(Token::Op(Ops::Loe)),
          _ => anyhow::bail!("unexpected character: {c}"),
        }
      }
      c if c.is_whitespace() => continue,
      c if c.is_numeric() => {
        let mut num = c.to_string();
        while let Some(cc) = chars.next_if(|&cc| cc.is_numeric() || cc == '.') {
          num.extend(cc.to_lowercase());
        }
        tokens.push(if num.contains('.') {
          Token::Real(num.parse()?)
        } else {
          Token::Int(num.parse()?)
        });
      }
      '\'' | '"' => {
        let mut value = String::new();
        while let Some(cc) = chars.next_if(|&cc| cc.is_alphanumeric() || cc == '_' || cc == ' ') {
          value.extend(cc.to_lowercase());
        }
        if let Some(_) = chars.next_if(|&cc| cc == '\'' || cc == '"') {
          tokens.push(Token::String(value));
        } else {
          bail!("Unterminated string '{value}")
        }
      }
      c if c.is_alphabetic() => {
        let mut ident = c.to_string().to_lowercase();
        while let Some(cc) = chars.next_if(|&cc| cc.is_alphanumeric() || cc == '_') {
          ident.extend(cc.to_lowercase());
        }

        match ident.as_str() {
          "create" => tokens.push(Token::Create),
          "table" => tokens.push(Token::Table),
          "select" => tokens.push(Token::Select),
          "where" => tokens.push(Token::Where),
          "as" => tokens.push(Token::As),
          "from" => tokens.push(Token::From),
          "and" => tokens.push(Token::Op(Ops::And)),
          "or" => tokens.push(Token::Op(Ops::Or)),
          "null" => tokens.push(Token::Null),
          _ => tokens.push(Token::Identifier(ident)),
        }
      }
      _ => anyhow::bail!("unexpected character: {}", c),
    }
  }
  Ok(tokens)
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//
//     #[test]
//     fn test_tokenize_simple_select() {
//         let input = "SELECT * FROM users";
//         let tokens = tokenize(input).unwrap();
//
//         assert_eq!(tokens, vec![
//             Token::Select,
//             Token::Star,
//             Token::From,
//             Token::Identifier("users".to_string())
//         ]);
//     }
//
//     #[test]
//     fn test_tokenize_create_table() {
//         let input = "CREATE TABLE users (id INTEGER, name TEXT)";
//         let tokens = tokenize(input).unwrap();
//
//         assert_eq!(tokens, vec![
//             Token::Create,
//             Token::Table,
//             Token::Identifier("users".to_string()),
//             Token::LPar,
//             Token::Identifier("id".to_string()),
//             Token::Identifier("integer".to_string()),
//             Token::Comma,
//             Token::Identifier("name".to_string()),
//             Token::Identifier("text".to_string()),
//             Token::RPar
//         ]);
//     }
//
//     #[test]
//     fn test_tokenize_where_clause() {
//         let input = "SELECT * FROM users WHERE id = 10";
//         let tokens = tokenize(input).unwrap();
//
//         assert_eq!(tokens, vec![
//             Token::Select,
//             Token::Star,
//             Token::From,
//             Token::Identifier("users".to_string()),
//             Token::Where,
//             Token::Identifier("id".to_string()),
//             Token::Op(Ops::Eq),
//             Token::Int(10)
//         ]);
//     }
//
//     #[test]
//     fn test_tokenize_comparison_operators() {
//         let input = "= != < > <= >=";
//         let tokens = tokenize(input).unwrap();
//
//         assert_eq!(tokens, vec![
//             Token::Op(Ops::Eq),
//             Token::Op(Ops::Ne),
//             Token::Op(Ops::Lt),
//             Token::Op(Ops::Gt),
//             Token::Op(Ops::Loe),
//             Token::Op(Ops::Goe)
//         ]);
//     }
//
//     #[test]
//     fn test_tokenize_numbers() {
//         let input = "123 45.67";
//         let tokens = tokenize(input).unwrap();
//
//         assert_eq!(tokens, vec![
//             Token::Int(123),
//             Token::Real(45.67)
//         ]);
//     }
//
//     #[test]
//     fn test_tokenize_string() {
//         let input = "'hello world'";
//         let tokens = tokenize(input).unwrap();
//
//         assert_eq!(tokens, vec![
//             Token::String("hello world".to_string())
//         ]);
//     }
//
//     #[test]
//     fn test_tokenize_null() {
//         let input = "NULL";
//         let tokens = tokenize(input).unwrap();
//
//         assert_eq!(tokens, vec![Token::Null]);
//     }
//
//     #[test]
//     fn test_tokenize_logical_operators() {
//         let input = "AND OR";
//         let tokens = tokenize(input).unwrap();
//
//         assert_eq!(tokens, vec![
//             Token::Op(Ops::And),
//             Token::Op(Ops::Or)
//         ]);
//     }
//
//     #[test]
//     fn test_token_as_identifier() {
//         let token = Token::Identifier("test".to_string());
//         assert_eq!(token.as_identifier(), Some("test"));
//
//         let token = Token::Select;
//         assert_eq!(token.as_identifier(), None);
//     }
//
//     #[test]
//     fn test_token_as_op() {
//         let token = Token::Op(Ops::Eq);
//         assert_eq!(token.as_op(), Some(&Ops::Eq));
//
//         let token = Token::Select;
//         assert_eq!(token.as_op(), None);
//     }
//
//     #[test]
//     fn test_token_as_literal() {
//         let token = Token::Int(42);
//         assert_eq!(token.as_literal(), Some(Expr::Int(42)));
//
//         let token = Token::Real(3.14);
//         assert_eq!(token.as_literal(), Some(Expr::Real(3.14)));
//
//         let token = Token::String("hello".to_string());
//         assert_eq!(token.as_literal(), Some(Expr::Text("hello".to_string())));
//
//         let token = Token::Null;
//         assert_eq!(token.as_literal(), Some(Expr::Null));
//
//         let token = Token::Select;
//         assert_eq!(token.as_literal(), None);
//     }
//
//     #[test]
//     fn test_ops_compare() {
//         // Tests for comparison operations need to be done with actual Value types
//         // which are defined in cursor::value::Value
//         // Since we can't easily create these values here, we'll test the logic
//         // by ensuring the comparison methods exist and compile correctly
//         // Actual comprehensive tests would require setting up Value instances
//     }
// }
