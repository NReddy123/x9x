use crate::{ebcdic, layout};
use std::ops::Range;

#[derive(Debug, thiserror::Error)]
pub enum X9Error {
    #[error("file is empty")]
    Empty,
    #[error("record at offset {0} is truncated or has an invalid length")]
    BadRecord(usize),
    #[error("not an X9.37 file: first record is not a file header (01)")]
    NotX9,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Encoding {
    Ascii,
    Ebcdic,
}

impl Encoding {
    pub fn decode(self, b: u8) -> u8 {
        match self {
            Encoding::Ascii => b,
            Encoding::Ebcdic => ebcdic::to_ascii(b),
        }
    }
    pub fn encode(self, b: u8) -> u8 {
        match self {
            Encoding::Ascii => b,
            Encoding::Ebcdic => ebcdic::to_ebcdic(b),
        }
    }
    pub fn label(self) -> &'static str {
        match self {
            Encoding::Ascii => "ASCII",
            Encoding::Ebcdic => "EBCDIC",
        }
    }
}

/// One record, stored as raw bytes (without the RDW) in the file encoding.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Record {
    pub data: Vec<u8>,
}

impl Record {
    pub fn rtype(&self, enc: Encoding) -> String {
        self.text(enc, 1, 2)
    }

    /// Text of the 1-based field `[start, start+len)`, clipped to the record.
    /// Non-printable bytes are shown as `.`.
    pub fn text(&self, enc: Encoding, start: usize, len: usize) -> String {
        let s = start.saturating_sub(1).min(self.data.len());
        let e = (s + len).min(self.data.len());
        self.data[s..e]
            .iter()
            .map(|&b| {
                let c = enc.decode(b);
                if (0x20..0x7F).contains(&c) { c as char } else { '.' }
            })
            .collect()
    }

    /// Set a field. Numeric-like kinds are right-aligned and zero padded,
    /// others left-aligned and space padded; the value is clipped to fit.
    pub fn set_field(&mut self, enc: Encoding, def: &layout::FieldDef, value: &str) {
        let s = def.start - 1;
        if s >= self.data.len() {
            return;
        }
        let len = def.len.min(self.data.len() - s);
        let numeric = matches!(def.kind, layout::Kind::Numeric | layout::Kind::Amount);
        let v: Vec<u8> = value.bytes().filter(|b| b.is_ascii() && !b.is_ascii_control()).collect();
        let v = if v.len() > len { if numeric { v[v.len() - len..].to_vec() } else { v[..len].to_vec() } } else { v };
        let pad = if numeric { b'0' } else { b' ' };
        let mut out = vec![pad; len];
        if numeric {
            out[len - v.len()..].copy_from_slice(&v);
        } else {
            out[..v.len()].copy_from_slice(&v);
        }
        for (i, b) in out.into_iter().enumerate() {
            self.data[s + i] = enc.encode(b);
        }
    }

    fn num(&self, enc: Encoding, start: usize, len: usize) -> u64 {
        self.text(enc, start, len).trim().parse().unwrap_or(0)
    }

    fn put_num(&mut self, enc: Encoding, start: usize, len: usize, v: u64) {
        let def = layout::FieldDef { name: "", start, len, kind: layout::Kind::Numeric };
        self.set_field(enc, &def, &v.to_string());
    }
}

/// An image attached to an item (a 50 record followed by its 52 record).
#[derive(Clone, Debug)]
pub struct ImageView {
    pub detail: usize,
    pub data: usize,
    pub back: bool,
    /// Byte range of the image inside the 52 record.
    pub range: Range<usize>,
}

/// A check (25) or credit (61) with all its following records.
#[derive(Clone, Debug)]
pub struct Item {
    pub range: Range<usize>,
    pub is_credit: bool,
}

#[derive(Clone, Debug)]
pub struct Bundle {
    pub header: usize,
    pub control: Option<usize>,
    pub items: Vec<Item>,
}

#[derive(Clone, Debug)]
pub struct CashLetter {
    pub header: usize,
    pub control: Option<usize>,
    pub bundles: Vec<Bundle>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Issue {
    pub record: usize,
    pub message: String,
}

#[derive(Clone, Debug)]
pub struct X9File {
    pub encoding: Encoding,
    pub records: Vec<Record>,
}

impl X9File {
    pub fn parse(bytes: &[u8]) -> Result<Self, X9Error> {
        if bytes.len() < 6 {
            return Err(X9Error::Empty);
        }
        let encoding = match (bytes[4], bytes[5]) {
            (0xF0, 0xF1) => Encoding::Ebcdic,
            (b'0', b'1') => Encoding::Ascii,
            _ => return Err(X9Error::NotX9),
        };
        let mut records = Vec::new();
        let mut pos = 0;
        while pos + 4 <= bytes.len() {
            let len = u32::from_be_bytes(bytes[pos..pos + 4].try_into().unwrap()) as usize;
            if len == 0 && bytes[pos..].iter().all(|&b| b == 0) {
                break; // trailing padding
            }
            if len < 2 || pos + 4 + len > bytes.len() {
                return Err(X9Error::BadRecord(pos));
            }
            records.push(Record { data: bytes[pos + 4..pos + 4 + len].to_vec() });
            pos += 4 + len;
        }
        Ok(Self { encoding, records })
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut out = Vec::with_capacity(self.records.iter().map(|r| r.data.len() + 4).sum());
        for r in &self.records {
            out.extend_from_slice(&(r.data.len() as u32).to_be_bytes());
            out.extend_from_slice(&r.data);
        }
        out
    }

    pub fn rtype(&self, i: usize) -> String {
        self.records[i].rtype(self.encoding)
    }

    pub fn fields(&self, i: usize) -> &'static [layout::FieldDef] {
        layout::layout_for(&self.rtype(i))
    }

    pub fn field_text(&self, i: usize, def: &layout::FieldDef) -> String {
        self.records[i].text(self.encoding, def.start, def.len)
    }

    pub fn set_field(&mut self, i: usize, def: &layout::FieldDef, value: &str) {
        let enc = self.encoding;
        self.records[i].set_field(enc, def, value);
    }

    /// Item amount in cents of a 25/61 record.
    pub fn amount(&self, i: usize) -> u64 {
        self.records[i].num(self.encoding, 48, 10)
    }

    /// Hierarchical view: cash letters > bundles > items.
    pub fn outline(&self) -> Vec<CashLetter> {
        let mut out: Vec<CashLetter> = Vec::new();
        for i in 0..self.records.len() {
            match self.rtype(i).as_str() {
                "10" => out.push(CashLetter { header: i, control: None, bundles: vec![] }),
                "20" => {
                    if let Some(cl) = out.last_mut() {
                        cl.bundles.push(Bundle { header: i, control: None, items: vec![] });
                    }
                }
                t @ ("25" | "61") => {
                    if let Some(b) = out.last_mut().and_then(|c| c.bundles.last_mut()) {
                        b.items.push(Item { range: i..i + 1, is_credit: t == "61" });
                    }
                }
                "70" => {
                    if let Some(b) = out.last_mut().and_then(|c| c.bundles.last_mut()) {
                        b.control = Some(i);
                    }
                }
                "90" => {
                    if let Some(c) = out.last_mut() {
                        c.control = Some(i);
                    }
                }
                _ => {
                    if let Some(it) = out
                        .last_mut()
                        .and_then(|c| c.bundles.last_mut())
                        .filter(|b| b.control.is_none())
                        .and_then(|b| b.items.last_mut())
                    {
                        it.range.end = i + 1;
                    }
                }
            }
        }
        out
    }

    /// Images (50 + 52 pairs) belonging to the records in `range`.
    pub fn images(&self, range: Range<usize>) -> Vec<ImageView> {
        let enc = self.encoding;
        let mut v = Vec::new();
        let mut i = range.start;
        while i < range.end {
            if self.rtype(i) == "50" {
                let back = self.records[i].text(enc, 25, 1) == "1";
                let mut j = i + 1;
                while j < range.end && self.rtype(j) != "52" {
                    j += 1;
                }
                if j < range.end {
                    if let Some(r) = self.image_range(j) {
                        v.push(ImageView { detail: i, data: j, back, range: r });
                    }
                    i = j;
                }
            }
            i += 1;
        }
        v
    }

    fn image_range(&self, i: usize) -> Option<Range<usize>> {
        let r = &self.records[i];
        let enc = self.encoding;
        let key = r.text(enc, 102, 4).trim().parse::<usize>().ok()?;
        let sig_at = 105 + key;
        let sig = r.text(enc, sig_at + 1, 5).trim().parse::<usize>().ok()?;
        let len_at = sig_at + 5 + sig;
        let len = r.text(enc, len_at + 1, 7).trim().parse::<usize>().ok()?;
        let start = len_at + 7;
        let end = (start + len).min(r.data.len());
        (start <= end).then_some(start..end)
    }

    pub fn image_bytes(&self, v: &ImageView) -> &[u8] {
        &self.records[v.data].data[v.range.clone()]
    }

    /// Remove an item (all of its records). Call `recalculate_controls` after.
    pub fn delete_item(&mut self, item: &Item) {
        self.records.drain(item.range.clone());
    }

    /// Recompute bundle (70), cash letter (90) and file (99) control totals.
    pub fn recalculate_controls(&mut self) {
        let enc = self.encoding;
        let outline = self.outline();
        let (mut files_items, mut files_total) = (0u64, 0u64);
        for cl in &outline {
            let (mut cl_items, mut cl_total, mut cl_images) = (0u64, 0u64, 0u64);
            for b in &cl.bundles {
                let (mut items, mut total, mut micr, mut images) = (0u64, 0u64, 0u64, 0u64);
                for it in &b.items {
                    let amt = self.amount(it.range.start);
                    items += 1;
                    total += amt;
                    if !it.is_credit && self.records[it.range.start].text(enc, 75, 1) == "1" {
                        micr += amt;
                    }
                    images += it.range.clone().filter(|&i| self.rtype(i) == "52").count() as u64;
                }
                if let Some(c) = b.control {
                    let r = &mut self.records[c];
                    r.put_num(enc, 3, 4, items);
                    r.put_num(enc, 7, 12, total);
                    r.put_num(enc, 19, 12, micr);
                    r.put_num(enc, 31, 5, images);
                }
                cl_items += items;
                cl_total += total;
                cl_images += images;
            }
            if let Some(c) = cl.control {
                let r = &mut self.records[c];
                r.put_num(enc, 3, 6, cl.bundles.len() as u64);
                r.put_num(enc, 9, 8, cl_items);
                r.put_num(enc, 17, 14, cl_total);
                r.put_num(enc, 31, 9, cl_images);
            }
            files_items += cl_items;
            files_total += cl_total;
        }
        let n = self.records.len() as u64;
        if let Some(i) = self.records.iter().rposition(|r| r.rtype(enc) == "99") {
            let r = &mut self.records[i];
            r.put_num(enc, 3, 6, outline.len() as u64);
            r.put_num(enc, 9, 8, n);
            r.put_num(enc, 17, 8, files_items);
            r.put_num(enc, 25, 16, files_total);
        }
    }

    /// Check structure and control totals; returns the problems found.
    pub fn validate(&self) -> Vec<Issue> {
        let mut issues = Vec::new();
        if self.records.is_empty() || self.rtype(0) != "01" {
            issues.push(Issue { record: 0, message: "File must start with a File Header (01)".into() });
        }
        match self.records.last() {
            Some(_) if self.rtype(self.records.len() - 1) == "99" => {}
            _ => issues.push(Issue {
                record: self.records.len().saturating_sub(1),
                message: "File must end with a File Control (99)".into(),
            }),
        }
        let mut fixed = self.clone();
        fixed.recalculate_controls();
        for (i, (a, b)) in self.records.iter().zip(&fixed.records).enumerate() {
            if a != b {
                issues.push(Issue {
                    record: i,
                    message: format!("{} totals do not match the file contents", layout::record_name(&self.rtype(i))),
                });
            }
        }
        issues
    }
}
