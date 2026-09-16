//! Content a document takes from somewhere else, read from its relationships.
//!
//! Every part of an Office document says what it refers to in a `.rels` part
//! beside it, and a reference to anything outside the file is marked
//! `TargetMode="External"`. That is how a picture linked from `C:\Users\…`, data
//! from another workbook and a template on a server all look — and ADR 0039's
//! service has no network and no files, so none of them is fetched, and each is
//! said by what it is.
//!
//! **A hyperlink is not linked content.** Nothing is fetched to show one; it is
//! an address written on the page, and the copy carries it as that.

use std::collections::BTreeSet;

use crate::carried::Linked;
use crate::xml::{self, NotXml};
use crate::zip::{NotRead, Zipped};

/// Why the relationships could not be read.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NotLinked {
    /// A part could not be read out of the zip.
    Zip(NotRead),
    /// A part was not XML.
    Xml(NotXml),
}

/// Every kind of linked content a document's relationships name.
///
/// # Errors
/// [`NotLinked`].
pub fn linked(zipped: &mut Zipped<'_>) -> Result<BTreeSet<Linked>, NotLinked> {
    let parts: Vec<String> = zipped
        .names()
        .filter(|name| name.ends_with(".rels"))
        .map(str::to_owned)
        .collect();
    let mut found = BTreeSet::new();
    for part in parts {
        let Some(bytes) = zipped.read(&part).map_err(NotLinked::Zip)? else {
            continue;
        };
        for read in xml::read(&bytes).map_err(NotLinked::Xml)? {
            if !read.opens("Relationship") || read.attribute("TargetMode") != Some("External") {
                continue;
            }
            if let Some(kind) = read.attribute("Type").and_then(what_it_is) {
                found.insert(kind);
            }
        }
    }
    Ok(found)
}

/// What a relationship of this type links to, or [`None`] for a hyperlink.
fn what_it_is(relationship: &str) -> Option<Linked> {
    let named = relationship.rsplit('/').next().unwrap_or(relationship);
    match named {
        "hyperlink" => None,
        "image" => Some(Linked::Picture),
        "attachedTemplate" => Some(Linked::Template),
        "oleObject" | "package" | "externalLinkPath" | "externalLink" | "recipientData"
        | "dataSource" | "mailMergeSource" => Some(Linked::Data),
        _ => Some(Linked::SomethingElse),
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::testing::{a_zip, the_document};

    /// **The picture the owner's document links from their own disk is found**,
    /// and the two documents that link nothing link nothing.
    #[test]
    fn the_linked_picture_in_a_real_document_is_found() {
        let bytes = the_document("sample.docx");
        let mut zipped = Zipped::of(&bytes).unwrap();
        assert_eq!(
            linked(&mut zipped).unwrap(),
            BTreeSet::from([Linked::Picture])
        );
        for other in ["sample.xlsx", "sample.pptx"] {
            let bytes = the_document(other);
            let mut zipped = Zipped::of(&bytes).unwrap();
            assert_eq!(linked(&mut zipped).unwrap(), BTreeSet::new(), "{other}");
        }
    }

    /// A hyperlink is not linked content, and each other kind is said by what
    /// it is.
    #[test]
    fn a_hyperlink_is_not_linked_content() {
        let relationships = br#"<Relationships>
            <Relationship Id="1" Type="http://x/relationships/hyperlink" Target="https://example.org" TargetMode="External"/>
            <Relationship Id="2" Type="http://x/relationships/attachedTemplate" Target="file:///n/t.dotm" TargetMode="External"/>
            <Relationship Id="3" Type="http://x/relationships/oleObject" Target="file:///n/d.xlsx" TargetMode="External"/>
            <Relationship Id="4" Type="http://x/relationships/video" Target="https://example.org/v" TargetMode="External"/>
            <Relationship Id="5" Type="http://x/relationships/image" Target="media/image1.png"/>
        </Relationships>"#;
        let bytes = a_zip(&[("word/_rels/document.xml.rels", relationships)]);
        let mut zipped = Zipped::of(&bytes).unwrap();
        assert_eq!(
            linked(&mut zipped).unwrap(),
            BTreeSet::from([Linked::Data, Linked::Template, Linked::SomethingElse])
        );
    }

    /// Relationships that are not XML end the inventory.
    #[test]
    fn relationships_that_are_not_xml_are_not_read_past() {
        let bytes = a_zip(&[(
            "_rels/.rels",
            b"<Relationships><Relationship></Relationships>",
        )]);
        let mut zipped = Zipped::of(&bytes).unwrap();
        assert!(matches!(linked(&mut zipped), Err(NotLinked::Xml(_))));
    }
}
