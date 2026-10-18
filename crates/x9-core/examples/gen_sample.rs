//! Generates a small synthetic EBCDIC X9.37 file for trying the viewer.
//! Usage: cargo run -p x9-core --example gen_sample -- out.x9 [front.tif]
use x9_core::{ebcdic, X9File};

fn e(s: &str) -> Vec<u8> {
    s.bytes().map(ebcdic::to_ebcdic).collect()
}
fn pad(s: String, n: usize) -> Vec<u8> {
    e(&format!("{:<n$}", s, n = n))
}
fn rec(out: &mut Vec<u8>, d: Vec<u8>) {
    out.extend((d.len() as u32).to_be_bytes());
    out.extend(d);
}

fn main() {
    let mut args = std::env::args().skip(1);
    let out_path = args.next().unwrap_or_else(|| "sample.x9".into());
    let img = args.next().map(|p| std::fs::read(p).expect("image")).unwrap_or_default();
    let mut o = Vec::new();
    rec(&mut o, pad("0135T123456789987654321202401011200N                 SAMPLE DEST       SAMPLE ORIGIN   1US".into(), 80));
    rec(&mut o, pad("1001123456789987654321202401012024010112001CL000001".into(), 80));
    rec(&mut o, pad("2001123456789987654321202401012024010100000000010001".into(), 80));
    for n in 1..=5u32 {
        let chk = format!("25{:<15} 12345678{}{:<20}{:010}{:<15}G  1  00 ", "", n % 10, format!("{}", 1000 + n), 12345 * n, format!("ITEM{n:04}"));
        rec(&mut o, pad(chk, 80));
        for (side, vs) in [("0", "FRONT"), ("1", "BACK")] {
            rec(&mut o, pad(format!("5001123456789202401011 00{}00 000000000000000000000000", side), 80));
            let _ = vs;
            let mut d = pad(format!("52123456789202401010{:<2}ITEM{n:04}", "1"), 101);
            d.truncate(101);
            d.extend(e("0000"));
            d.extend(e("00000"));
            d.extend(e(&format!("{:07}", img.len())));
            d.extend(&img);
            rec(&mut o, d);
        }
    }
    rec(&mut o, pad("70".into(), 80));
    rec(&mut o, pad("90".into(), 80));
    rec(&mut o, pad("99".into(), 80));
    let mut f = X9File::parse(&o).expect("parse");
    f.recalculate_controls();
    std::fs::write(&out_path, f.to_bytes()).unwrap();
    println!("wrote {out_path}: {} records, issues: {:?}", f.records.len(), f.validate());
}
