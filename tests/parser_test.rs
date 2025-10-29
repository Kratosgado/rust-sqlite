#[cfg(test)]
mod tests {
  use rust_sqlite::sql::{
    ast::{ColumnDef, Expr, ResultColumn, SelectFrom, Statement, Type},
    parser::{parse_create_statement, parse_statement},
    tokenizer::Ops,
  };

  #[test]
  fn test_parse_simple_select() {
    let query = "SELECT * FROM users;";
    let result = parse_statement(query, false);
    assert!(result.is_ok());

    if let Ok(Statement::Select(select_stmt)) = result {
      assert_eq!(select_stmt.core.result_columns.len(), 1);
      if let ResultColumn::Star = &select_stmt.core.result_columns[0] {
      } else {
        panic!("Expected Star result column");
      }

      match &select_stmt.core.from {
        SelectFrom::Table(table_name) => {
          assert_eq!(table_name, "users");
        }
      }
    } else {
      panic!("Expected SELECT statement");
    }
  }

  #[test]
  fn test_parse_select_with_columns() {
    let query = "SELECT id, name FROM users;";
    let result = parse_statement(query, false);
    assert!(result.is_ok());

    if let Ok(Statement::Select(select_stmt)) = result {
      assert_eq!(select_stmt.core.result_columns.len(), 2);
      match &select_stmt.core.result_columns[0] {
        ResultColumn::Expr(expr_col) => {
          if let Expr::Column(col_name) = &expr_col.expr {
            assert_eq!(col_name, "id");
          } else {
            panic!("Expected column expression");
          }
        }
        _ => panic!("Expected expression result column"),
      }
    } else {
      panic!("Expected SELECT statement");
    }
  }

  #[test]
  fn test_parse_create_table() {
    let query = "CREATE TABLE users (id INTEGER, name TEXT)";
    let result = parse_statement(query, false);
    assert!(result.is_ok());

    if let Ok(Statement::CreateTable(create_stmt)) = result {
      assert_eq!(create_stmt.name, "users");
      assert_eq!(create_stmt.columns.len(), 2);

      assert_eq!(
        create_stmt.columns[0],
        ColumnDef {
          name: "id".to_string(),
          col_type: Type::Integer,
        }
      );

      assert_eq!(
        create_stmt.columns[1],
        ColumnDef {
          name: "name".to_string(),
          col_type: Type::Text,
        }
      );
    } else {
      panic!("Expected CREATE TABLE statement");
    }
  }

  #[test]
  fn test_parse_create_table_with_various_types() {
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
  fn test_parse_select_with_where_clause() {
    let query = "SELECT * FROM users WHERE id = 10;";
    let result = parse_statement(query, false);
    assert!(result.is_ok());

    if let Ok(Statement::Select(select_stmt)) = result {
      if let Some(Expr::Comparison(left, op, right)) = select_stmt.core.where_clause {
        if let Expr::Column(field) = left.as_ref() {
          assert_eq!(field, "id");
        } else {
          panic!("Expected column in where clause");
        }

        match op {
          Ops::Eq => {} // Expected
          _ => panic!("Expected equality operator"),
        }

        if let Expr::Int(val) = right.as_ref() {
          assert_eq!(*val, 10);
        } else {
          panic!("Expected integer in where clause");
        }
      } else {
        panic!("Expected where clause with comparison");
      }
    } else {
      panic!("Expected SELECT statement");
    }
  }

  #[test]
  fn test_parse_create_statement_function() {
    let query = "CREATE TABLE users (id INTEGER)";
    let result = parse_create_statement(query);
    assert!(result.is_ok());

    let create_stmt = result.unwrap();
    assert_eq!(create_stmt.name, "users");
    assert_eq!(create_stmt.columns[0].name, "id");
    assert_eq!(create_stmt.columns[0].col_type, Type::Integer);
  }

  #[test]
  fn test_parse_invalid_statement() {
    let query = "INVALID statement";
    let result = parse_statement(query, false);
    assert!(result.is_err());
  }

  #[test]
  fn test_parse_create_with_unsupported_type() {
    let query = "CREATE TABLE test (data INVALID_TYPE)";
    let result = parse_statement(query, false);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("unsupported type"));
  }
}
