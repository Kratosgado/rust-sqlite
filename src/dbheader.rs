use crate::{read_be_byte_at, read_be_double_at, read_be_word_at};

#[derive(Debug, Copy, Clone)]
pub struct DbHeader {
    pub page_size: u32,
    pub file_format_w: u8,
    pub file_format_r: u8,
    pub max_embedded_payload: u8,
    pub min_embedded_payload: u8,
    pub leaf_payload_fraction: u8,
    pub file_change_counter: u32,
    pub db_size: u32,
    pub schema_cookie: u32,
    pub sq_version: u32,
}

const HEADER_PREFIX: &[u8] = b"SQLite format 3\0";
const HEADER_PAGE_SIZE_OFFSET: usize = 16;
const FILE_FORMAT_W_OFFSET: usize = 18;
const FILE_FORMAT_R_OFFSET: usize = 19;
const MAX_EMBEDDED_PAYLOAD_OFFSET: usize = 21;
const MIN_EMBEDDED_PAYLOAD_OFFSET: usize = 22;
const LEAF_PAYLOAD_FRACTION_OFFSET: usize = 23;
const FILE_CHANGE_COUNTER_OFFSET: usize = 24;
const DB_SIZE_OFFSET: usize = 28;
const SCHEMA_COOKIE_OFFSET: usize = 32;
const SQ_VERSION_OFFSET: usize = 96;
pub const PAGE_MAX_SIZE: u32 = 65536;
pub const HEADER_SIZE: usize = 100;

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
    let file_format_w = read_be_byte_at(buffer, FILE_FORMAT_W_OFFSET);
    let file_format_r = read_be_byte_at(buffer, FILE_FORMAT_R_OFFSET);
    let max_embedded_payload = read_be_byte_at(buffer, MAX_EMBEDDED_PAYLOAD_OFFSET);
    let min_embedded_payload = read_be_byte_at(buffer, MIN_EMBEDDED_PAYLOAD_OFFSET);
    let leaf_payload_fraction = read_be_byte_at(buffer, LEAF_PAYLOAD_FRACTION_OFFSET);
    let file_change_counter = read_be_double_at(buffer, FILE_CHANGE_COUNTER_OFFSET);
    let db_size = read_be_double_at(buffer, DB_SIZE_OFFSET);
    let schema_cookie = read_be_double_at(buffer, SCHEMA_COOKIE_OFFSET);
    let sq_version = read_be_double_at(buffer, SQ_VERSION_OFFSET);

    Ok(DbHeader {
        page_size,
        file_format_r,
        file_format_w,
        max_embedded_payload,
        min_embedded_payload,
        leaf_payload_fraction,
        file_change_counter,
        db_size,
        schema_cookie,
        sq_version,
    })
}
