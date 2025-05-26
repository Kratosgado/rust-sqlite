use std::{
    collections::HashMap,
    io::{Read, Seek, SeekFrom},
    sync::{Arc, Mutex, RwLock},
};

use anyhow::{Context, Ok};

use crate::dbheader::DbHeader;

use super::page_utils::{self, Cell, Page, PageHeader, PageType, TableLeafCell};

pub const HEADER_SIZE: usize = 100;
const HEADER_PREFIX: &[u8] = b"SQLite format 3\0";
const HEADER_PAGE_SIZE_OFFSET: usize = 16;

const PAGE_LEAF_HEADER_SIZE: usize = 8;
const PAGE_FIRST_FREEBLOCK_OFFSET: usize = 1;
const PAGE_CELL_COUNT_OFFSET: usize = 3;
const PAGE_CELL_CONTENT_OFFSET: usize = 5;
const PAGE_FRAGMENTED_BYTES_COUNT_OFFSET: usize = 7;

const PAGE_MAX_SIZE: u32 = 65536;

const PAGE_LEAF_TABLE_ID: u8 = 0x0d;
const PAGE_INTERIO_TABLE_ID: u8 = 0x05;

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
            pages: Arc::new(RwLock::new(HashMap::new())),
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

        Ok(Arc::new(parse_page(&buffer, n)?))
    }
}

/// The header starts with the magic string 'SQLite format 3\0'
/// followed by the page size encoded as a big-endian 2-byte integer at offset 16
pub fn parse_header(buffer: &[u8]) -> anyhow::Result<DbHeader> {
    if !buffer.starts_with(HEADER_PREFIX) {
        let prefix = String::from_utf8_lossy(&buffer[..HEADER_PREFIX.len()]);
        anyhow::bail!("Invalid header prefix: {prefix}");
    }

    let page_size_raw = read_be_word_at(buffer, HEADER_PAGE_SIZE_OFFSET);
    let page_size = match page_size_raw {
        1 => PAGE_MAX_SIZE,
        n if n.is_power_of_two() => n as u32,
        _ => anyhow::bail!("page size is not a power of 2: {}", page_size_raw),
    };

    Ok(DbHeader { page_size })
}

fn read_be_word_at(input: &[u8], offset: usize) -> u16 {
    u16::from_be_bytes(input[offset..offset + 2].try_into().unwrap())
}

// TODO: verify validity
fn read_be_double_at(input: &[u8], offset: usize) -> u32 {
    u32::from_be_bytes(input[offset..offset + 4].try_into().unwrap())
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
//
// fn parse_table_leaf_page(buffer: &[u8], ptr_offset: u16) -> anyhow::Result<Cell> {
//     let header = parse_page_header(buffer)?;
//
//     let content_buffer = &buffer[PAGE_LEAF_HEADER_SIZE..];
//     let cell_pointers = parse_cell_pointers(content_buffer, header.cell_count as usize, ptr_offset);
//
//     let cells = cell_pointers
//         .iter()
//         .map(|&ptr| parse_table_leaf_cell(&buffer[ptr as usize..]))
//         .collect::<anyhow::Result<Vec<TableLeafCell>>>()?;
//
//     Ok(Page::TableLeaf(TableLeafPage {
//         header,
//         cell_pointers,
//         cells,
//     }))
// }

fn parse_page_header(buffer: &[u8]) -> anyhow::Result<PageHeader> {
    let (page_type, has_rightmost_ptr) = match buffer[0] {
        PAGE_LEAF_TABLE_ID => (PageType::TableLeaf, false),
        PAGE_INTERIO_TABLE_ID => (PageType::TableInterior, true),
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
        Some(read_be_double_at(
            buffer,
            PAGE_FRAGMENTED_BYTES_COUNT_OFFSET,
        ))
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

pub fn read_varint_at(buffer: &[u8], mut offset: usize) -> (u8, i64) {
    let mut size = 0;
    let mut result = 0;

    while size < 8 && buffer[offset] >= 0b1000_0000 {
        result |= ((buffer[offset] as i64) & 0b0111_1111) << (7 * size);
        offset += 1;
        size += 1;
    }

    result |= (buffer[offset] as i64) << (7 * size);
    (size + 1, result)
}
