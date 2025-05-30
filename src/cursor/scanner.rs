use crate::{
    page::{page_utils::Cell, positioned_page::PositionedPage},
    pager::Pager,
};

use super::{cursor::Cursor, record::parse_record_header};

#[derive(Debug)]
enum ScannerElem {
    Page(u32),
    Cursor(Cursor),
}

#[derive(Debug)]
pub struct Scanner {
    pager: Pager,
    inital_page: usize,
    page_stack: Vec<PositionedPage>,
}

impl Scanner {
    pub fn new(pager: Pager, page: usize) -> Self {
        Self {
            pager,
            inital_page: page,
            page_stack: vec![],
        }
    }

    pub fn next_record(&mut self) -> anyhow::Result<Option<Cursor>> {
        loop {
            match self.next_elem() {
                Ok(Some(ScannerElem::Cursor(cursor))) => return Ok(Some(cursor)),
                Ok(Some(ScannerElem::Page(page_num))) => {
                    let new_page = self.pager.read_page(page_num as usize)?.clone();
                    self.page_stack.push(PositionedPage {
                        page: new_page,
                        cell: 0,
                    });
                }
                Ok(None) if self.page_stack.len() > 1 => {
                    self.page_stack.pop();
                }
                Ok(None) => return Ok(None),
                Err(e) => return Err(e),
            }
        }
    }

    fn next_elem(&mut self) -> anyhow::Result<Option<ScannerElem>> {
        let Some(page) = self.current_page()? else {
            return Ok(None);
        };

        // for overflow maybe
        if let Some(page) = page.next_page() {
            return Ok(Some(ScannerElem::Page(page)));
        }

        let Some(cell) = page.next_cell() else {
            return Ok(None);
        };

        match cell {
            Cell::TableLeaf(cell) => {
                let header = parse_record_header(&cell.payload)?;
                Ok(Some(ScannerElem::Cursor(Cursor {
                    header,
                    payload: cell.payload.clone(),
                })))
            }
            Cell::TableInterior(cell) => Ok(Some(ScannerElem::Page(cell.left_child_page))),
            c => unimplemented!("Scannine {c:?} not implemented"),
        }
    }

    fn current_page(&mut self) -> anyhow::Result<Option<&mut PositionedPage>> {
        if self.page_stack.is_empty() {
            let page = match self.pager.read_page(self.inital_page) {
                Ok(page) => page.clone(),
                Err(e) => return Err(e),
            };

            self.page_stack.push(PositionedPage { page, cell: 0 });
        }
        Ok(self.page_stack.last_mut())
    }
}
