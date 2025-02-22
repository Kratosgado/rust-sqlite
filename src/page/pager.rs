use std::u16;

use crate::dbheader::DbHeader;

use super::page_utils::{Page, PageHeader, PageType};

pub const HEADER_SIZE: usize = 100;
const HEADER_PREFIX: &[u8] = b"SQLite format 3\0";
const HEADER_PAGE_SIZE_OFFSET: usize = 16;

const PAGE_LEAF_HEADER_SIZE: usize = 8;
const PAGE_FIRST_FREEBLOCK_OFFSET: usize = 1;
const PAGE_CELL_COUNT_OFFSET: usize = 3;
const PAGE_CELL_CONTENT_OFFSET: usize = 5;
const PAGE_FRAGMENTED_BYTES_COUNT_OFFSET: usize = 7;

const PAGE_MAX_SIZE: u32 = 65536;

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
        PAGE_LEAF_TABLE_ID => parse
    }
}

fn parse_table_leaf_page(buffer: &[u8], ptr_offset: u16) -> anyhow::Result<Page> {
    let header = parse_page_header(buffer)?;

    let content_buffer = &buffer[PAGE_LEAF_HEADER_SIZE..];
    let cell_pointers = pa
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

    Ok(PageHeader{
        page_type, first_freeblock, cell_count, cell_content_offset, fragmented_bytes_count
    })
}

fn parse_cell_pointers(buffer: &[u8], n: usize, ptr_offset: u16) -> Vec<u16> {
    let mut pointers = Vec::with_capacity(n);
    for i  in 0..n {
        pointers.push(read_be_word_at(buffer, 2 * i) -ptr_offset);
    }
    pointers
}
