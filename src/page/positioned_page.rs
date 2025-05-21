use super::page_utils::{Cell, Page, PageType};


#[derive(Debug)]
pub struct PositionedPage {
    pub page: Page,
    pub cell: usize,
}

impl PositionedPage {
    pub fn next_cell(&mut self) -> Option<&Cell>{
        let cell = self.page.get(self.cell);
        self.cell += 1;
        cell
    }

    pub fn next_page(&mut self) -> Option<u32>{
        if self.page.header.page_type == PageType::TableInterior && self.cell == self.page.cells.len() {
            self.cell += 1;
            self.page.header.rightmost_pointer
        } else { None}
    }

}
