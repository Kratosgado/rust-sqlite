use crate::{page::page_utils::Page, pager::Pager};

use super::{cursor::Cursor, record};

#[derive(Debug)]
pub struct Scanner<'p> {
    pager: &'p mut Pager,
    page: usize,
    cell: usize,
}

impl<'p> Scanner<'p> {
    pub fn new(pager: &'p mut Pager, page: usize) -> Self {
        Self {
            pager,
            page,
            cell: 0,
        }
    }

    pub fn next_record(&mut self) -> Option<anyhow::Result<Cursor>> {
        let page = match self.pager.read_page(self.page) {
            Ok(page) => page,
            Err(e) => return Some(Err(e)),
        };

        match page {
            Page::TableLeaf(leaf) => {
                let cell = leaf.cells.get(self.cell)?;

                let header = match record::parse_record_header(&cell.payload) {
                    Ok(header) => header,
                    Err(e) => return Some(Err(e)),
                };

                let record = Cursor {
                    header,
                    pager: self.pager,
                    page_index: self.page,
                    page_cell: self.cell,
                };

                self.cell += 1;
                Some(Ok(record))
            }
        }
    }
}
