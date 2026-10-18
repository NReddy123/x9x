# x9x — a fast, free X9 viewer & editor

Native Rust desktop app built on [GPUI](https://www.gpui.rs/) and the
[GPUI Kit](https://gpui-kit.com) component library (`gpui-kit` / `gpui-component`).

## What is X9?

X9 (ANSI **X9.37**, today **X9.100-187**) is the binary interchange format banks use for
*Check 21* image cash letters: scanned check images plus MICR/amount data.

* A file is a sequence of records, each prefixed by a 4-byte big-endian **RDW** (record length).
* Text is **EBCDIC** (usual) or ASCII; image payloads are raw binary (CCITT G4 TIFF, sometimes JPEG).
* Hierarchy: `01` File Header › `10` Cash Letter Header › `20` Bundle Header ›
  items (`25` Check Detail, `26/28/31` addenda, `50` Image View Detail + `52` Image View Data per side) ›
  `70` Bundle Control › `90` Cash Letter Control › `99` File Control.
* Control records carry item counts/totals that must match the contents.
* Record `52` embeds: length-prefixed image reference key, digital signature and image data
  (4/5/7-digit text length fields).

## Layout

* `crates/x9-core` — parser, EBCDIC/ASCII handling, field layouts, outline (cash letters › bundles › items),
  image extraction, field editing, delete item, control-total recalculation, validation, lossless writer.
  `cargo test -p x9-core`; `cargo run -p x9-core --example gen_sample -- out.x9 front.tif` makes a demo file.
* `crates/x9-viewer` — GPUI app: virtualized navigation tree, async image decoding with zoom
  and front/back toggle, record/field editor, Open/Save/Save As, Validate, Recalc totals, image export, dark mode.

## Run

```
cargo run --release -p x9-viewer -- path/to/file.x9
```

Linux needs `libfontconfig1-dev libxkbcommon-x11-dev libwayland-dev libxcb1-dev libssl-dev`, and a Vulkan/GL-capable GPU.

Note: the field layouts follow the public X9.37 record structure; some fields of less common records
(26, 61, 62) are best-effort and unmapped bytes are always preserved on save.
`vendor/xattr` is a one-line patch needed for gpui's `zed-async-tar` dependency to build on current `libc`.
