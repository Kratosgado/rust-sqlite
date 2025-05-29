use std::{io::Read, path::Path, usize};

use anyhow::Context;

use crate::{
    cursor::{cursor::Cursor, scanner::Scanner},
    dbheader::{self, DbHeader},
    pager::Pager,
    sql::{self, ast},
};

pub struct Db {
    pub header: DbHeader,
    pub tables_metadata: Vec<TableMetadata>,
    pager: Pager,
}

#[derive(Debug, Clone)]
pub struct TableMetadata {
    pub name: String,
    pub columns: Vec<ast::ColumnDef>,
    pub first_page: usize,
}

impl Db {
    pub fn from_file(filename: impl AsRef<Path>) -> anyhow::Result<Self> {
        let mut file = std::fs::File::open(filename.as_ref()).context("open db file")?;

        let mut header_buffer = [0; dbheader::HEADER_SIZE];
        file.read_exact(&mut header_buffer)
            .context("read db header")?;

        let header = dbheader::parse_header(&header_buffer).context("parse db header")?;

        let pager = Pager::new(file, header.page_size as usize);
        let tables_metadata = Self::collect_tables_metadata(pager.clone())?;
        Ok(Self {
            header,
            pager,
            tables_metadata,
        })
    }

    pub fn scanner(&self, page: usize) -> Scanner {
        Scanner::new(self.pager.clone(), page)
    }

    fn collect_tables_metadata(pager: Pager) -> anyhow::Result<Vec<TableMetadata>> {
        let mut metadata = vec![];
        let mut scanner = Scanner::new(pager, 1);

        while let Some(record) = scanner.next_record()? {
            if let Some(m) = TableMetadata::from_cursor(&record)? {
                metadata.push(m);
            }
        }
        Ok(metadata)
    }
}

impl TableMetadata {
    fn from_cursor(cursor: &Cursor) -> anyhow::Result<Option<Self>> {
        let type_value = cursor
            .field(0)
            .context("missing type field")
            .context("invalid type field")?;
        if type_value.as_str() != Some("table") {
            return Ok(None);
        }

        let create_stmt = cursor
            .field(4)
            .context("missing create statement")
            .context("invalid create statement")?
            .as_str()
            .context("table create statement should be a string")?
            .to_owned();

        let create = sql::parser::parse_create_statement(&create_stmt)?;

        let first_page = cursor
            .field(3)
            .context("missing table first page")?
            .as_int()
            .context("table first page should be an integer")? as usize;
        println!("type_value: {type_value:?}");
        println!("create_stmt: {create_stmt:?}");
        println!("create: {create:?}");
        println!("first_page: {first_page:?}");

        Ok(Some(TableMetadata {
            name: create.name,
            columns: create.columns,
            first_page,
        }))
    }
}
