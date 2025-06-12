use std::io::{stdin, stdout, BufRead, Write};

use anyhow::Context;
use rust_sqlite::{db::Db, engine, sql};

fn main() -> anyhow::Result<()> {
    let database = Db::from_file(std::env::args().nth(1).context("missing db file")?)?;
    cli(database)
}

fn cli(mut db: Db) -> anyhow::Result<()> {
    print_flushed("rqlite> ")?;

    let mut line_buffer = String::new();

    while stdin().lock().read_line(&mut line_buffer).is_ok() {
        match line_buffer.trim() {
            ".help" => display_help(),
            ".exit" => break,
            ".tables" => display_tables(&mut db)?,
            stmt => eval_query(&db, stmt).unwrap_or_else(|e| println!("Error: {e}")),
        }

        print_flushed("\nrqlite> ")?;
        line_buffer.clear();
    }
    Ok(())
}

fn eval_query(db: &Db, query: &str) -> anyhow::Result<()> {
    let parsed_query = sql::parser::parse_statement(query, false)?;
    // println!("{parsed_query:?}");
    let mut op = engine::plan::Planner::new(db).compile(&parsed_query)?;
    println!("{op:?}");

    while let Some(values) = op.next_row()? {
        println!("{values:?}");
        let formated = values
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join("\t| ");

        println!("{formated}");
    }

    Ok(())
}

fn display_tables(db: &mut Db) -> anyhow::Result<()> {
    for table in &db.tables_metadata {
        print!("{} ", &table.name)
    }

    Ok(())
}

fn display_help() {
    print!(
        "MAN PAGE!\n
        .tables -- display tables.
        .help -- display help.
        .exit -- exit REPL "
    )
}

fn print_flushed(s: &str) -> anyhow::Result<()> {
    print!("{}", s);
    stdout().flush().context("flush stdout")
}
