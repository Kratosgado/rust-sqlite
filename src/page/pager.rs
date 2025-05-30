use std::{
    collections::HashMap,
    io::{Read, Seek, SeekFrom},
    sync::{Arc, Mutex, RwLock},
};

use anyhow::{Context, Ok};

use crate::{
    dbheader::{HEADER_SIZE, PAGE_MAX_SIZE},
    read_be_double_at, read_be_word_at, read_varint_at,
};

use super::page_utils::{self, Cell, Page, PageHeader, PageType, TableLeafCell};

pub const PAGE_FIRST_FREEBLOCK_OFFSET: usize = 1;
pub const PAGE_CELL_COUNT_OFFSET: usize = 3;
pub const PAGE_CELL_CONTENT_OFFSET: usize = 5;
pub const PAGE_FRAGMENTED_BYTES_COUNT_OFFSET: usize = 7;
pub const PAGE_LEAF_HEADER_SIZE: usize = 8;

const PAGE_INTERIOR_INDEX_ID: u8 = 0x02;
const PAGE_INTERIROR_TABLE_ID: u8 = 0x05;
const PAGE_LEAF_INDEX_ID: u8 = 0x0a;
const PAGE_LEAF_TABLE_ID: u8 = 0x0d;

/// pager reads and caches pages from the db file
#[derive(Debug)]
pub struct Pager<I: Read + Seek = std::fs::File> {
    input: Arc<Mutex<I>>,
    page_size: usize,
    pages: Arc<RwLock<HashMap<usize, Arc<Page>>>>,
}

impl<I: Read + Seek> Pager<I> {
    pub fn new(input: I, page_size: usize) -> Self {
        Self {
            input: Arc::new(Mutex::new(input)),
            page_size,
            pages: Arc::default(),
        }
    }

    pub fn read_page(&self, n: usize) -> anyhow::Result<Arc<Page>> {
        {
            let read_pages = self
                .pages
                .read()
                .map_err(|_| anyhow::anyhow!("failed to acquire pager read lock"))?;
            if let Some(page) = read_pages.get(&n) {
                return Ok(page.clone());
            }
        }

        let mut write_pages = self
            .pages
            .write()
            .map_err(|_| anyhow::anyhow!("failed to acquire pager write lock"))?;

        if let Some(page) = write_pages.get(&n) {
            return Ok(page.clone());
        }

        let page = self.load_page(n)?;
        write_pages.insert(n, page.clone());
        Ok(page)
    }

    fn load_page(&self, n: usize) -> anyhow::Result<Arc<Page>> {
        let offset = n.saturating_sub(1) * self.page_size;

        let mut input_guard = self
            .input
            .lock()
            .map_err(|_| anyhow::anyhow!("failed to lock pager mutex"))?;

        input_guard
            .seek(SeekFrom::Start(offset as u64))
            .context("seek to page start")?;

        let mut buffer = vec![0; self.page_size];
        input_guard.read_exact(&mut buffer).context("read page")?;
        // println!("page({n}): {buffer:?}");
        // let page = parse_page(&buffer, n)?;
        // println!("{page:?}");

        Ok(Arc::new(parse_page(&buffer, n)?))
    }
}

impl Clone for Pager {
    fn clone(&self) -> Self {
        Self {
            input: self.input.clone(),
            page_size: self.page_size,
            pages: self.pages.clone(),
        }
    }
}

fn parse_page(buffer: &[u8], page_num: usize) -> anyhow::Result<Page> {
    let ptr_offset = if page_num == 1 { HEADER_SIZE as u16 } else { 0 };
    let content_buffer = &buffer[ptr_offset as usize..];
    let header = parse_page_header(content_buffer)?;
    let cell_pointers = parse_cell_pointers(
        &content_buffer[header.byte_size()..],
        header.cell_count as usize,
        ptr_offset,
    );

    let cells_parsing_fn = match header.page_type {
        PageType::TableLeaf => parse_table_leaf_cell,
        PageType::TableInterior => parse_table_interior_cell,
        _ => unimplemented!("parsing index is not implemented"),
    };

    let cells = parse_cells(content_buffer, &cell_pointers, cells_parsing_fn)?;

    Ok(Page {
        header,
        cell_pointers,
        cells,
    })
}

fn parse_cells(
    buffer: &[u8],
    cell_pointers: &[u16],
    parse_fn: impl Fn(&[u8]) -> anyhow::Result<Cell>,
) -> anyhow::Result<Vec<Cell>> {
    cell_pointers
        .iter()
        .map(|&ptr| parse_fn(&buffer[ptr as usize..]))
        .collect()
}

fn parse_page_header(buffer: &[u8]) -> anyhow::Result<PageHeader> {
    let (page_type, has_rightmost_ptr) = match buffer[0] {
        PAGE_LEAF_TABLE_ID => (PageType::TableLeaf, false),
        PAGE_INTERIROR_TABLE_ID => (PageType::TableInterior, true),
        PAGE_INTERIOR_INDEX_ID => (PageType::IndexInterior, true),
        PAGE_LEAF_INDEX_ID => (PageType::IndexLeaf, false),
        _ => anyhow::bail!("unknown page type: {}", buffer[0]),
    };

    let first_freeblock = read_be_word_at(buffer, PAGE_FIRST_FREEBLOCK_OFFSET);
    let cell_count = read_be_word_at(buffer, PAGE_CELL_COUNT_OFFSET);
    let cell_content_offset = match read_be_word_at(buffer, PAGE_CELL_CONTENT_OFFSET) {
        0 => PAGE_MAX_SIZE,
        n => n as u32,
    };

    let fragmented_bytes_count = buffer[PAGE_FRAGMENTED_BYTES_COUNT_OFFSET];
    let rightmost_pointer = if has_rightmost_ptr {
        Some(read_be_double_at(buffer, PAGE_LEAF_HEADER_SIZE))
    } else {
        None
    };

    Ok(PageHeader {
        page_type,
        first_freeblock,
        cell_count,
        cell_content_offset,
        fragmented_bytes_count,
        rightmost_pointer,
    })
}

fn parse_cell_pointers(buffer: &[u8], n: usize, ptr_offset: u16) -> Vec<u16> {
    let mut pointers = Vec::with_capacity(n);
    for i in 0..n {
        pointers.push(read_be_word_at(buffer, 2 * i) - ptr_offset);
    }
    pointers
}

fn parse_table_leaf_cell(mut buffer: &[u8]) -> anyhow::Result<page_utils::Cell> {
    let (n, size) = read_varint_at(buffer, 0);
    buffer = &buffer[n as usize..];

    let (n, row_id) = read_varint_at(buffer, 0);
    buffer = &buffer[n as usize..];

    let payload = buffer[..size as usize].to_vec();
    Ok(TableLeafCell {
        size,
        row_id,
        payload,
    }
    .into())
}

fn parse_table_interior_cell(mut buffer: &[u8]) -> anyhow::Result<page_utils::Cell> {
    let left_child_page = read_be_double_at(buffer, 0);
    buffer = &buffer[4..];

    let (_, key) = read_varint_at(buffer, 0);

    Ok(page_utils::TableInteriorCell {
        left_child_page,
        key,
    }
    .into())
}
