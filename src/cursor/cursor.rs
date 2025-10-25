use std::borrow::Cow;

use super::{
  record::{RecordFieldType, RecordHeader},
  value::{OwnedValue, Value},
};

#[derive(Debug)]
pub struct Cursor {
  pub header: RecordHeader,
  pub payload: Vec<u8>,
}

impl Cursor {
  pub fn field(&self, n: usize) -> Option<Value> {
    let record_field = self.header.fields.get(n)?;

    match record_field.field_type {
      RecordFieldType::Null => Some(Value::Null),
      RecordFieldType::I8 => Some(Value::Int(read_i8_at(&self.payload, record_field.offset))),
      RecordFieldType::I16 => Some(Value::Int(read_i16_at(&self.payload, record_field.offset))),
      RecordFieldType::I24 => Some(Value::Int(read_i24_at(&self.payload, record_field.offset))),
      RecordFieldType::I32 => Some(Value::Int(read_i32_at(&self.payload, record_field.offset))),
      RecordFieldType::I48 => Some(Value::Int(read_i48_at(&self.payload, record_field.offset))),
      RecordFieldType::I64 => Some(Value::Int(read_i64_at(&self.payload, record_field.offset))),
      RecordFieldType::Float => Some(Value::Float(read_f64_at(
        &self.payload,
        record_field.offset,
      ))),
      RecordFieldType::String(length) => {
        let value =
          std::str::from_utf8(&self.payload[record_field.offset..record_field.offset + length])
            .expect("invalid utf8");
        Some(Value::String(Cow::Borrowed(value)))
      }
      RecordFieldType::Blob(length) => {
        let value = &self.payload[record_field.offset..record_field.offset + length];
        Some(Value::Blob(Cow::Borrowed(value)))
      }
      RecordFieldType::One => Some(Value::Int(1)),
      RecordFieldType::Zero => Some(Value::Int(0)),
    }
  }

  // #[inline(always)]
  // pub fn by_predicate(&self, p: &Predicate) -> bool {
  //     self.field(p.field).unwrap().compare(&p.value)
  // }

  pub fn owned_field(&self, n: usize) -> Option<OwnedValue> {
    self.field(n).map(Into::into)
  }
}

fn read_i8_at(input: &[u8], offset: usize) -> i64 {
  input[offset] as i64
}

fn read_i16_at(input: &[u8], offset: usize) -> i64 {
  i16::from_be_bytes(input[offset..offset + 2].try_into().unwrap()) as i64
}

fn read_i24_at(input: &[u8], offset: usize) -> i64 {
  (i32::from_be_bytes(input[offset..offset + 3].try_into().unwrap()) & 0x00FFFFFF) as i64
}

fn read_i32_at(input: &[u8], offset: usize) -> i64 {
  i32::from_be_bytes(input[offset..offset + 4].try_into().unwrap()) as i64
}

fn read_i48_at(input: &[u8], offset: usize) -> i64 {
  i64::from_be_bytes(input[offset..offset + 6].try_into().unwrap()) & 0x0000FFFFFFFFFFFF
}

fn read_i64_at(input: &[u8], offset: usize) -> i64 {
  i64::from_be_bytes(input[offset..offset + 8].try_into().unwrap())
}

fn read_f64_at(input: &[u8], offset: usize) -> f64 {
  f64::from_be_bytes(input[offset..offset + 8].try_into().unwrap())
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use crate::cursor::record::{RecordField, RecordFieldType, RecordHeader};
//     use crate::cursor::value::{OwnedValue, Value};
//
//     #[test]
//     fn test_cursor_field_null() {
//         let header = RecordHeader {
//             fields: vec![RecordField {
//                 offset: 0,
//                 field_type: RecordFieldType::Null,
//             }],
//         };
//
//         let cursor = Cursor {
//             header,
//             payload: vec![], // No payload needed for null
//         };
//
//         let field = cursor.field(0);
//         assert_eq!(field, Some(Value::Null));
//     }
//
//     #[test]
//     fn test_cursor_field_i8() {
//         let header = RecordHeader {
//             fields: vec![RecordField {
//                 offset: 0,
//                 field_type: RecordFieldType::I8,
//             }],
//         };
//
//         let cursor = Cursor {
//             header,
//             payload: vec![0xFF], // -1 as i8
//         };
//
//         let field = cursor.field(0);
//         assert_eq!(field, Some(Value::Int(-1)));
//     }
//
//     #[test]
//     fn test_cursor_field_i16() {
//         let header = RecordHeader {
//             fields: vec![RecordField {
//                 offset: 0,
//                 field_type: RecordFieldType::I16,
//             }],
//         };
//
//         let cursor = Cursor {
//             header,
//             payload: vec![0xFF, 0xFE], // -2 as i16 in big endian
//         };
//
//         let field = cursor.field(0);
//         assert_eq!(field, Some(Value::Int(-2)));
//     }
//
//     #[test]
//     fn test_cursor_field_string() {
//         let header = RecordHeader {
//             fields: vec![RecordField {
//                 offset: 0,
//                 field_type: RecordFieldType::String(5), // String of length 5
//             }],
//         };
//
//         let cursor = Cursor {
//             header,
//             payload: b"hello".to_vec(),
//         };
//
//         let field = cursor.field(0);
//         assert_eq!(
//             field,
//             Some(Value::String(std::borrow::Cow::Borrowed("hello")))
//         );
//     }
//
//     #[test]
//     fn test_cursor_field_blob() {
//         let header = RecordHeader {
//             fields: vec![RecordField {
//                 offset: 0,
//                 field_type: RecordFieldType::Blob(3), // Blob of length 3
//             }],
//         };
//
//         let cursor = Cursor {
//             header,
//             payload: vec![0x01, 0x02, 0x03],
//         };
//
//         let field = cursor.field(0);
//         assert_eq!(
//             field,
//             Some(Value::Blob(std::borrow::Cow::Borrowed(&[0x01, 0x02, 0x03])))
//         );
//     }
//
//     #[test]
//     fn test_cursor_field_out_of_bounds() {
//         let header = RecordHeader {
//             fields: vec![RecordField {
//                 offset: 0,
//                 field_type: RecordFieldType::Null,
//             }],
//         };
//
//         let cursor = Cursor {
//             header,
//             payload: vec![],
//         };
//
//         let field = cursor.field(5); // Index out of bounds
//         assert_eq!(field, None);
//     }
//
//     #[test]
//     fn test_cursor_owned_field() {
//         let header = RecordHeader {
//             fields: vec![RecordField {
//                 offset: 0,
//                 field_type: RecordFieldType::I8,
//             }],
//         };
//
//         let cursor = Cursor {
//             header,
//             payload: vec![0x2A], // 42 as i8
//         };
//
//         let owned_field = cursor.owned_field(0);
//         if let Some(OwnedValue::Int(value)) = owned_field {
//             assert_eq!(value, 42);
//         } else {
//             panic!("Expected Some(OwnedValue::Int(42))");
//         }
//     }
//
//     #[test]
//     fn test_read_functions() {
//         let data = vec![0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF];
//
//         // Test various read functions
//         assert_eq!(read_i8_at(&data, 0), -1);
//         assert_eq!(read_i16_at(&data, 0), -1);
//         assert_eq!(read_i32_at(&data, 0), -1);
//         assert_eq!(read_i64_at(&data, 0), -1);
//
//         // Test with positive values
//         let data = vec![0x00, 0x00, 0x00, 0x2A, 0x00, 0x00, 0x00, 0x00];
//         assert_eq!(read_i8_at(&data, 0), 0);
//         assert_eq!(read_i16_at(&data, 0), 42);
//         assert_eq!(read_i32_at(&data, 0), 42);
//     }
// }
