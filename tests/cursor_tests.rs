#[cfg(test)]
mod cursor {
  use rust_sqlite::cursor::{
    cursor::Cursor,
    record::{RecordField, RecordFieldType, RecordHeader},
    value::{OwnedValue, Value},
  };

  #[test]
  fn field_null() {
    let header = RecordHeader {
      fields: vec![RecordField {
        offset: 0,
        field_type: RecordFieldType::Null,
      }],
    };

    let cursor = Cursor {
      header,
      payload: vec![], // No payload needed for null
    };

    let field = cursor.field(0);
    assert_eq!(field, Some(Value::Null));
  }

  #[test]
  fn field_i8() {
    let header = RecordHeader {
      fields: vec![RecordField {
        offset: 0,
        field_type: RecordFieldType::I8,
      }],
    };

    let cursor = Cursor {
      header,
      payload: vec![0xFF], // -1 as i8
    };

    let field = cursor.field(0);
    assert_eq!(field, Some(Value::Int(255)));
  }

  #[test]
  fn field_i16() {
    let header = RecordHeader {
      fields: vec![RecordField {
        offset: 0,
        field_type: RecordFieldType::I16,
      }],
    };

    let cursor = Cursor {
      header,
      payload: vec![0xFF, 0xFE], // -2 as i16 in big endian
    };

    let field = cursor.field(0);
    assert_eq!(field, Some(Value::Int(-2)));
  }

  #[test]
  fn field_string() {
    let header = RecordHeader {
      fields: vec![RecordField {
        offset: 0,
        field_type: RecordFieldType::String(5), // String of length 5
      }],
    };

    let cursor = Cursor {
      header,
      payload: b"hello".to_vec(),
    };

    let field = cursor.field(0);
    assert_eq!(
      field,
      Some(Value::String(std::borrow::Cow::Borrowed("hello")))
    );
  }

  #[test]
  fn field_blob() {
    let header = RecordHeader {
      fields: vec![RecordField {
        offset: 0,
        field_type: RecordFieldType::Blob(3), // Blob of length 3
      }],
    };

    let cursor = Cursor {
      header,
      payload: vec![0x01, 0x02, 0x03],
    };

    let field = cursor.field(0);
    assert_eq!(
      field,
      Some(Value::Blob(std::borrow::Cow::Borrowed(&[0x01, 0x02, 0x03])))
    );
  }

  #[test]
  fn field_out_of_bounds() {
    let header = RecordHeader {
      fields: vec![RecordField {
        offset: 0,
        field_type: RecordFieldType::Null,
      }],
    };

    let cursor = Cursor {
      header,
      payload: vec![],
    };

    let field = cursor.field(5); // Index out of bounds
    assert_eq!(field, None);
  }

  #[test]
  fn owned_field() {
    let header = RecordHeader {
      fields: vec![RecordField {
        offset: 0,
        field_type: RecordFieldType::I8,
      }],
    };

    let cursor = Cursor {
      header,
      payload: vec![0x2A], // 42 as i8
    };

    let owned_field = cursor.owned_field(0);
    if let Some(OwnedValue::Int(value)) = owned_field {
      assert_eq!(value, 42);
    } else {
      panic!("Expected Some(OwnedValue::Int(42))");
    }
  }
}
