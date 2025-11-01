#[cfg(test)]
mod integration {
  use rust_sqlite::db::Db;
  use rust_sqlite::engine::plan::Planner;
  use rust_sqlite::sql::parser::parse_statement;

  const USER_QUERY: &str = "SELECT * FROM users;";
  const USER_WHERE_QUERY: &str = "SELECT * FROM users where id = 3;";

  #[test]
  fn read_existing_database() {
    let result = Db::from_file("no.db");

    if std::path::Path::new("no.db").exists() {
      assert!(result.is_ok());
    } else {
      // If the file doesn't exist, ensure we get an appropriate error
      assert!(result.is_err());
    }
  }

  #[test]
  fn read_queries_database() {
    // Test reading the queries_test.db file if it exists
    let result = Db::from_file("queries_test.db");

    if std::path::Path::new("queries_test.db").exists() {
      assert!(result.is_ok());
      let _db = result.unwrap();

      // Test that we can access table metadata
      // The exact tables will depend on what's in the test database
      // println!("Found {} tables", db.tables_metadata.len());
      // for table in &db.tables_metadata {
      //   println!("Table: {}, columns: {}", table.name, table.columns.len());
      // }
    } else {
      // If the file doesn't exist, ensure we get an appropriate error
      assert!(result.is_err());
    }
  }

  #[test]
  fn execute_simple_query() {
    execute_query(USER_QUERY);
  }

  #[test]
  fn execute_simple_where_query() {
    execute_query(USER_WHERE_QUERY);
  }

  #[test]
  fn execute_where_compound_query() {
    const USER_WHERE_COMPOUND_QUERY: &str = "SELECT * FROM users where id = 3 and name = 'prince';";
    execute_query(USER_WHERE_COMPOUND_QUERY);
  }

  fn execute_query(query: &str) {
    println!("{query}");
    let db = &Db::from_file("queries_test.db").unwrap();
    // Test parsing a statement
    let parsed = &parse_statement(query, false).unwrap();
    println!("{parsed:?}");
    let op = Planner::new(db).compile(parsed);
    assert!(op.is_ok());
    let mut op = op.unwrap();

    loop {
      let next = op.next_row();
      assert!(next.is_ok());
      let Some(values) = next.unwrap() else {
        break;
      };
      let formated = values
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join("\t| ");

      println!("{formated}");
    }
  }
}
