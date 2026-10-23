//! Core library for reading, editing and writing ANSI X9.37 / X9.100-187
//! (Check21 image cash letter) files.
//!
//! A file is a sequence of records, each prefixed by a 4-byte big-endian
//! Record Descriptor Word (RDW) holding the record length. Text is EBCDIC
//! (most common) or ASCII; image data (record 52) is binary (TIFF/JPEG).

pub mod ebcdic;
pub mod file;
pub mod layout;

pub use file::{Encoding, Framing, Issue, Item, ImageView, Record, X9Error, X9File, Bundle, CashLetter};
pub use layout::{FieldDef, Kind};
