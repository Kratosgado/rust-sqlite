pub mod cursor;
pub mod db;
pub mod dbheader;
pub mod engine;
pub mod page;
pub mod sql;

pub use page::pager;

fn read_varint_at(buffer: &[u8], mut offset: usize) -> (u8, i64) {
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

/// Read the next 2 bytes from the offset
fn read_be_word_at(input: &[u8], offset: usize) -> u16 {
    u16::from_be_bytes(input[offset..offset + 2].try_into().unwrap())
}

/// Read the next 2 bytes from the offset
fn read_be_byte_at(input: &[u8], offset: usize) -> u8 {
    u8::from_be_bytes(input[offset..offset + 1].try_into().unwrap())
}

/// read the next 4 bytes from the offset
fn read_be_double_at(input: &[u8], offset: usize) -> u32 {
    u32::from_be_bytes(input[offset..offset + 4].try_into().unwrap())
}
