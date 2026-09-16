//! The zips the tests are shown, built one way for the unit tests and the
//! integration tests alike.
//!
//! A real document is what the conversion is measured against, and those are
//! in `tests/documents/` and are never made here. What is made here is what no
//! office application would save — a compression bomb, a document missing its
//! main part — and the smallest Word document that sets its text in a font the
//! engine carries: the one case the owner's three files cannot show, because
//! each of them loses something.

#![allow(
    dead_code,
    reason = "each test binary that includes this uses a different part of it"
)]

use miniz_oxide::deflate::compress_to_vec;

/// A zip holding these parts, each deflated, with correct checksums.
pub fn a_zip(parts: &[(&str, &[u8])]) -> Vec<u8> {
    let mut zip = Vec::new();
    let mut directory = Vec::new();
    for (name, bytes) in parts {
        let compressed = compress_to_vec(bytes, 6);
        let offset = u32::try_from(zip.len()).unwrap_or(u32::MAX);
        let crc = crc32(bytes);
        let name_len = u16::try_from(name.len()).unwrap_or(u16::MAX);
        let packed = u32::try_from(compressed.len()).unwrap_or(u32::MAX);
        let size = u32::try_from(bytes.len()).unwrap_or(u32::MAX);

        zip.extend_from_slice(&0x0403_4b50_u32.to_le_bytes());
        zip.extend_from_slice(&[20, 0, 0, 0, 8, 0, 0, 0, 0x21, 0]);
        zip.extend_from_slice(&crc.to_le_bytes());
        zip.extend_from_slice(&packed.to_le_bytes());
        zip.extend_from_slice(&size.to_le_bytes());
        zip.extend_from_slice(&name_len.to_le_bytes());
        zip.extend_from_slice(&0_u16.to_le_bytes());
        zip.extend_from_slice(name.as_bytes());
        zip.extend_from_slice(&compressed);

        directory.extend_from_slice(&0x0201_4b50_u32.to_le_bytes());
        directory.extend_from_slice(&[20, 0, 20, 0, 0, 0, 8, 0, 0, 0, 0x21, 0]);
        directory.extend_from_slice(&crc.to_le_bytes());
        directory.extend_from_slice(&packed.to_le_bytes());
        directory.extend_from_slice(&size.to_le_bytes());
        directory.extend_from_slice(&name_len.to_le_bytes());
        directory.extend_from_slice(&[0; 12]);
        directory.extend_from_slice(&offset.to_le_bytes());
        directory.extend_from_slice(name.as_bytes());
    }
    let start = u32::try_from(zip.len()).unwrap_or(u32::MAX);
    let length = u32::try_from(directory.len()).unwrap_or(u32::MAX);
    let count = u16::try_from(parts.len()).unwrap_or(u16::MAX);
    zip.extend_from_slice(&directory);
    zip.extend_from_slice(&0x0605_4b50_u32.to_le_bytes());
    zip.extend_from_slice(&[0, 0, 0, 0]);
    zip.extend_from_slice(&count.to_le_bytes());
    zip.extend_from_slice(&count.to_le_bytes());
    zip.extend_from_slice(&length.to_le_bytes());
    zip.extend_from_slice(&start.to_le_bytes());
    zip.extend_from_slice(&0_u16.to_le_bytes());
    zip
}

/// The checksum a zip keeps for each part.
fn crc32(bytes: &[u8]) -> u32 {
    let mut crc = u32::MAX;
    for byte in bytes {
        crc ^= u32::from(*byte);
        for _ in 0..8 {
            crc = if crc & 1 == 1 {
                (crc >> 1) ^ 0xEDB8_8320
            } else {
                crc >> 1
            };
        }
    }
    !crc
}

/// What the parts of a Word document are.
const CONTENT_TYPES: &[u8] = br#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types"><Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/><Default Extension="xml" ContentType="application/xml"/><Override PartName="/word/document.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml"/></Types>"#;

/// The package's relationship to its main part.
const RELATIONSHIPS: &[u8] = br#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="word/document.xml"/></Relationships>"#;

/// The smallest Word document that sets one line of text in this family.
pub fn a_word_document_in(family: &str) -> Vec<u8> {
    let document = format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:body><w:p><w:r><w:rPr><w:rFonts w:ascii="{family}" w:hAnsi="{family}"/></w:rPr><w:t>The quarterly figures are attached.</w:t></w:r></w:p></w:body></w:document>"#
    );
    a_zip(&[
        ("[Content_Types].xml", CONTENT_TYPES),
        ("_rels/.rels", RELATIONSHIPS),
        ("word/document.xml", document.as_bytes()),
    ])
}

/// A Word document whose main part decompresses past every ceiling: a
/// compression bomb that `alo-opening` still recognises by its list of parts.
pub fn a_compression_bomb() -> Vec<u8> {
    let spaces = vec![b' '; 65 * 1024 * 1024];
    a_zip(&[
        ("[Content_Types].xml", CONTENT_TYPES),
        ("_rels/.rels", RELATIONSHIPS),
        ("word/document.xml", &spaces),
    ])
}
