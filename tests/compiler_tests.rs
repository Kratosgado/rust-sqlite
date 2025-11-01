#[cfg(test)]
mod compiler {
  use rust_sqlite::{
    db::Db,
    engine::{operator::Operator, plan::Planner},
    sql::{
      ast::{Expr, Statement, Type},
      parser::{parse_create_statement, parse_statement},
      tokenizer::Ops,
    },
  };

  #[test]
  fn simple_select() {
    let db = &Db::from_file("queries_test.db").unwrap();
    let query = "SELECT * FROM users;";
    let parsed = &parse_statement(query, false).unwrap();
    let op = Planner::new(db).compile(parsed);
    assert!(op.is_ok());
    match op.unwrap() {
      Operator::SeqScan(s) => {
        assert_eq!(s.fields.len(), 2);
      }
      _ => panic!("Expected Sequential Scan operation"),
    }
  }

  #[test]
  fn select_with_columns() {
    let query = "SELECT id, name FROM users;";
    let db = &Db::from_file("queries_test.db").unwrap();
    let parsed = &parse_statement(query, false).unwrap();
    let op = Planner::new(db).compile(parsed);
    assert!(op.is_ok());
    match op.unwrap() {
      Operator::SeqScan(s) => {
        assert_eq!(s.fields.len(), 2);
      }
      _ => panic!("Expected Sequential Scan operation"),
    }
  }
  #[test]
  fn select_with_where_clause() {
    let query = "SELECT * FROM users WHERE id = 10;";
    let db = &Db::from_file("queries_test.db").unwrap();
    let parsed = &parse_statement(query, false).unwrap();
    let op = Planner::new(db).compile(parsed);
    assert!(op.is_ok());
    match op.unwrap() {
      Operator::SeqScanWithPredicate(s) => match s.predicate {
        Expr::Comparison(l, ops, r) => {
          assert_eq!(*l, Expr::Alias(0));
          assert_eq!(ops, Ops::Eq);
          assert_eq!(*r, Expr::Int(10));
        }
        _ => panic!("Expected a comparison"),
      },
      _ => panic!("Expected Sequential Scan with predicate operation"),
    }
  }

  #[test]
  fn select_with_where_compound_clause() {
    let query = "SELECT * FROM users WHERE id = 10 or name = 'kratos';";
    let db = &Db::from_file("queries_test.db").unwrap();
    let parsed = &parse_statement(query, false).unwrap();
    let op = Planner::new(db).compile(parsed);
    assert!(op.is_ok());
    match op.unwrap() {
      Operator::SeqScanWithPredicate(s) => match s.predicate {
        Expr::Comparison(l, ops, r) => {
          match *l {
            Expr::Comparison(l, ops, r) => {
              assert_eq!(*l, Expr::Alias(0));
              assert_eq!(ops, Ops::Eq);
              assert_eq!(*r, Expr::Int(10));
            }
            _ => panic!("Expected a comparison"),
          }
          assert_eq!(ops, Ops::Or);
          match *r {
            Expr::Comparison(l, ops, r) => {
              assert_eq!(*l, Expr::Alias(1));
              assert_eq!(ops, Ops::Eq);
              assert_eq!(*r, Expr::Text("kratos".to_string()));
            }
            _ => panic!("Expected a comparison"),
          }
        }
        _ => panic!("Expected a comparison"),
      },
      _ => panic!("Expected Sequential Scan with predicate operation"),
    }
  }

  #[test]
  fn create_statement_function() {
    let query = "CREATE TABLE users (id INTEGER)";
    let result = parse_create_statement(query);
    assert!(result.is_ok());

    let create_stmt = result.unwrap();
    assert_eq!(create_stmt.name, "users");
    assert_eq!(create_stmt.columns[0].name, "id");
    assert_eq!(create_stmt.columns[0].col_type, Type::Integer);
  }

  #[test]
  fn create_table() {
    // let query = "CREATE TABLE users (id INTEGER, name TEXT)";
    // let db = &Db::from_file("queries_test.db").unwrap();
    // let parsed = &parse_statement(query, false).unwrap();
    // let op = Planner::new(db).compile(parsed);
    // assert!(true)
  }

  #[test]
  fn create_table_with_various_types() {
    //TODO: fix after create function
    let query = "CREATE TABLE test (id INTEGER, value REAL, data TEXT, raw BLOB)";
    let result = parse_statement(query, false);
    assert!(result.is_ok());

    if let Ok(Statement::CreateTable(create_stmt)) = result {
      assert_eq!(create_stmt.columns[0].col_type, Type::Integer);
      assert_eq!(create_stmt.columns[1].col_type, Type::Real);
      assert_eq!(create_stmt.columns[2].col_type, Type::Text);
      assert_eq!(create_stmt.columns[3].col_type, Type::Blob);
    } else {
      panic!("Expected CREATE TABLE statement");
    }
  }

  #[test]
  fn invalid_statement() {
    let query = "INVALID statement";
    let result = parse_statement(query, false);
    assert!(result.is_err());
  }

  #[test]
  fn create_with_unsupported_type() {
    let query = "CREATE TABLE test (data INVALID_TYPE)";
    let result = parse_statement(query, false);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("unsupported type"));
  }
}
