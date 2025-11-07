use std::collections::HashMap;
use crate::amf::amf0::{markers, Amf0Value};
use crate::{ByteBuffer, Error};
use crate::Result;
pub struct Amf0Decoder<'a> {
    buffer: &'a mut ByteBuffer,
    references: Vec<Amf0Value>,
}

impl<'a> Amf0Decoder<'a> {
    pub fn new(buffer: &'a mut ByteBuffer) -> Self {
        Amf0Decoder {
            buffer,
            references: Vec::new(),
        }
    }

    /// Check if decoder has remaining data to decode
    pub fn has_remaining(&self) -> bool {
        self.buffer.remaining() > 0
    }

    pub fn decode(&mut self) -> Result<Amf0Value> {
        let marker = self.buffer.read_u8()?;
        match marker {
            markers::NUMBER => self.decode_number(),
            markers::BOOLEAN => self.decode_boolean(),
            markers::STRING => self.decode_string(),
            markers::OBJECT => self.decode_object(),
            markers::NULL => Ok(Amf0Value::Null),
            markers::UNDEFINED => Ok(Amf0Value::Undefined),
            markers::ECMA_ARRAY => self.decode_ecma_array(),
            markers::STRICT_ARRAY => self.decode_strict_array(),
            markers::DATE => self.decode_date(),
            markers::LONG_STRING => self.decode_long_string(),
            markers::UNSUPPORTED => Ok(Amf0Value::Unsupported),
            markers::XML_DOCUMENT => self.decode_xml_document(),
            markers::TYPED_OBJECT => self.decode_typed_object(),
            _ => Err(Error::protocol(format!("Unknown AMF0 marker: 0x{:02x}", marker))),
        }
    }

    fn decode_number(&mut self) -> Result<Amf0Value> {
        let value = self.buffer.read_f64_be()?;
        Ok(Amf0Value::Number(value))
    }

    fn decode_string(&mut self) -> Result<Amf0Value> {
        let len = self.buffer.read_u16_be()? as usize;
        let bytes = self.buffer.read_bytes(len)?;
        let string = String::from_utf8(bytes)
            .map_err(|e| Error::protocol(format!("Invalid UTF-8 in string: {}", e)))?;
        Ok(Amf0Value::String(string))
    }

    fn decode_boolean(&mut self) -> Result<Amf0Value> {
        let value = self.buffer.read_u8()? != 0;
        Ok(Amf0Value::Boolean(value))
    }

    fn decode_object(&mut self) -> Result<Amf0Value> {
        let mut object = HashMap::new();
        loop {
            let name_len = self.buffer.read_u16_be()? as usize;
            if name_len == 0 {
                self.buffer.read_u8()?; // Object end marker
                break;
            }
            let name = String::from_utf8(self.buffer.read_bytes(name_len)?)
                .map_err(|e| Error::protocol(format!("Invalid UTF-8 in property name: {}", e)))?;
            let value = self.decode()?;
            object.insert(name, value);
        }
        Ok(Amf0Value::Object(object))
    }

    fn decode_ecma_array(&mut self) -> Result<Amf0Value> {
        let _count = self.buffer.read_u32_be()?; // Array count (not used)
        let mut array = HashMap::new();
        loop {
            let name_len = self.buffer.read_u16_be()? as usize;
            if name_len == 0 {
                self.buffer.read_u8()?; // Array end marker
                break;
            }
            let name = String::from_utf8(self.buffer.read_bytes(name_len)?)
                .map_err(|e| Error::protocol(format!("Invalid UTF-8 in property name: {}", e)))?;
            let value = self.decode()?;
            array.insert(name, value);
        }
        Ok(Amf0Value::EcmaArray(array))
    }

    fn decode_strict_array(&mut self) -> Result<Amf0Value> {
        let count = self.buffer.read_u32_be()? as usize;
        let mut array = Vec::with_capacity(count);
        for _ in 0..count {
            array.push(self.decode()?);
        }
        Ok(Amf0Value::Array(array))
    }

    fn decode_date(&mut self) -> Result<Amf0Value> {
        let timestamp = self.buffer.read_f64_be()?;
        let timezone = self.buffer.read_i16_be()?;
        Ok(Amf0Value::Date(timestamp, timezone))
    }

    fn decode_long_string(&mut self) -> Result<Amf0Value> {
        let len = self.buffer.read_u32_be()? as usize;
        let bytes = self.buffer.read_bytes(len)?;
        let string = String::from_utf8(bytes)
            .map_err(|e| Error::protocol(format!("Invalid UTF-8 in long string: {}", e)))?;
        Ok(Amf0Value::LongString(string))
    }

    fn decode_xml_document(&mut self) -> Result<Amf0Value> {
        let len = self.buffer.read_u32_be()? as usize;
        let bytes = self.buffer.read_bytes(len)?;
        let xml = String::from_utf8(bytes)
            .map_err(|e| Error::protocol(format!("Invalid UTF-8 in XML: {}", e)))?;
        Ok(Amf0Value::XmlDocument(xml))
    }

    fn decode_typed_object(&mut self) -> Result<Amf0Value> {
        let class_name_len = self.buffer.read_u16_be()? as usize;
        let class_name = String::from_utf8(self.buffer.read_bytes(class_name_len)?)
            .map_err(|e| Error::protocol(format!("Invalid UTF-8 in class name: {}", e)))?;

        let mut object = HashMap::new();
        loop {
            let name_len = self.buffer.read_u16_be()? as usize;
            if name_len == 0 {
                self.buffer.read_u8()?; // Object end marker
                break;
            }
            let name = String::from_utf8(self.buffer.read_bytes(name_len)?)
                .map_err(|e| Error::protocol(format!("Invalid UTF-8 in property name: {}", e)))?;
            let value = self.decode()?;
            object.insert(name, value);
        }
        Ok(Amf0Value::TypedObject(class_name, object))
    }
}