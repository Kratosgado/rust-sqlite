# Rust-Sqlite

## Project Overview

This project is a lightweight, from-scratch implementation of a SQLite-like database written in Rust. It is a command-line interface (CLI) tool that allows users to interact with a database file. The project is self-contained and does not have any external dependencies besides the `anyhow` crate for error handling.

The core architecture is composed of the following modules:

- **`db`**: The top-level module that orchestrates the database operations.
- **`dbheader`**: Handles parsing of the database header.
- **`page`**: Manages the reading and writing of pages from the database file.
- **`cursor`**: Provides a way to traverse the data in the database.
- **`sql`**: Contains the SQL parser and abstract syntax tree (AST).
- **`engine`**: Implements the query execution engine.

## Building and Running

To build and run the project, you can use the following `cargo` commands:

- **Build:**

  ```bash
  cargo build
  ```

- **Run:**

  ```bash
  cargo run <database_file>
  ```

  Replace `<database_file>` with the path to a database file. Two database files are provided in the root of the project for testing: `minimal_test.db` and `queries_test.db`.

- **Tests:**

  ```bash
  cargo test
  ```

## Query Execution

The query execution process can be broken down into the following steps:

1. **Tokenization**: The input SQL string is converted into a stream of tokens by the `tokenize` function in `src/sql/tokenizer.rs`.
2. **Parsing**: The `parse_statement` function in `src/sql/parser.rs` consumes the tokens and builds an abstract syntax tree (AST).
3. **Planning**: The `Planner` in `src/engine/plan.rs` takes the AST and creates a query plan, which is a tree of `Operator`s.
4. **Execution**: The execution engine traverses the query plan and calls the `next_row` method of each operator to get the next row of data.

## Extending the Database

To extend the database with new SQL features, you will need to modify the following files:

- **`src/sql/tokenizer.rs`**: Add new tokens for the new SQL keywords or operators.
- **`src/sql/parser.rs`**: Add new parsing functions for the new SQL statements or expressions.
- **`src/sql/ast.rs`**: Add new AST nodes for the new SQL constructs.
- **`src/engine/plan.rs`**: Add new planning logic for the new SQL statements or expressions.
- **`src/engine/operator.rs`**: Add new operators for the new SQL operations.

## Development Conventions

- **Code Style**: The project uses `rustfmt` to enforce a consistent code style. The configuration can be found in the `rustfmt.toml` file.
- **Testing**: The project has a suite of tests in the `tests/` directory. The tests cover different aspects of the database, including the cursor, header, lexer, and parser.
- **Error Handling**: The project uses the `anyhow` crate for error handling. This provides a convenient way to propagate and handle errors throughout the application.
