use std::borrow::Cow;

use crate::sql::ast::Predicate;

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
            RecordFieldType::I16 => {
                Some(Value::Int(read_i16_at(&self.payload, record_field.offset)))
            }
            RecordFieldType::I24 => {
                Some(Value::Int(read_i24_at(&self.payload, record_field.offset)))
            }
            RecordFieldType::I32 => {
                Some(Value::Int(read_i32_at(&self.payload, record_field.offset)))
            }
            RecordFieldType::I48 => {
                Some(Value::Int(read_i48_at(&self.payload, record_field.offset)))
            }
            RecordFieldType::I64 => {
                Some(Value::Int(read_i64_at(&self.payload, record_field.offset)))
            }
            RecordFieldType::Float => Some(Value::Float(read_f64_at(
                &self.payload,
                record_field.offset,
            ))),
            RecordFieldType::String(length) => {
                let value = std::str::from_utf8(
                    &self.payload[record_field.offset..record_field.offset + length],
                )
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

    #[inline(always)]
    pub fn by_predicate(&self, p: &Predicate) -> bool {
        self.field(p.field).unwrap().compare(&p.value)
    }

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
