//! Field layouts for the standard X9.37 / X9.100-187 record types.
//!
//! Positions are 1-based, as written in the standard. Layouts are applied
//! leniently: fields extending past the end of a record are clipped.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Numeric,
    Alpha,
    Amount,
    Date,
    Flag,
}

#[derive(Clone, Copy, Debug)]
pub struct FieldDef {
    pub name: &'static str,
    pub start: usize,
    pub len: usize,
    pub kind: Kind,
}

const fn f(name: &'static str, start: usize, len: usize, kind: Kind) -> FieldDef {
    FieldDef { name, start, len, kind }
}
use Kind::*;

pub const FILE_HEADER: &[FieldDef] = &[
    f("Record Type", 1, 2, Numeric),
    f("Standard Level", 3, 2, Numeric),
    f("Test File Indicator", 5, 1, Flag),
    f("Immediate Destination Routing", 6, 9, Numeric),
    f("Immediate Origin Routing", 15, 9, Numeric),
    f("File Creation Date", 24, 8, Date),
    f("File Creation Time", 32, 4, Numeric),
    f("Resend Indicator", 36, 1, Flag),
    f("Immediate Destination Name", 37, 18, Alpha),
    f("Immediate Origin Name", 55, 18, Alpha),
    f("File ID Modifier", 73, 1, Alpha),
    f("Country Code", 74, 2, Alpha),
    f("User Field", 76, 4, Alpha),
    f("Reserved", 80, 1, Alpha),
];

pub const CASH_LETTER_HEADER: &[FieldDef] = &[
    f("Record Type", 1, 2, Numeric),
    f("Collection Type Indicator", 3, 2, Numeric),
    f("Destination Routing", 5, 9, Numeric),
    f("ECE Institution Routing", 14, 9, Numeric),
    f("Cash Letter Business Date", 23, 8, Date),
    f("Cash Letter Creation Date", 31, 8, Date),
    f("Cash Letter Creation Time", 39, 4, Numeric),
    f("Record Type Indicator", 43, 1, Flag),
    f("Documentation Type Indicator", 44, 1, Flag),
    f("Cash Letter ID", 45, 8, Alpha),
    f("Originator Contact Name", 53, 14, Alpha),
    f("Originator Contact Phone", 67, 10, Alpha),
    f("Fed Work Type", 77, 1, Flag),
    f("User Field", 78, 2, Alpha),
    f("Reserved", 80, 1, Alpha),
];

pub const BUNDLE_HEADER: &[FieldDef] = &[
    f("Record Type", 1, 2, Numeric),
    f("Collection Type Indicator", 3, 2, Numeric),
    f("Destination Routing", 5, 9, Numeric),
    f("ECE Institution Routing", 14, 9, Numeric),
    f("Bundle Business Date", 23, 8, Date),
    f("Bundle Creation Date", 31, 8, Date),
    f("Bundle ID", 39, 10, Alpha),
    f("Bundle Sequence Number", 49, 4, Alpha),
    f("Cycle Number", 53, 2, Alpha),
    f("Return Location Routing", 55, 9, Numeric),
    f("User Field", 64, 5, Alpha),
    f("Reserved", 69, 12, Alpha),
];

pub const CHECK_DETAIL: &[FieldDef] = &[
    f("Record Type", 1, 2, Numeric),
    f("Auxiliary On-Us", 3, 15, Alpha),
    f("External Processing Code", 18, 1, Flag),
    f("Payor Bank Routing", 19, 8, Numeric),
    f("Payor Check Digit", 27, 1, Numeric),
    f("On-Us", 28, 20, Alpha),
    f("Item Amount", 48, 10, Amount),
    f("ECE Item Sequence Number", 58, 15, Alpha),
    f("Documentation Type Indicator", 73, 1, Flag),
    f("Return Acceptance Indicator", 74, 1, Flag),
    f("MICR Valid Indicator", 75, 1, Flag),
    f("BOFD Indicator", 76, 1, Flag),
    f("Addendum Count", 77, 2, Numeric),
    f("Correction Indicator", 79, 1, Flag),
    f("Archive Type Indicator", 80, 1, Flag),
];

pub const ADDENDUM_A: &[FieldDef] = &[
    f("Record Type", 1, 2, Numeric),
    f("Addendum Record Number", 3, 1, Numeric),
    f("Return Location Routing", 4, 9, Numeric),
    f("BOFD Routing", 13, 9, Numeric),
    f("BOFD Business Date", 22, 8, Date),
    f("BOFD Item Sequence Number", 30, 15, Alpha),
    f("Deposit Account at BOFD", 45, 18, Alpha),
    f("BOFD Deposit Branch", 63, 5, Alpha),
    f("Payee Name", 68, 13, Alpha),
];

pub const ADDENDUM_B: &[FieldDef] = &[
    f("Record Type", 1, 2, Numeric),
    f("Image Reference Key Indicator", 3, 1, Flag),
    f("Microfilm Archive Sequence Number", 4, 15, Alpha),
    f("Length of Image Reference Key", 19, 4, Numeric),
    f("Image Reference Key", 23, 34, Alpha),
    f("Description", 57, 15, Alpha),
    f("User Field", 72, 4, Alpha),
];

pub const ADDENDUM_C: &[FieldDef] = &[
    f("Record Type", 1, 2, Numeric),
    f("Addendum Record Number", 3, 2, Numeric),
    f("Endorsing Bank Routing", 5, 9, Numeric),
    f("Endorsement Business Date", 14, 8, Date),
    f("Endorsing Bank Item Sequence", 22, 15, Alpha),
    f("Truncation Indicator", 37, 1, Flag),
    f("Conversion Indicator", 38, 1, Flag),
    f("Correction Indicator", 39, 1, Flag),
    f("Return Reason", 40, 1, Flag),
    f("User Field", 41, 19, Alpha),
    f("Endorsing Bank Identifier", 60, 1, Flag),
];

pub const IMAGE_VIEW_DETAIL: &[FieldDef] = &[
    f("Record Type", 1, 2, Numeric),
    f("Image Indicator", 3, 1, Flag),
    f("Image Creator Routing", 4, 9, Numeric),
    f("Image Creator Date", 13, 8, Date),
    f("Image View Format Indicator", 21, 2, Numeric),
    f("Compression Algorithm", 23, 2, Numeric),
    f("View Side Indicator", 25, 1, Flag),
    f("View Descriptor", 26, 2, Numeric),
    f("Digital Signature Indicator", 28, 1, Flag),
    f("Digital Signature Method", 29, 2, Numeric),
    f("Security Key Size", 31, 5, Numeric),
    f("Start of Protected Data", 36, 7, Numeric),
    f("Length of Protected Data", 43, 7, Numeric),
    f("Image Recreate Indicator", 50, 1, Flag),
    f("User Field", 51, 8, Alpha),
];

pub const IMAGE_VIEW_DATA: &[FieldDef] = &[
    f("Record Type", 1, 2, Numeric),
    f("ECE Institution Routing", 3, 9, Numeric),
    f("Bundle Business Date", 12, 8, Date),
    f("Cycle Number", 20, 2, Alpha),
    f("ECE Item Sequence Number", 22, 15, Alpha),
    f("Security Originator Name", 37, 16, Alpha),
    f("Security Authenticator Name", 53, 16, Alpha),
    f("Security Key Name", 69, 16, Alpha),
    f("Clipping Origin", 85, 1, Flag),
    f("Clipping Coordinate h1", 86, 4, Numeric),
    f("Clipping Coordinate h2", 90, 4, Numeric),
    f("Clipping Coordinate v1", 94, 4, Numeric),
    f("Clipping Coordinate v2", 98, 4, Numeric),
    f("Length of Image Reference Key", 102, 4, Numeric),
];

pub const CREDIT: &[FieldDef] = &[
    f("Record Type", 1, 2, Numeric),
    f("Auxiliary On-Us", 3, 15, Alpha),
    f("External Processing Code", 18, 1, Flag),
    f("Posting Bank Routing", 19, 9, Numeric),
    f("On-Us", 28, 20, Alpha),
    f("Item Amount", 48, 10, Amount),
    f("ECE Item Sequence Number", 58, 15, Alpha),
    f("Documentation Type Indicator", 73, 1, Flag),
    f("Type of Account Code", 74, 1, Flag),
    f("Source of Work", 75, 1, Flag),
];

pub const BUNDLE_CONTROL: &[FieldDef] = &[
    f("Record Type", 1, 2, Numeric),
    f("Items Within Bundle", 3, 4, Numeric),
    f("Bundle Total Amount", 7, 12, Amount),
    f("MICR Valid Total Amount", 19, 12, Amount),
    f("Images Within Bundle", 31, 5, Numeric),
    f("User Field", 36, 20, Alpha),
];

pub const CASH_LETTER_CONTROL: &[FieldDef] = &[
    f("Record Type", 1, 2, Numeric),
    f("Bundle Count", 3, 6, Numeric),
    f("Items Within Cash Letter", 9, 8, Numeric),
    f("Cash Letter Total Amount", 17, 14, Amount),
    f("Images Within Cash Letter", 31, 9, Numeric),
    f("ECE Institution Name", 40, 18, Alpha),
    f("Settlement Date", 58, 8, Date),
];

pub const FILE_CONTROL: &[FieldDef] = &[
    f("Record Type", 1, 2, Numeric),
    f("Cash Letter Count", 3, 6, Numeric),
    f("Total Record Count", 9, 8, Numeric),
    f("Total Item Count", 17, 8, Numeric),
    f("File Total Amount", 25, 16, Amount),
    f("Immediate Origin Contact Name", 41, 14, Alpha),
    f("Immediate Origin Contact Phone", 55, 10, Alpha),
];

/// Human readable name of a record type.
pub fn record_name(rtype: &str) -> &'static str {
    match rtype {
        "01" => "File Header",
        "10" => "Cash Letter Header",
        "20" => "Bundle Header",
        "25" => "Check Detail",
        "26" => "Check Addendum A",
        "28" => "Check Addendum B",
        "31" => "Check Addendum C",
        "50" => "Image View Detail",
        "52" => "Image View Data",
        "61" => "Credit Detail",
        "62" => "Credit Addendum A",
        "68" => "User Record",
        "70" => "Bundle Control",
        "75" => "Box Summary",
        "85" => "Routing Number Summary",
        "90" => "Cash Letter Control",
        "99" => "File Control",
        _ => "Unknown Record",
    }
}

/// Field layout for a record type (empty for unknown types).
pub fn layout_for(rtype: &str) -> &'static [FieldDef] {
    match rtype {
        "01" => FILE_HEADER,
        "10" => CASH_LETTER_HEADER,
        "20" => BUNDLE_HEADER,
        "25" => CHECK_DETAIL,
        "26" => ADDENDUM_A,
        "28" => ADDENDUM_B,
        "31" => ADDENDUM_C,
        "50" => IMAGE_VIEW_DETAIL,
        "52" => IMAGE_VIEW_DATA,
        "61" => CREDIT,
        "70" => BUNDLE_CONTROL,
        "90" => CASH_LETTER_CONTROL,
        "99" => FILE_CONTROL,
        _ => &[],
    }
}
