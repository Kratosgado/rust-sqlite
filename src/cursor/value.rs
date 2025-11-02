use std::{borrow::Cow, rc::Rc};

use crate::sql::ast::Expr;

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum Value<'p> {
  Null,
  String(Cow<'p, str>),
  Blob(Cow<'p, [u8]>),
  Int(i64),
  Float(f64),
  Bool(bool),
}

impl<'p> Value<'p> {
  pub fn as_str(&self) -> Option<&str> {
    if let Value::String(s) = self {
      Some(s.as_ref())
    } else {
      None
    }
  }

  pub fn as_int(&self) -> Option<i64> {
    if let Value::Int(i) = self {
      Some(*i)
    } else {
      None
    }
  }
}

impl<'p> From<&Expr> for Value<'p> {
  fn from(value: &Expr) -> Self {
    match value {
      Expr::Column(_) => todo!(),
      Expr::Alias(_) => todo!(),
      Expr::Null => Value::Null,
      Expr::Int(i) => Value::Int(*i),
      Expr::Bool(v) => Value::Bool(*v),
      Expr::Real(i) => Value::Float(*i),
      Expr::Text(i) => Value::String(Cow::Owned(i.clone())),
      Expr::Comparison(_expr, _ops, _expr1) => todo!(),
    }
  }
}

impl<'p> From<bool> for Value<'p> {
  fn from(value: bool) -> Self {
    Value::Bool(value)
  }
}

impl<'p> From<Value<'p>> for bool {
  fn from(value: Value<'p>) -> Self {
    match value {
      Value::Int(v) => v == 1,
      _ => false,
    }
  }
}

#[derive(Debug, Clone, PartialEq)] // Added for testing
pub enum OwnedValue {
  Null,
  String(Rc<String>),
  Blob(Rc<Vec<u8>>),
  Int(i64),
  Bool(bool),
  Float(f64),
}

impl<'p> From<Value<'p>> for OwnedValue {
  fn from(value: Value<'p>) -> Self {
    match value {
      Value::Null => Self::Null,
      Value::Int(i) => Self::Int(i),
      Value::Bool(v) => Self::Bool(v),
      Value::Float(f) => Self::Float(f),
      Value::String(s) => Self::String(Rc::new(s.into_owned())),
      Value::Blob(b) => Self::Blob(Rc::new(b.into_owned())),
    }
  }
}

impl std::fmt::Display for OwnedValue {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      OwnedValue::Null => write!(f, "null"),
      OwnedValue::Blob(items) => {
        write!(
          f,
          "{}",
          items
            .iter()
            .filter_map(|&n| char::from_u32(n as u32).filter(char::is_ascii))
            .collect::<String>()
        )
      }
      OwnedValue::String(s) => s.fmt(f),
      OwnedValue::Bool(b) => b.fmt(f),
      OwnedValue::Int(i) => i.fmt(f),
      OwnedValue::Float(x) => x.fmt(f),
    }
  }
}
