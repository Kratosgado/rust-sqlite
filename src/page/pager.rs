use std::{
    collections::HashMap,
    io::{Read, Seek, SeekFrom},
};

use anyhow::{Context, Ok};

use crate::dbheader::DbHeader;

use super::page_utils::{Page, PageHeader, PageType, TableLeafCell, TableLeafPage};

pub const HEADER_SIZE: usize = 100;
const HEADER_PREFIX: &[u8] = b"SQLite format 3\0";
const HEADER_PAGE_SIZE_OFFSET: usize = 16;

const PAGE_LEAF_HEADER_SIZE: usize = 8;
const PAGE_FIRST_FREEBLOCK_OFFSET: usize = 1;
const PAGE_CELL_COUNT_OFFSET: usize = 3;
const PAGE_CELL_CONTENT_OFFSET: usize = 5;
const PAGE_FRAGMENTED_BYTES_COUNT_OFFSET: usize = 7;

const PAGE_MAX_SIZE: u32 = 65536;

/// pager reads and caches pages from the db file
#[derive(Debug)]
pub struct Pager<I: Read + Seek = std::fs::File> {
    input: I,
    page_size: usize,
    pages: HashMap<usize, Page>,
}

impl<I: Read + Seek> Pager<I> {
    pub fn new(input: I, page_size: usize) -> Self {
        Self {
            input,
            page_size,
            pages: HashMap::new(),
        }
    }

    pub fn read_page(&mut self, n: usize) -> anyhow::Result<&Page> {
        if self.pages.contains_key(&n) {
            return Ok(self.pages.get(&n).unwrap());
        }

        let page = self.load_page(n)?;
        self.pages.insert(n, page);
        Ok(self.pages.get(&n).unwrap())
    }

    fn load_page(&mut self, n: usize) -> anyhow::Result<Page> {
        let offset = n.saturating_sub(1) * self.page_size;

        self.input
            .seek(SeekFrom::Start(offset as u64))
            .context("seek to page start")?;

        let mut buffer = vec![0; self.page_size];
        self.input.read_exact(&mut buffer).context("read page")?;

        parse_page(&buffer, n)
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

fn parse_page(buffer: &[u8], page_num: usize) -> anyhow::Result<Page> {
    let ptr_offset = if page_num == 1 { HEADER_SIZE as u16 } else { 0 };

    match buffer[0] {
        PAGE_LEAF_TABLE_ID => parse_table_leaf_page(buffer, ptr_offset),
        _ => Err(anyhow::anyhow!("Unknown page type: {}", buffer[0])),
    }
}

fn parse_table_leaf_page(buffer: &[u8], ptr_offset: u16) -> anyhow::Result<Page> {
    let header = parse_page_header(buffer)?;

    let content_buffer = &buffer[PAGE_LEAF_HEADER_SIZE..];
    let cell_pointers = parse_cell_pointers(content_buffer, header.cell_count as usize, ptr_offset);

    let cells = cell_pointers
        .iter()
        .map(|&ptr| parse_table_leaf_cell(&buffer[ptr as usize..]))
        .collect::<anyhow::Result<Vec<TableLeafCell>>>()?;

    Ok(Page::TableLeaf(TableLeafPage {
        header,
        cell_pointers,
        cells,
    }))
}

fn parse_page_header(buffer: &[u8]) -> anyhow::Result<PageHeader> {
    let page_type = match buffer[0] {
        0x0d => PageType::TableLeaf,
        _ => anyhow::bail!("unknown page type: {}", buffer[0]),
    };

    let first_freeblock = read_be_word_at(buffer, PAGE_FIRST_FREEBLOCK_OFFSET);
    let cell_count = read_be_word_at(buffer, PAGE_CELL_COUNT_OFFSET);
    let cell_content_offset = match read_be_word_at(buffer, PAGE_CELL_CONTENT_OFFSET) {
        0 => 65536,
        n => n as u32,
    };

    let fragmented_bytes_count = buffer[PAGE_FRAGMENTED_BYTES_COUNT_OFFSET];

    Ok(PageHeader {
        page_type,
        first_freeblock,
        cell_count,
        cell_content_offset,
        fragmented_bytes_count,
    })
}

fn parse_cell_pointers(buffer: &[u8], n: usize, ptr_offset: u16) -> Vec<u16> {
    let mut pointers = Vec::with_capacity(n);
    for i in 0..n {
        pointers.push(read_be_word_at(buffer, 2 * i) - ptr_offset);
    }
    pointers
}

fn parse_table_leaf_cell(mut buffer: &[u8]) -> anyhow::Result<TableLeafCell> {
    let (n, size) = read_varint_at(buffer, 0);
    buffer = &buffer[n as usize..];

    let (n, row_id) = read_varint_at(buffer, 0);
    buffer = &buffer[n as usize..];

    let payload = buffer[..size as usize].to_vec();
    Ok(TableLeafCell {
        size,
        row_id,
        payload,
    })
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
