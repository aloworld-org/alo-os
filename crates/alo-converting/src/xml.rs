//! A document part's XML, as the few things an inventory asks of it.
//!
//! The one file in this crate that names the XML reader (ADR 0039 §5). What the
//! inventories need is small — an element's local name, its attributes by local
//! name, and whether text is inside it — so that is the whole of what is handed
//! on, and a namespace prefix a document chose (`w:`, `a:`, none at all) never
//! decides anything.
//!
//! **A part that is not well-formed ends the inventory.** An inventory that
//! skipped what it could not read would be one that says *lost nothing* about a
//! document it did not finish reading.

use quick_xml::events::{BytesStart, Event};
use quick_xml::{Reader, XmlVersion};

/// Why a part's XML could not be read.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
#[error("a part of the document is not XML that could be read")]
pub struct NotXml;

/// One thing read from a part.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Read {
    /// An element opened; `empty` when it closed in the same tag.
    Opened {
        /// Its local name, without any prefix.
        name: String,
        /// Its attributes, by local name, with their values unescaped.
        attributes: Vec<(String, String)>,
        /// Whether it closed where it opened.
        empty: bool,
    },
    /// An element closed.
    Closed {
        /// Its local name.
        name: String,
    },
    /// Text inside an element, unescaped where it could be.
    Text(String),
}

impl Read {
    /// The value of an attribute, by local name, when this opened an element.
    #[must_use]
    pub fn attribute(&self, wanted: &str) -> Option<&str> {
        match self {
            Self::Opened { attributes, .. } => attributes
                .iter()
                .find(|(name, _)| name == wanted)
                .map(|(_, value)| value.as_str()),
            _ => None,
        }
    }

    /// Whether this opened an element of this local name.
    #[must_use]
    pub fn opens(&self, wanted: &str) -> bool {
        matches!(self, Self::Opened { name, .. } if name == wanted)
    }

    /// Whether this closed an element of this local name, including one that
    /// closed where it opened.
    #[must_use]
    pub fn closes(&self, wanted: &str) -> bool {
        match self {
            Self::Closed { name } => name == wanted,
            Self::Opened { name, empty, .. } => *empty && name == wanted,
            Self::Text(_) => false,
        }
    }
}

/// Everything read from a part, in order.
///
/// # Errors
/// [`NotXml`] when the part is not well-formed, or not UTF-8.
pub fn read(bytes: &[u8]) -> Result<Vec<Read>, NotXml> {
    let text = std::str::from_utf8(bytes).map_err(|_| NotXml)?;
    let mut reader = Reader::from_str(text);
    let mut read = Vec::new();
    loop {
        match reader.read_event().map_err(|_| NotXml)? {
            Event::Start(start) => read.push(opened(&start, false)?),
            Event::Empty(start) => read.push(opened(&start, true)?),
            Event::End(end) => read.push(Read::Closed {
                name: local(end.local_name().as_ref())?,
            }),
            Event::Text(text) => {
                let decoded = text.xml10_content().map_err(|_| NotXml)?;
                read.push(Read::Text(decoded.into_owned()));
            }
            Event::CData(data) => {
                let decoded = data.decode().map_err(|_| NotXml)?;
                read.push(Read::Text(decoded.into_owned()));
            }
            // `&amp;` and its kind arrive on their own; what an inventory asks
            // of text is whether there is any, and a reference is some.
            Event::GeneralRef(_) => read.push(Read::Text("&".to_owned())),
            Event::Eof => return Ok(read),
            Event::Comment(_) | Event::Decl(_) | Event::PI(_) | Event::DocType(_) => {}
        }
    }
}

/// Where in a part a walk through it is: the elements open around the next
/// thing read.
#[derive(Debug, Clone, Default)]
pub struct Walk {
    /// The open elements, outermost first.
    open: Vec<String>,
}

impl Walk {
    /// The element directly around the next thing read.
    #[must_use]
    pub fn parent(&self) -> Option<&str> {
        self.open.last().map(String::as_str)
    }

    /// The element around that one.
    #[must_use]
    pub fn grandparent(&self) -> Option<&str> {
        self.open
            .len()
            .checked_sub(2)
            .and_then(|at| self.open.get(at))
            .map(String::as_str)
    }

    /// Whether an element of this name is open anywhere around.
    #[must_use]
    pub fn within(&self, name: &str) -> bool {
        self.open.iter().any(|open| open == name)
    }

    /// Move past one thing read. Asked **after** the thing has been looked at,
    /// so that while it is, [`Walk::parent`] is what is around it.
    pub fn past(&mut self, read: &Read) {
        match read {
            Read::Opened {
                name, empty: false, ..
            } => self.open.push(name.clone()),
            Read::Closed { .. } => {
                self.open.pop();
            }
            Read::Opened { .. } | Read::Text(_) => {}
        }
    }
}

/// An opened element, as a [`Read`].
fn opened(start: &BytesStart<'_>, empty: bool) -> Result<Read, NotXml> {
    let mut attributes = Vec::new();
    for attribute in start.attributes() {
        let attribute = attribute.map_err(|_| NotXml)?;
        let name = local(attribute.key.local_name().as_ref())?;
        let value = attribute
            .normalized_value(XmlVersion::Implicit1_0)
            .map_err(|_| NotXml)?;
        attributes.push((name, value.into_owned()));
    }
    Ok(Read::Opened {
        name: local(start.local_name().as_ref())?,
        attributes,
        empty,
    })
}

/// A local name as text.
fn local(name: &[u8]) -> Result<String, NotXml> {
    std::str::from_utf8(name)
        .map(str::to_owned)
        .map_err(|_| NotXml)
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;

    /// Prefixes are dropped and attributes are found by their local names.
    #[test]
    fn prefixes_never_decide_anything() {
        let read = read(
            br#"<w:r xmlns:w="x"><w:rFonts w:ascii="Garamond"/><w:t>Hi &amp; bye</w:t></w:r>"#,
        )
        .unwrap();
        assert!(read.first().unwrap().opens("r"));
        assert_eq!(read.get(1).unwrap().attribute("ascii"), Some("Garamond"));
        assert!(read.get(1).unwrap().closes("rFonts"));
        assert!(
            read.iter()
                .any(|one| matches!(one, Read::Text(text) if text == "Hi "))
        );
        assert!(read.last().unwrap().closes("r"));
    }

    /// **A part that is not well-formed is not half-read.**
    #[test]
    fn a_part_that_is_not_well_formed_is_refused() {
        assert_eq!(read(b"<a><b></a>"), Err(NotXml));
        assert_eq!(read(b"<a x=\"1></a>"), Err(NotXml));
        assert_eq!(read(b"\xff\xfe<a/>"), Err(NotXml));
    }
}
