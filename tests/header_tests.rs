#[cfg(test)]
mod tests {
  use rust_sqlite::dbheader::parse_header;
  use rust_sqlite::dbheader::*;
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

  #[test]
  fn test_parse_valid_header() {
    let mut buffer = [0u8; HEADER_SIZE];
    // Write the SQLite header prefix
    buffer[0..16].copy_from_slice(b"SQLite format 3\0");

    // Write a valid page size (4096 = 0x1000)
    buffer[HEADER_PAGE_SIZE_OFFSET..HEADER_PAGE_SIZE_OFFSET + 2].copy_from_slice(&[0x10, 0x00]);

    // Write some test values
    buffer[FILE_FORMAT_W_OFFSET] = 4; // write format
    buffer[FILE_FORMAT_R_OFFSET] = 4; // read format
    buffer[MAX_EMBEDDED_PAYLOAD_OFFSET] = 64; // max embedded payload
    buffer[MIN_EMBEDDED_PAYLOAD_OFFSET] = 32; // min embedded payload
    buffer[LEAF_PAYLOAD_FRACTION_OFFSET] = 32; // leaf payload fraction
    buffer[FILE_CHANGE_COUNTER_OFFSET..FILE_CHANGE_COUNTER_OFFSET + 4]
      .copy_from_slice(&[0x00, 0x00, 0x00, 0x01]); // change counter
    buffer[DB_SIZE_OFFSET..DB_SIZE_OFFSET + 4].copy_from_slice(&[0x00, 0x00, 0x00, 0x10]); // db size
    buffer[SCHEMA_COOKIE_OFFSET..SCHEMA_COOKIE_OFFSET + 4]
      .copy_from_slice(&[0x00, 0x00, 0x00, 0x00]); // schema cookie
    buffer[SQ_VERSION_OFFSET..SQ_VERSION_OFFSET + 4].copy_from_slice(&[0x00, 0x00, 0x00, 0x04]); // SQL version

    let header = parse_header(&buffer).unwrap();
    assert_eq!(header.page_size, 4096);
    assert_eq!(header.file_format_w, 4);
    assert_eq!(header.file_format_r, 4);
  }

  #[test]
  fn test_parse_header_invalid_prefix() {
    let mut buffer = [0u8; HEADER_SIZE];
    // Write an invalid prefix
    buffer[0..16].copy_from_slice(b"Invalid prefi  \0");

    let result = parse_header(&buffer);
    assert!(result.is_err());
    assert!(result
      .unwrap_err()
      .to_string()
      .contains("Invalid header prefix"));
  }

  #[test]
  fn test_parse_header_invalid_page_size() {
    let mut buffer = [0u8; HEADER_SIZE];
    // Write the valid SQLite header prefix
    buffer[0..16].copy_from_slice(b"SQLite format 3\0");

    // Write an invalid page size (not power of 2)
    buffer[HEADER_PAGE_SIZE_OFFSET..HEADER_PAGE_SIZE_OFFSET + 2].copy_from_slice(&[0x00, 0x03]); // 3 is not a power of 2

    let result = parse_header(&buffer);
    assert!(result.is_err());
    assert!(result
      .unwrap_err()
      .to_string()
      .contains("page size is not a power of 2"));
  }

  #[test]
  fn test_parse_header_max_page_size() {
    let mut buffer = [0u8; HEADER_SIZE];
    // Write the valid SQLite header prefix
    buffer[0..16].copy_from_slice(b"SQLite format 3\0");

    // Write page size as 1 (special case for 65536)
    buffer[HEADER_PAGE_SIZE_OFFSET..HEADER_PAGE_SIZE_OFFSET + 2].copy_from_slice(&[0x00, 0x01]); // 1 represents max size

    let header = parse_header(&buffer).unwrap();
    assert_eq!(header.page_size, PAGE_MAX_SIZE);
  }

  #[test]
  fn test_parse_header_power_of_two_page_size() {
    let mut buffer = [0u8; HEADER_SIZE];
    // Write the valid SQLite header prefix
    buffer[0..16].copy_from_slice(b"SQLite format 3\0");

    // Write page size as 2048 (0x0800, which is 2^11)
    buffer[HEADER_PAGE_SIZE_OFFSET..HEADER_PAGE_SIZE_OFFSET + 2].copy_from_slice(&[0x08, 0x00]);

    let header = parse_header(&buffer).unwrap();
    assert_eq!(header.page_size, 2048);
  }
}
