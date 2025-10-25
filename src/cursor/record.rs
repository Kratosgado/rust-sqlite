#[derive(Debug, Copy, Clone, PartialEq)]
pub enum RecordFieldType {
  Null,
  I8,
  I16,
  I24,
  I32,
  I48,
  I64,
  Float,
  Zero,
  One,
  String(usize),
  Blob(usize),
}

#[derive(Debug, Clone)]
pub struct RecordField {
  pub offset: usize,
  pub field_type: RecordFieldType,
}

#[derive(Debug, Clone)]
pub struct RecordHeader {
  pub fields: Vec<RecordField>,
}

pub fn parse_record_header(mut cell_buff: &[u8]) -> anyhow::Result<RecordHeader> {
  let (varint_size, header_length) = crate::read_varint_at(cell_buff, 0);
  cell_buff = &cell_buff[varint_size as usize..header_length as usize];

  let mut fields = vec![];
  let mut current_offset = header_length as usize;

  while !cell_buff.is_empty() {
    let (serial_size, serial_type) = crate::read_varint_at(cell_buff, 0);
    cell_buff = &cell_buff[serial_size as usize..];

    let (field_type, field_size) = match serial_type {
      0 => (RecordFieldType::Null, 0),
      1 => (RecordFieldType::I8, 1),
      2 => (RecordFieldType::I16, 2),
      3 => (RecordFieldType::I24, 3),
      4 => (RecordFieldType::I32, 4),
      5 => (RecordFieldType::I48, 6),
      6 => (RecordFieldType::I64, 8),
      7 => (RecordFieldType::Float, 8),
      8 => (RecordFieldType::Zero, 0),
      9 => (RecordFieldType::One, 0),
      n if n >= 12 && n % 2 == 0 => {
        let size = ((n - 12) / 2) as usize;
        (RecordFieldType::Blob(size), size)
      }
      n if n >= 13 && n % 2 == 1 => {
        let size = ((n - 13) / 2) as usize;
        (RecordFieldType::String(size), size)
      }
      n => anyhow::bail!("unsupported field type: {}", n),
    };

    fields.push(RecordField {
      offset: current_offset,
      field_type,
    });

    current_offset += field_size;
  }
  Ok(RecordHeader { fields })
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//
//     #[test]
//     fn test_parse_record_header_with_various_types() {
//         // Create a mock buffer that represents a header with multiple field types
//         // The first varint is the header length itself, then serial types follow
//         let mut buffer = Vec::new();
//
//         // First, we need to calculate the header length
//         // Let's create a header with: Null, I8, I16, String(5), Blob(3)
//         let serial_types = vec![0, 1, 2, 13 + 2*5]; // 0=Null, 1=I8, 2=I16, 13+2*5=String(5)
//         let mut mock_header = Vec::new();
//
//         // Add the total header length as a varint (we'll calculate this later)
//         // For now, let's just add the serial types as varints
//         for &st in &serial_types {
//             // Add varint representation of the serial type
//             if st < 128 {
//                 mock_header.push(st as u8);
//             } else {
//                 // For simplicity, we'll just test simple cases first
//                 mock_header.push(st as u8);
//             }
//         }
//
//         // Now, we'll manually create a test buffer with the correct format
//         let test_buffer = vec![
//             0x04,  // Length of header (4 bytes): varint(0x04)
//             0x00,  // Serial type 0 (Null)
//             0x01,  // Serial type 1 (I8)
//             0x02,  // Serial type 2 (I16)
//             0x11,  // Serial type 17 (String(2): (17-13)/2 = 2)
//             // Now comes the actual data that would follow the header
//             0x00, 0x00, 0x00, 0x00,  // I8 value
//             0x00, 0x00,             // I16 value
//             b'H', b'i',             // String value "Hi"
//         ];
//
//         let result = parse_record_header(&test_buffer);
//         assert!(result.is_ok());
//
//         let header = result.unwrap();
//         assert_eq!(header.fields.len(), 4);
//
//         assert_eq!(header.fields[0].field_type, RecordFieldType::Null);
//         assert_eq!(header.fields[1].field_type, RecordFieldType::I8);
//         assert_eq!(header.fields[2].field_type, RecordFieldType::I16);
//         if let RecordFieldType::String(2) = header.fields[3].field_type {
//             // This is correct
//         } else {
//             panic!("Expected String(2) field type");
//         }
//     }
//
//     #[test]
//     fn test_parse_record_header_null_type() {
//         let buffer = vec![
//             0x01,  // Length of header: varint(0x01) = 1 byte for header
//             0x00,  // Serial type 0 (Null)
//             // No data follows for a null field
//         ];
//
//         let result = parse_record_header(&buffer);
//         assert!(result.is_ok());
//
//         let header = result.unwrap();
//         assert_eq!(header.fields.len(), 1);
//         assert_eq!(header.fields[0].field_type, RecordFieldType::Null);
//     }
//
//     #[test]
//     fn test_parse_record_header_i8_type() {
//         let buffer = vec![
//             0x02,  // Length of header: varint(0x02) = 2 bytes for header
//             0x01,  // Serial type 1 (I8)
//             0x01,  // Serial type 1 (I8) - for second field
//             0x2A,  // I8 value (42)
//         ];
//
//         let result = parse_record_header(&buffer);
//         assert!(result.is_ok());
//
//         let header = result.unwrap();
//         assert_eq!(header.fields.len(), 2);
//         assert_eq!(header.fields[0].field_type, RecordFieldType::I8);
//     }
//
//     #[test]
//     fn test_parse_record_header_string_type() {
//         let buffer = vec![
//             0x02,  // Length of header: varint(0x02) = 2 bytes for header
//             0x0D,  // Serial type 13 (String(0): (13-13)/2 = 0)
//             0x0F,  // Serial type 15 (String(1): (15-13)/2 = 1)
//         ];
//
//         let result = parse_record_header(&buffer);
//         assert!(result.is_ok());
//
//         let header = result.unwrap();
//         assert_eq!(header.fields.len(), 2);
//
//         if let RecordFieldType::String(0) = header.fields[0].field_type {
//         } else {
//             panic!("Expected String(0) field type");
//         }
//
//         if let RecordFieldType::String(1) = header.fields[1].field_type {
//         } else {
//             panic!("Expected String(1) field type");
//         }
//     }
//
//     #[test]
//     fn test_parse_record_header_blob_type() {
//         let buffer = vec![
//             0x02,  // Length of header: varint(0x02) = 2 bytes for header
//             0x0C,  // Serial type 12 (Blob(0): (12-12)/2 = 0)
//             0x0E,  // Serial type 14 (Blob(1): (14-12)/2 = 1)
//         ];
//
//         let result = parse_record_header(&buffer);
//         assert!(result.is_ok());
//
//         let header = result.unwrap();
//         assert_eq!(header.fields.len(), 2);
//
//         if let RecordFieldType::Blob(0) = header.fields[0].field_type {
//         } else {
//             panic!("Expected Blob(0) field type");
//         }
//
//         if let RecordFieldType::Blob(1) = header.fields[1].field_type {
//         } else {
//             panic!("Expected Blob(1) field type");
//         }
//     }
//
//     #[test]
//     fn test_parse_record_header_unsupported_type() {
//         let buffer = vec![
//             0x01,  // Length of header: varint(0x01) = 1 byte for header
//             0xFF,  // Unsupported serial type
//         ];
//
//         let result = parse_record_header(&buffer);
//         assert!(result.is_err());
//         assert!(result.unwrap_err().to_string().contains("unsupported field type"));
//     }
// }
