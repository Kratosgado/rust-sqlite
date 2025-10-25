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

// #[cfg(test)]
// mod tests {
//     use super::*;
//
//     #[test]
//     fn test_parse_valid_header() {
//         let mut buffer = [0u8; HEADER_SIZE];
//         // Write the SQLite header prefix
//         buffer[0..16].copy_from_slice(b"SQLite format 3\0");
//
//         // Write a valid page size (4096 = 0x1000)
//         buffer[HEADER_PAGE_SIZE_OFFSET..HEADER_PAGE_SIZE_OFFSET + 2].copy_from_slice(&[0x10, 0x00]);
//
//         // Write some test values
//         buffer[FILE_FORMAT_W_OFFSET] = 4;  // write format
//         buffer[FILE_FORMAT_R_OFFSET] = 4;  // read format
//         buffer[MAX_EMBEDDED_PAYLOAD_OFFSET] = 64;  // max embedded payload
//         buffer[MIN_EMBEDDED_PAYLOAD_OFFSET] = 32;  // min embedded payload
//         buffer[LEAF_PAYLOAD_FRACTION_OFFSET] = 32;  // leaf payload fraction
//         buffer[FILE_CHANGE_COUNTER_OFFSET..FILE_CHANGE_COUNTER_OFFSET + 4].copy_from_slice(&[0x00, 0x00, 0x00, 0x01]);  // change counter
//         buffer[DB_SIZE_OFFSET..DB_SIZE_OFFSET + 4].copy_from_slice(&[0x00, 0x00, 0x00, 0x10]);  // db size
//         buffer[SCHEMA_COOKIE_OFFSET..SCHEMA_COOKIE_OFFSET + 4].copy_from_slice(&[0x00, 0x00, 0x00, 0x00]);  // schema cookie
//         buffer[SQ_VERSION_OFFSET..SQ_VERSION_OFFSET + 4].copy_from_slice(&[0x00, 0x00, 0x00, 0x04]);  // SQL version
//
//         let header = parse_header(&buffer).unwrap();
//         assert_eq!(header.page_size, 4096);
//         assert_eq!(header.file_format_w, 4);
//         assert_eq!(header.file_format_r, 4);
//     }
//
//     #[test]
//     fn test_parse_header_invalid_prefix() {
//         let mut buffer = [0u8; HEADER_SIZE];
//         // Write an invalid prefix
//         buffer[0..16].copy_from_slice(b"Invalid prefix  \0");
//
//         let result = parse_header(&buffer);
//         assert!(result.is_err());
//         assert!(result.unwrap_err().to_string().contains("Invalid header prefix"));
//     }
//
//     #[test]
//     fn test_parse_header_invalid_page_size() {
//         let mut buffer = [0u8; HEADER_SIZE];
//         // Write the valid SQLite header prefix
//         buffer[0..16].copy_from_slice(b"SQLite format 3\0");
//
//         // Write an invalid page size (not power of 2)
//         buffer[HEADER_PAGE_SIZE_OFFSET..HEADER_PAGE_SIZE_OFFSET + 2].copy_from_slice(&[0x00, 0x03]); // 3 is not a power of 2
//
//         let result = parse_header(&buffer);
//         assert!(result.is_err());
//         assert!(result.unwrap_err().to_string().contains("page size is not a power of 2"));
//     }
//
//     #[test]
//     fn test_parse_header_max_page_size() {
//         let mut buffer = [0u8; HEADER_SIZE];
//         // Write the valid SQLite header prefix
//         buffer[0..16].copy_from_slice(b"SQLite format 3\0");
//
//         // Write page size as 1 (special case for 65536)
//         buffer[HEADER_PAGE_SIZE_OFFSET..HEADER_PAGE_SIZE_OFFSET + 2].copy_from_slice(&[0x00, 0x01]); // 1 represents max size
//
//         let header = parse_header(&buffer).unwrap();
//         assert_eq!(header.page_size, PAGE_MAX_SIZE);
//     }
//
//     #[test]
//     fn test_parse_header_power_of_two_page_size() {
//         let mut buffer = [0u8; HEADER_SIZE];
//         // Write the valid SQLite header prefix
//         buffer[0..16].copy_from_slice(b"SQLite format 3\0");
//
//         // Write page size as 2048 (0x0800, which is 2^11)
//         buffer[HEADER_PAGE_SIZE_OFFSET..HEADER_PAGE_SIZE_OFFSET + 2].copy_from_slice(&[0x08, 0x00]);
//
//         let header = parse_header(&buffer).unwrap();
//         assert_eq!(header.page_size, 2048);
//     }
// }
