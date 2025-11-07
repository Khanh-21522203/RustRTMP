use std::collections::HashMap;
use crate::{ByteBuffer, Error, Result};

/// AMF0 data types
#[derive(Debug, Clone, PartialEq)]
pub enum Amf0Value {
    // Core types (required for RTMP)
    Number(f64),                                    // 0x00
    Boolean(bool),                                  // 0x01
    String(String),                                 // 0x02
    Object(HashMap<String, Amf0Value>),            // 0x03
    Null,                                          // 0x05
    Undefined,                                     // 0x06
    EcmaArray(HashMap<String, Amf0Value>),         // 0x08 (for metadata)

    // Extended types (optional)
    Array(Vec<Amf0Value>),                         // 0x0A (strict array)
    Date(f64, i16),                                // 0x0B
    LongString(String),                            // 0x0C

    // Legacy types (for compatibility only)
    Unsupported,                                   // 0x0D
    XmlDocument(String),                           // 0x0F
    TypedObject(String, HashMap<String, Amf0Value>), // 0x10
}

// For minimal RTMP implementation, you only need:
// - Number, Boolean, String, Object, Null, Undefined, EcmaArray
// The rest can be handled as Unsupported or implemented later

// AMF0 type markers
pub mod markers {
    // Core types (required for basic RTMP)
    pub const NUMBER: u8 = 0x00;        // Numbers, timestamps, transaction IDs
    pub const BOOLEAN: u8 = 0x01;       // Boolean values
    pub const STRING: u8 = 0x02;        // Strings (up to 65535 bytes)
    pub const OBJECT: u8 = 0x03;        // Objects (key-value pairs)
    pub const NULL: u8 = 0x05;          // Null value
    pub const UNDEFINED: u8 = 0x06;     // Undefined value
    pub const ECMA_ARRAY: u8 = 0x08;    // Associative arrays (metadata)
    pub const OBJECT_END: u8 = 0x09;    // Object terminator marker

    // Extended types (optional, for compatibility)
    pub const STRICT_ARRAY: u8 = 0x0A;  // Strict arrays (indexed)
    pub const DATE: u8 = 0x0B;          // Date with timezone
    pub const LONG_STRING: u8 = 0x0C;   // Strings > 65535 bytes

    // Legacy types (rarely used, can be stubbed)
    pub const MOVIE_CLIP: u8 = 0x04;    // Flash MovieClip (deprecated)
    pub const REFERENCE: u8 = 0x07;     // Object reference (complex)
    pub const UNSUPPORTED: u8 = 0x0D;   // Unsupported type marker
    pub const RECORDSET: u8 = 0x0E;     // Flash Remoting (legacy)
    pub const XML_DOCUMENT: u8 = 0x0F;  // XML document (deprecated)
    pub const TYPED_OBJECT: u8 = 0x10;  // Typed object (custom class)
    pub const AVMPLUS_OBJECT: u8 = 0x11;// AMF3 object (different spec)
}

impl Amf0Value {
    /// Extract number value
    pub fn as_number(&self) -> Option<f64> {
        match self {
            Amf0Value::Number(n) => Some(*n),
            _ => None,
        }
    }

    /// Extract string reference
    pub fn as_string(&self) -> Option<&str> {
        match self {
            Amf0Value::String(s) | Amf0Value::LongString(s) => Some(s.as_str()),
            _ => None,
        }
    }

    /// Extract boolean value
    pub fn as_boolean(&self) -> Option<bool> {
        match self {
            Amf0Value::Boolean(b) => Some(*b),
            _ => None,
        }
    }

    /// Extract object reference
    pub fn as_object(&self) -> Option<&HashMap<String, Amf0Value>> {
        match self {
            Amf0Value::Object(obj) | Amf0Value::EcmaArray(obj) => Some(obj),
            Amf0Value::TypedObject(_, obj) => Some(obj),
            _ => None,
        }
    }

    /// Extract array reference
    pub fn as_array(&self) -> Option<&Vec<Amf0Value>> {
        match self {
            Amf0Value::Array(arr) => Some(arr),
            _ => None,
        }
    }

    /// Get property from object
    pub fn get_property(&self, key: &str) -> Option<&Amf0Value> {
        self.as_object().and_then(|obj| obj.get(key))
    }

    /// Check if null or undefined
    pub fn is_null(&self) -> bool {
        matches!(self, Amf0Value::Null | Amf0Value::Undefined)
    }
}

pub fn decode_amf0_number(bytes: &[u8]) -> Result<(f64, usize)> {
    if bytes.len() < 8 {
        return Err(Error::amf_decode("Not enough bytes for number"));
    }

    let mut buffer = ByteBuffer::new(bytes.to_vec());
    let value = buffer.read_f64_be()?;
    Ok((value, 8))
}

pub fn encode_amf0_string(value: &str) -> Vec<u8> {
    let mut buffer = ByteBuffer::with_capacity(3 + value.len());
    buffer.write_u8(markers::STRING).unwrap();
    buffer.write_u16_be(value.len() as u16).unwrap();
    buffer.write_bytes(value.as_bytes()).unwrap();
    buffer.to_vec()
}