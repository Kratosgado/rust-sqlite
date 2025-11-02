#[cfg(test)]
mod parser {
  use rust_sqlite::sql::{
    ast::{ColumnDef, Expr, ExprResultColumn, ResultColumn, SelectFrom, Statement, Type},
    parser::{parse_create_statement, parse_statement},
    tokenizer::Ops,
  };

  #[test]
  fn simple_select() {
    let query = "SELECT * FROM users;";
    let result = parse_statement(query, false);
    assert!(result.is_ok());

    let Statement::Select(select_stmt) = result.unwrap() else {
      panic!("Expected SELECT statement");
    };

    assert_eq!(select_stmt.core.result_columns.len(), 1);
    assert_eq!(ResultColumn::Star, select_stmt.core.result_columns[0]);
    assert_eq!(
      select_stmt.core.from,
      SelectFrom::Table("users".to_string())
    );
  }

  #[test]
  fn select_with_columns() {
    let query = "SELECT id, name FROM users;";
    let result = parse_statement(query, false);
    assert!(result.is_ok());

    let Statement::Select(select_stmt) = result.unwrap() else {
      panic!("Expected SELECT statement");
    };

    assert_eq!(select_stmt.core.result_columns.len(), 2);
    assert_eq!(
      select_stmt.core.result_columns,
      vec![
        ResultColumn::Expr(ExprResultColumn {
          expr: Expr::Column("id".to_string()),
          alias: None
        }),
        ResultColumn::Expr(ExprResultColumn {
          expr: Expr::Column("name".to_string()),
          alias: None
        }),
      ]
    );
  }

  #[test]
  fn create_table_with_various_types() {
    let query = "CREATE TABLE users (id INTEGER,  name TEXT, is_admin BOOL, amount REAL,raw BLOB)";
    let result = parse_statement(query, false);
    assert!(result.is_ok());
    let Statement::CreateTable(create_stmt) = result.unwrap() else {
      panic!("Expected SELECT statement");
    };

    assert_eq!(create_stmt.name, "users");
    assert_eq!(
      create_stmt.columns,
      vec![
        ColumnDef {
          name: "id".to_string(),
          col_type: Type::Integer,
        },
        ColumnDef {
          name: "name".to_string(),
          col_type: Type::Text,
        },
        ColumnDef {
          name: "is_admin".to_string(),
          col_type: Type::Bool,
        },
        ColumnDef {
          name: "amount".to_string(),
          col_type: Type::Real,
        },
        ColumnDef {
          name: "raw".to_string(),
          col_type: Type::Blob,
        }
      ]
    );
  }

  #[test]
  fn select_with_where_clause() {
    let query = "SELECT * FROM users WHERE id = 10;";
    let result = parse_statement(query, false);
    assert!(result.is_ok());

    if let Ok(Statement::Select(select_stmt)) = result {
      if let Some(Expr::Comparison(left, op, right)) = select_stmt.core.where_clause {
        assert_eq!(*left, Expr::Column("id".to_string()));
        assert_eq!(op, Ops::Eq);
        assert_eq!(*right, Expr::Int(10));
      } else {
        panic!("Expected where clause with comparison");
      }
    } else {
      panic!("Expected SELECT statement");
    }
  }
  #[test]
  fn select_with_where_compound_clause() {
    let query = "SELECT * FROM users WHERE id = 10 or name = 'kratos';";
    let result = parse_statement(query, false);
    assert!(result.is_ok());

    if let Ok(Statement::Select(select_stmt)) = result {
      if let Some(Expr::Comparison(left, op, right)) = select_stmt.core.where_clause {
        if let Expr::Comparison(left, op, right) = *left {
          assert_eq!(*left, Expr::Column("id".to_string()));
          assert_eq!(op, Ops::Eq);
          assert_eq!(*right, Expr::Int(10));
        } else {
          panic!("Expected where clause with comparison");
        }
        assert_eq!(op, Ops::Or);
        if let Expr::Comparison(left, op, right) = *right {
          assert_eq!(*left, Expr::Column("name".to_string()));
          assert_eq!(op, Ops::Eq);
          assert_eq!(*right, Expr::Text("kratos".to_string()));
        } else {
          panic!("Expected where clause with comparison");
        }
      } else {
        panic!("Expected SELECT statement");
      }
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
