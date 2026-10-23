use x9_core::*;

fn pad(s: &str, n: usize) -> String {
    format!("{:<width$}", s, width = n)
}

fn enc(enc: Encoding, s: &str) -> Vec<u8> {
    s.bytes().map(|b| enc.encode(b)).collect()
}

fn rec(e: Encoding, out: &mut Vec<u8>, data: Vec<u8>) {
    out.extend((data.len() as u32).to_be_bytes());
    out.extend(data);
    let _ = e;
}

fn sample(e: Encoding) -> Vec<u8> {
    let mut o = Vec::new();
    rec(e, &mut o, enc(e, &pad("0135T123456789987654321202401011200N", 80)));
    rec(e, &mut o, enc(e, &pad("1001123456789987654321202401012024010112001", 80)));
    rec(e, &mut o, enc(e, &pad("2001123456789987654321202401012024010100000000010001", 80)));
    let mut chk = String::from("25");
    chk += &pad("", 15);
    chk += " 12345678";
    chk += "9";
    chk += &pad("1234", 20);
    chk += "0000012345";
    chk += &pad("SEQ1", 15);
    chk += "G";
    chk += " 1  00 ";
    rec(e, &mut o, enc(e, &pad(&chk, 80)));
    rec(e, &mut o, enc(e, &pad("5001", 80)));
    let img = b"II*\0fakeimage";
    let mut d = enc(e, &pad("52", 101));
    d.extend(enc(e, "0000"));
    d.extend(enc(e, "00000"));
    d.extend(enc(e, &format!("{:07}", img.len())));
    d.extend(img);
    rec(e, &mut o, d);
    rec(e, &mut o, enc(e, &pad("70", 80)));
    rec(e, &mut o, enc(e, &pad("90", 80)));
    rec(e, &mut o, enc(e, &pad("99", 80)));
    o
}

#[test]
fn roundtrip_both_encodings() {
    for e in [Encoding::Ascii, Encoding::Ebcdic] {
        let bytes = sample(e);
        let f = X9File::parse(&bytes).unwrap();
        assert_eq!(f.encoding, e);
        assert_eq!(f.to_bytes(), bytes);
        assert_eq!(f.records.len(), 9);
        assert_eq!(f.rtype(3), "25");
    }
}

#[test]
fn outline_images_and_recalc() {
    let mut f = X9File::parse(&sample(Encoding::Ebcdic)).unwrap();
    let o = f.outline();
    assert_eq!(o.len(), 1);
    assert_eq!(o[0].bundles[0].items.len(), 1);
    let item = o[0].bundles[0].items[0].clone();
    assert_eq!(item.range, 3..6);
    assert_eq!(f.amount(3), 12345);
    let imgs = f.images(item.range.clone());
    assert_eq!(imgs.len(), 1);
    assert_eq!(f.image_bytes(&imgs[0]), b"II*\0fakeimage");
    assert!(!f.validate().is_empty());
    f.recalculate_controls();
    assert!(f.validate().is_empty(), "{:?}", f.validate());
    let ctl = &f.outline()[0].bundles[0];
    assert_eq!(f.records[ctl.control.unwrap()].text(f.encoding, 3, 4), "0001");
    f.delete_item(&item);
    f.recalculate_controls();
    assert_eq!(f.outline()[0].bundles[0].items.len(), 0);
    assert!(f.validate().is_empty());
}

#[test]
fn set_field_pads() {
    let mut f = X9File::parse(&sample(Encoding::Ebcdic)).unwrap();
    let def = *f.fields(3).iter().find(|d| d.name == "Item Amount").unwrap();
    f.set_field(3, &def, "250");
    assert_eq!(f.field_text(3, &def), "0000000250");
    let def = *f.fields(3).iter().find(|d| d.name == "On-Us").unwrap();
    f.set_field(3, &def, "abc");
    assert_eq!(f.field_text(3, &def), format!("{:<20}", "abc"));
}

#[test]
fn rejects_garbage() {
    assert!(X9File::parse(b"hello world").is_err());
    assert!(X9File::parse(&[]).is_err());
}

#[test]
fn alternate_framings_roundtrip() {
    use x9_core::Framing;
    for e in [Encoding::Ascii, Encoding::Ebcdic] {
        let mut f = X9File::parse(&sample(e)).unwrap();
        for fr in [Framing::LittleEndian, Framing::IbmBlocked] {
            f.framing = fr;
            let bytes = f.to_bytes();
            let g = X9File::parse(&bytes).unwrap();
            assert_eq!(g.framing, fr);
            assert_eq!(g.encoding, e);
            assert_eq!(g.records.len(), f.records.len());
            assert_eq!(g.to_bytes(), bytes);
        }
    }
}
