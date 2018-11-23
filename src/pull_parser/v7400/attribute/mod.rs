//! Node attributes.

use std::io;
use std::num::NonZeroU64;

use crate::low::v7400::{ArrayAttributeHeader, AttributeType, SpecialAttributeHeader};

use super::{FromReader, Parser, ParserSource, Result};

use self::array::{ArrayAttributeValues, AttributeStreamDecoder, BooleanArrayAttributeValues};
pub use self::direct::DirectAttributeValue;
pub use self::visitor::VisitAttribute;

mod array;
mod direct;
pub mod visitor;

/// Node attributes reader.
#[derive(Debug)]
pub struct Attributes<'a, R: 'a> {
    /// Total number of attributes of the current node.
    total_count: u64,
    /// Rest number of attributes of the current node.
    rest_count: u64,
    /// End offset of the previous attribute, if available.
    ///
    /// "End offset" means a next byte of the last byte of the previous
    /// attribute.
    prev_attr_end_offset: Option<NonZeroU64>,
    /// Parser.
    parser: &'a mut Parser<R>,
}

impl<'a, R: 'a + ParserSource> Attributes<'a, R> {
    /// Creates a new `Attributes`.
    pub(crate) fn from_parser(parser: &'a mut Parser<R>) -> Self {
        let total_count = parser.current_attributes_count();
        Self {
            total_count,
            rest_count: total_count,
            prev_attr_end_offset: None,
            parser,
        }
    }

    /// Returns the total number of attributes.
    pub fn total_count(&self) -> u64 {
        self.total_count
    }

    /// Returns the rest number of attributes.
    pub fn rest_count(&self) -> u64 {
        self.rest_count
    }

    /// Updates the attribute end offset according to the given size (in bytes).
    fn update_attr_end_offset(&mut self, size: u64) {
        self.prev_attr_end_offset = NonZeroU64::new(
            self.parser
                .reader()
                .position()
                .checked_add(size)
                .expect("FBX data too large"),
        );
    }

    /// Returns the next attribute type.
    fn read_next_attr_type(&mut self) -> Result<Option<AttributeType>> {
        if self.rest_count() == 0 {
            return Ok(None);
        }
        // This never overflows because `rest_count > 0` holds here.
        self.rest_count -= 1;

        // Skip the previous attribute value if it remains.
        if let Some(prev_attr_end_offset) = self.prev_attr_end_offset {
            if self.parser.reader().position() < prev_attr_end_offset.get() {
                self.parser.reader().skip_to(prev_attr_end_offset.get())?;
            }
        }

        let attr_type = self.parser.parse::<AttributeType>()?;
        Ok(Some(attr_type))
    }

    /// Let visitor visit the next node attribute.
    pub fn visit_next<V>(&mut self, visitor: V) -> Result<Option<V::Output>>
    where
        V: VisitAttribute,
    {
        let attr_type = match self.read_next_attr_type()? {
            Some(v) => v,
            None => return Ok(None),
        };
        self.visit_next_impl(attr_type, visitor).map(Some)
    }

    /// Let visitor visit the next node attribute.
    ///
    /// This method prefers `V::visit_{binary,string}_buffered` to
    /// `V::visit_{binary,string}`.
    pub fn visit_next_buffered<V>(&mut self, visitor: V) -> Result<Option<V::Output>>
    where
        R: io::BufRead,
        V: VisitAttribute,
    {
        let attr_type = match self.read_next_attr_type()? {
            Some(v) => v,
            None => return Ok(None),
        };
        self.visit_next_buffered_impl(attr_type, visitor).map(Some)
    }

    /// Internal implementation of `visit_next`.
    fn visit_next_impl<V>(&mut self, attr_type: AttributeType, visitor: V) -> Result<V::Output>
    where
        V: VisitAttribute,
    {
        match attr_type {
            AttributeType::Bool => {
                let raw = self.parser.parse::<u8>()?;
                let value = (raw & 1) != 0;
                self.update_attr_end_offset(0);
                /*
                if raw != b'T' && raw != b'Y' {
                    // FIXME: Warn the incorrect boolean attribute value.
                }
                */
                visitor.visit_bool(value)
            }
            AttributeType::I16 => {
                let value = self.parser.parse::<i16>()?;
                self.update_attr_end_offset(0);
                visitor.visit_i16(value)
            }
            AttributeType::I32 => {
                let value = self.parser.parse::<i32>()?;
                self.update_attr_end_offset(0);
                visitor.visit_i32(value)
            }
            AttributeType::I64 => {
                let value = self.parser.parse::<i64>()?;
                self.update_attr_end_offset(0);
                visitor.visit_i64(value)
            }
            AttributeType::F32 => {
                let value = self.parser.parse::<f32>()?;
                self.update_attr_end_offset(0);
                visitor.visit_f32(value)
            }
            AttributeType::F64 => {
                let value = self.parser.parse::<f64>()?;
                self.update_attr_end_offset(0);
                visitor.visit_f64(value)
            }
            AttributeType::ArrBool => {
                let header = ArrayAttributeHeader::from_reader(self.parser.reader())?;
                self.update_attr_end_offset(u64::from(header.bytelen()));
                let reader =
                    AttributeStreamDecoder::create(header.encoding(), self.parser.reader())?;
                let count = header.elements_count();
                let mut iter = BooleanArrayAttributeValues::new(reader, count);
                let res = visitor.visit_seq_bool(&mut iter, count as usize)?;
                /*
                if iter.has_incorrect_boolean_value() {
                    // FIXME: Warn the incorrect boolean attribute value.
                }
                */
                Ok(res)
            }
            AttributeType::ArrI32 => {
                let header = ArrayAttributeHeader::from_reader(self.parser.reader())?;
                self.update_attr_end_offset(u64::from(header.bytelen()));
                let reader =
                    AttributeStreamDecoder::create(header.encoding(), self.parser.reader())?;
                let count = header.elements_count();
                let iter = ArrayAttributeValues::<_, i32>::new(reader, count);
                visitor.visit_seq_i32(iter, count as usize)
            }
            AttributeType::ArrI64 => {
                let header = ArrayAttributeHeader::from_reader(self.parser.reader())?;
                self.update_attr_end_offset(u64::from(header.bytelen()));
                let reader =
                    AttributeStreamDecoder::create(header.encoding(), self.parser.reader())?;
                let count = header.elements_count();
                let iter = ArrayAttributeValues::<_, i64>::new(reader, count);
                visitor.visit_seq_i64(iter, count as usize)
            }
            AttributeType::ArrF32 => {
                let header = ArrayAttributeHeader::from_reader(self.parser.reader())?;
                self.update_attr_end_offset(u64::from(header.bytelen()));
                let reader =
                    AttributeStreamDecoder::create(header.encoding(), self.parser.reader())?;
                let count = header.elements_count();
                let iter = ArrayAttributeValues::<_, f32>::new(reader, count);
                visitor.visit_seq_f32(iter, count as usize)
            }
            AttributeType::ArrF64 => {
                let header = ArrayAttributeHeader::from_reader(self.parser.reader())?;
                self.update_attr_end_offset(u64::from(header.bytelen()));
                let reader =
                    AttributeStreamDecoder::create(header.encoding(), self.parser.reader())?;
                let count = header.elements_count();
                let iter = ArrayAttributeValues::<_, f64>::new(reader, count);
                visitor.visit_seq_f64(iter, count as usize)
            }
            AttributeType::Binary => {
                let header = self.parser.parse::<SpecialAttributeHeader>()?;
                let bytelen = u64::from(header.bytelen());
                self.update_attr_end_offset(bytelen);
                // `self.parser.reader().by_ref().take(bytelen)` is rejected by
                // borrowck (of rustc 1.31.0-beta.15 (4b3a1d911 2018-11-20)).
                let reader = io::Read::take(self.parser.reader(), bytelen);
                visitor.visit_binary(reader, bytelen)
            }
            AttributeType::String => {
                let header = self.parser.parse::<SpecialAttributeHeader>()?;
                let bytelen = u64::from(header.bytelen());
                self.update_attr_end_offset(bytelen);
                // `self.parser.reader().by_ref().take(bytelen)` is rejected by
                // borrowck (of rustc 1.31.0-beta.15 (4b3a1d911 2018-11-20)).
                let reader = io::Read::take(self.parser.reader(), bytelen);
                visitor.visit_string(reader, bytelen)
            }
        }
    }

    /// Internal implementation of `visit_next_buffered`.
    fn visit_next_buffered_impl<V>(
        &mut self,
        attr_type: AttributeType,
        visitor: V,
    ) -> Result<V::Output>
    where
        R: io::BufRead,
        V: VisitAttribute,
    {
        match attr_type {
            AttributeType::Binary => {
                let header = self.parser.parse::<SpecialAttributeHeader>()?;
                let bytelen = u64::from(header.bytelen());
                self.update_attr_end_offset(bytelen);
                // `self.parser.reader().by_ref().take(bytelen)` is rejected by
                // borrowck (of rustc 1.31.0-beta.15 (4b3a1d911 2018-11-20)).
                let reader = io::Read::take(self.parser.reader(), bytelen);
                visitor.visit_binary_buffered(reader, bytelen)
            }
            AttributeType::String => {
                let header = self.parser.parse::<SpecialAttributeHeader>()?;
                let bytelen = u64::from(header.bytelen());
                self.update_attr_end_offset(bytelen);
                // `self.parser.reader().by_ref().take(bytelen)` is rejected by
                // borrowck (of rustc 1.31.0-beta.15 (4b3a1d911 2018-11-20)).
                let reader = io::Read::take(self.parser.reader(), bytelen);
                visitor.visit_string_buffered(reader, bytelen)
            }
            _ => self.visit_next_impl(attr_type, visitor),
        }
    }
}