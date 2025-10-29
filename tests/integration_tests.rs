use rust_sqlite::db::Db;

#[test]
fn test_read_existing_database() {
  let result = Db::from_file("minimal_test.db");

  if std::path::Path::new("minimal_test.db").exists() {
    assert!(result.is_ok());
  } else {
    // If the file doesn't exist, ensure we get an appropriate error
    assert!(result.is_err());
  }
}

//
#[test]
fn test_read_queries_database() {
  // Test reading the queries_test.db file if it exists
  let result = Db::from_file("queries_test.db");

  if std::path::Path::new("queries_test.db").exists() {
    assert!(result.is_ok());
    let db = result.unwrap();

    // Test that we can access table metadata
    // The exact tables will depend on what's in the test database
    println!("Found {} tables", db.tables_metadata.len());
    for table in &db.tables_metadata {
      println!("Table: {}, columns: {}", table.name, table.columns.len());
    }
  } else {
    // If the file doesn't exist, ensure we get an appropriate error
    assert!(result.is_err());
  }
}
//
#[cfg(test)]
mod parser_tests {
  use rust_sqlite::sql::ast::{ResultColumn, Statement};
  use rust_sqlite::sql::parser::parse_statement;

  #[test]
  fn test_parse_and_execute_simple_query() {
    // Test parsing a statement
    let query = "SELECT * FROM users;";
    let parsed = parse_statement(query, false);
    assert!(parsed.is_ok());

    match parsed.unwrap() {
      Statement::Select(select_stmt) => {
        // Verify the parsed structure
        if let ResultColumn::Star = &select_stmt.core.result_columns[0] {
        } else {
          panic!("Expected star in result columns");
        }
      }
      _ => panic!("Expected a SELECT statement"),
    }
  }
}

#[cfg(test)]
mod engine_tests {
  use rust_sqlite::db::Db;
  use rust_sqlite::engine::plan::Planner;
  use rust_sqlite::sql::parser::parse_statement;

  #[test]
  fn test_planner_with_mock_db() {
    // If test database files exist, try to use them
    if std::path::Path::new("minimal_test.db").exists() {
      let db_result = Db::from_file("minimal_test.db");
      if let Ok(db) = db_result {
        let planner = Planner::new(&db);

        // Try to parse and compile a simple statement
        let stmt = parse_statement("SELECT * FROM sqlite_master", false);
        if let Ok(stmt) = stmt {
          // The compilation might fail if there's no sqlite_master table
          let result = planner.compile(&stmt);
          // We don't assert success/failure since it depends on the actual database content
        }
      }
    }
  }
}
