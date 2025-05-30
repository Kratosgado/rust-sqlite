#[derive(Debug, Clone)]
pub struct TableLeafCell {
    pub size: i64,
    pub row_id: i64,
    pub payload: Vec<u8>,
    pub overflow_page_num: Option<u32>,
}

#[derive(Debug, Clone)]
pub struct TableInteriorCell {
    pub left_child_page: u32,
    pub key: i64,
}

#[derive(Debug, Clone)]
pub struct IndexLeafCell {
    pub size: i64,
    pub payload: Vec<u8>,
    pub overflow_page_num: Option<u32>,
}

#[derive(Debug, Clone)]
pub struct IndexInteriorCell {
    pub left_child_page: u32,
    pub size: i64,
    pub payload: Vec<u8>,
    pub overflow_page_num: Option<u32>,
}

#[derive(Debug, Clone)]
pub enum Cell {
    TableLeaf(TableLeafCell),
    TableInterior(TableInteriorCell),
    IndexLeaf(IndexLeafCell),
    IndexInterior(IndexInteriorCell),
}

#[derive(Debug, Clone)]
pub struct Page {
    pub header: PageHeader,
    pub cell_pointers: Vec<u16>,
    pub cells: Vec<Cell>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PageType {
    TableLeaf,
    TableInterior,
    IndexLeaf,
    IndexInterior,
}

#[derive(Debug, Clone, Copy)]
pub struct PageHeader {
    /// b-tree page type
    pub page_type: PageType,
    /// Gives the start of the first freeblock on the page,
    /// or is 0 if there are no freeblocks
    pub first_freeblock: u16,
    pub cell_count: u16,
    /// start of the cell content. 0 value indicates 65536
    pub cell_content_offset: u32,
    /// Number of fragmented free bytes within the cell content aread.
    pub fragmented_bytes_count: u8,
    /// Apperas in the header of interior b-tree pages only
    pub rightmost_pointer: Option<u32>,
}

impl PageHeader {
    pub fn byte_size(&self) -> usize {
        if self.rightmost_pointer.is_some() {
            12
        } else {
            8
        }
    }
}

impl From<TableLeafCell> for Cell {
    fn from(cell: TableLeafCell) -> Self {
        Self::TableLeaf(cell)
    }
}
impl From<TableInteriorCell> for Cell {
    fn from(cell: TableInteriorCell) -> Self {
        Self::TableInterior(cell)
    }
}

impl From<IndexLeafCell> for Cell {
    fn from(cell: IndexLeafCell) -> Self {
        Self::IndexLeaf(cell)
    }
}

impl From<IndexInteriorCell> for Cell {
    fn from(cell: IndexInteriorCell) -> Self {
        Self::IndexInterior(cell)
    }
}
