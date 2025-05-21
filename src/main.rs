use std::io::{stdin, stdout, BufRead, Write};

use anyhow::Context;
use rust_sqlite::db::Db;

fn main() -> anyhow::Result<()> {
    let database = Db::from_file(std::env::args().nth(1).context("missing db file")?)?;

    cli(database)
}

fn cli(mut db: Db) -> anyhow::Result<()> {
    print_flushed("rqlite> ")?;

    let mut line_buffer = String::new();

    while stdin().lock().read_line(&mut line_buffer).is_ok() {
        match line_buffer.trim() {
            ".exit" => break,
            ".tables" => display_tables(&mut db)?,
            _ => println!("Unrecognised command '{}'", line_buffer.trim()),
        }

        print_flushed("\nrqlite> ")?;
        line_buffer.clear();
    }
    Ok(())
}

fn display_tables(db: &mut Db) -> anyhow::Result<()> {
    let mut scanner = db.scanner(1);

    while let Some(Ok(mut record)) = scanner.next_record() {
        let type_value = record
            .field(0)
            .context("missing type field")
            .context("invalid type field")?;

        if type_value.as_str() == Some("table") {
            let name_value = record
                .field(1)
                .context("missing name field")
                .context("invalid name field")?;

            print!("{} ", name_value.as_str().unwrap());
        }
    }

    Ok(())
}

fn print_flushed(s: &str) -> anyhow::Result<()> {
    print!("{}", s);
    stdout().flush().context("flush stdout")
}
