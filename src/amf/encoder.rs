use std::collections::HashMap;
use crate::amf::amf0::{markers, Amf0Value};
use crate::ByteBuffer;
use crate::Result;

pub struct Amf0Encoder {
    buffer: ByteBuffer,
}

impl Amf0Encoder {
    pub fn new() -> Self {
        Amf0Encoder {
            buffer: ByteBuffer::with_capacity(1024),
        }
    }

    pub fn encode(&mut self, value: &Amf0Value) -> Result<()> {
        match value {
            Amf0Value::Number(n) => self.encode_number(*n),
            Amf0Value::Boolean(b) => self.encode_boolean(*b),
            Amf0Value::String(s) => self.encode_string(s),
            Amf0Value::Object(obj) => self.encode_object(obj),
            Amf0Value::Null => self.encode_null(),
            Amf0Value::Undefined => self.encode_undefined(),
            Amf0Value::EcmaArray(obj) => self.encode_ecma_array(obj),
            Amf0Value::Array(arr) => self.encode_array(arr),
            Amf0Value::Date(timestamp, timezone) => self.encode_date(*timestamp, *timezone),
            Amf0Value::LongString(s) => self.encode_long_string(s),
            Amf0Value::Unsupported => self.encode_unsupported(),
            Amf0Value::XmlDocument(xml) => self.encode_xml_document(xml),
            Amf0Value::TypedObject(class_name, obj) => self.encode_typed_object(class_name, obj),
        }
    }

    fn encode_number(&mut self, value: f64) -> Result<()> {
        self.buffer.write_u8(markers::NUMBER)?;
        self.buffer.write_f64_be(value)?;
        Ok(())
    }

    fn encode_string(&mut self, value: &str) -> Result<()> {
        self.buffer.write_u8(markers::STRING)?;
        let bytes = value.as_bytes();
        self.buffer.write_u16_be(bytes.len() as u16)?;
        self.buffer.write_bytes(bytes)?;
        Ok(())
    }

    fn encode_boolean(&mut self, value: bool) -> Result<()> {
        self.buffer.write_u8(markers::BOOLEAN)?;
        self.buffer.write_u8(if value { 1 } else { 0 })?;
        Ok(())
    }

    fn encode_object(&mut self, obj: &HashMap<String, Amf0Value>) -> Result<()> {
        self.buffer.write_u8(markers::OBJECT)?;
        for (key, value) in obj {
            self.write_string_no_marker(key)?;
            self.encode(value)?;
        }
        // Object end marker
        self.buffer.write_u16_be(0)?;
        self.buffer.write_u8(markers::OBJECT_END)?;
        Ok(())
    }

    fn encode_null(&mut self) -> Result<()> {
        self.buffer.write_u8(markers::NULL)?;
        Ok(())
    }

    fn encode_undefined(&mut self) -> Result<()> {
        self.buffer.write_u8(markers::UNDEFINED)?;
        Ok(())
    }

    fn encode_ecma_array(&mut self, obj: &HashMap<String, Amf0Value>) -> Result<()> {
        self.buffer.write_u8(markers::ECMA_ARRAY)?;
        self.buffer.write_u32_be(obj.len() as u32)?;
        for (key, value) in obj {
            self.write_string_no_marker(key)?;
            self.encode(value)?;
        }
        // Array end marker
        self.buffer.write_u16_be(0)?;
        self.buffer.write_u8(markers::OBJECT_END)?;
        Ok(())
    }

    fn encode_array(&mut self, arr: &Vec<Amf0Value>) -> Result<()> {
        self.buffer.write_u8(markers::STRICT_ARRAY)?;
        self.buffer.write_u32_be(arr.len() as u32)?;
        for value in arr {
            self.encode(value)?;
        }
        Ok(())
    }

    fn encode_date(&mut self, timestamp: f64, timezone: i16) -> Result<()> {
        self.buffer.write_u8(markers::DATE)?;
        self.buffer.write_f64_be(timestamp)?;
        self.buffer.write_i16_be(timezone)?;
        Ok(())
    }

    fn encode_long_string(&mut self, value: &str) -> Result<()> {
        self.buffer.write_u8(markers::LONG_STRING)?;
        let bytes = value.as_bytes();
        self.buffer.write_u32_be(bytes.len() as u32)?;
        self.buffer.write_bytes(bytes)?;
        Ok(())
    }

    fn encode_unsupported(&mut self) -> Result<()> {
        self.buffer.write_u8(markers::UNSUPPORTED)?;
        Ok(())
    }

    fn encode_xml_document(&mut self, xml: &str) -> Result<()> {
        self.buffer.write_u8(markers::XML_DOCUMENT)?;
        let bytes = xml.as_bytes();
        self.buffer.write_u32_be(bytes.len() as u32)?;
        self.buffer.write_bytes(bytes)?;
        Ok(())
    }

    fn encode_typed_object(&mut self, class_name: &str, obj: &HashMap<String, Amf0Value>) -> Result<()> {
        self.buffer.write_u8(markers::TYPED_OBJECT)?;
        let bytes = class_name.as_bytes();
        self.buffer.write_u16_be(bytes.len() as u16)?;
        self.buffer.write_bytes(bytes)?;

        for (key, value) in obj {
            self.write_string_no_marker(key)?;
            self.encode(value)?;
        }
        // Object end marker
        self.buffer.write_u16_be(0)?;
        self.buffer.write_u8(markers::OBJECT_END)?;
        Ok(())
    }

    /// Helper to write string without type marker (for object keys)
    fn write_string_no_marker(&mut self, value: &str) -> Result<()> {
        let bytes = value.as_bytes();
        self.buffer.write_u16_be(bytes.len() as u16)?;
        self.buffer.write_bytes(bytes)?;
        Ok(())
    }

    pub fn get_bytes(&self) -> Vec<u8> {
        self.buffer.to_vec()
    }
}