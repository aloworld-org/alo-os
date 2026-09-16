//! A Word document, inventoried: the fonts its text is set in, its fields, its
//! comments and its tracked changes.
//!
//! # How a run of text finds its font
//!
//! In the order Word applies them, stopping at the first that names one: the
//! run's own fonts; the character style it is given, and the styles that one is
//! based on; the paragraph's style (or the document's default paragraph style)
//! and the styles it is based on; the document's defaults. A font given as the
//! theme's heading or body font is the family the theme names. The Latin
//! family is the one read — `ascii`, then `hAnsi` — because that is what the
//! text of the three formats' common case is set in.
//!
//! **Where it stops following**: table styles, and a run whose chain names no
//! font at all. Neither adds a family, so neither can report a loss that is not
//! there; what they could miss is said in this crate's report.
//!
//! # Fields
//!
//! A field's instruction is read from `fldSimple` and from the `instrText`
//! between a `begin` and a `separate`, which Word splits across runs whenever
//! it likes. The first word says what it is.

use std::collections::HashMap;

use crate::carried::Field;
use crate::inventory::original::{NotInventoried, Original};
use crate::inventory::theme::{Slot, Theme, shows};
use crate::xml::{self, Read, Walk};
use crate::zip::Zipped;

/// The part a Word document cannot be without.
const THE_DOCUMENT: &str = "word/document.xml";

/// The deepest chain of styles followed.
const DEEPEST_STYLES: usize = 32;

/// A font as a run or a style gives it.
#[derive(Debug, Clone, PartialEq, Eq)]
enum Given {
    /// A family by name.
    Named(String),
    /// One of the theme's two.
    Theme(Slot),
}

/// One style.
#[derive(Debug, Clone, Default)]
struct Style {
    /// The style it is based on.
    based_on: Option<String>,
    /// The font it gives.
    font: Option<Given>,
}

/// A document's styles.
#[derive(Debug, Clone, Default)]
struct Styles {
    /// The font the document's defaults give.
    defaults: Option<Given>,
    /// Every style, by its id.
    by_id: HashMap<String, Style>,
    /// The default paragraph style's id.
    default_paragraph: Option<String>,
}

/// Inventory a Word document into `original`.
///
/// # Errors
/// [`NotInventoried`].
pub fn inventory(zipped: &mut Zipped<'_>, original: &mut Original) -> Result<(), NotInventoried> {
    if !zipped.holds(THE_DOCUMENT) {
        return Err(NotInventoried::Missing("main document part"));
    }
    let theme = match first_named(zipped, |name| {
        name.starts_with("word/theme/") && name.ends_with(".xml")
    }) {
        Some(name) => zipped
            .read(&name)?
            .map(|bytes| Theme::of(&bytes))
            .transpose()?
            .unwrap_or_default(),
        None => Theme::default(),
    };
    let styles = match zipped.read("word/styles.xml")? {
        Some(bytes) => styles(&xml::read(&bytes)?),
        None => Styles::default(),
    };

    let mut parts: Vec<String> = zipped
        .names()
        .filter(|name| is_text_part(name))
        .map(str::to_owned)
        .collect();
    parts.sort();
    for part in parts {
        if let Some(bytes) = zipped.read(&part)? {
            text_part(&xml::read(&bytes)?, &styles, &theme, original)?;
        }
    }

    if let Some(bytes) = zipped.read("word/comments.xml")?
        && xml::read(&bytes)?.iter().any(|read| read.opens("comment"))
    {
        original.has_comments();
    }
    Ok(())
}

/// Whether a part holds text that is shown on a page.
fn is_text_part(name: &str) -> bool {
    let Some(file) = name.strip_prefix("word/") else {
        return false;
    };
    if file.contains('/') {
        return false;
    }
    file == "document.xml"
        || file == "footnotes.xml"
        || file == "endnotes.xml"
        || ((file.starts_with("header") || file.starts_with("footer")) && file.ends_with(".xml"))
}

/// The first part, by name, that a rule picks.
fn first_named(zipped: &Zipped<'_>, picks: impl Fn(&str) -> bool) -> Option<String> {
    let mut names: Vec<&str> = zipped.names().filter(|name| picks(name)).collect();
    names.sort_unstable();
    names.first().map(|name| (*name).to_owned())
}

/// The font an `rFonts` element gives.
fn given(read: &Read) -> Option<Given> {
    if let Some(theme) = read
        .attribute("asciiTheme")
        .or_else(|| read.attribute("hAnsiTheme"))
    {
        if theme.starts_with("major") {
            return Some(Given::Theme(Slot::Major));
        }
        if theme.starts_with("minor") {
            return Some(Given::Theme(Slot::Minor));
        }
    }
    read.attribute("ascii")
        .or_else(|| read.attribute("hAnsi"))
        .map(|named| Given::Named(named.to_owned()))
}

/// A styles part, read.
fn styles(read: &[Read]) -> Styles {
    let mut styles = Styles::default();
    let mut walk = Walk::default();
    let mut current: Option<(String, Style)> = None;
    for one in read {
        if one.opens("style") {
            let id = one.attribute("styleId").unwrap_or_default().to_owned();
            if one.attribute("type") == Some("paragraph")
                && matches!(one.attribute("default"), Some("1" | "true" | "on"))
            {
                styles.default_paragraph = Some(id.clone());
            }
            current = Some((id, Style::default()));
        } else if one.closes("style") {
            if let Some((id, style)) = current.take() {
                styles.by_id.insert(id, style);
            }
        } else if one.opens("basedOn") {
            if let Some((_, style)) = current.as_mut() {
                style.based_on = one.attribute("val").map(str::to_owned);
            }
        } else if one.opens("rFonts") && walk.parent() == Some("rPr") {
            match walk.grandparent() {
                Some("style") => {
                    if let Some((_, style)) = current.as_mut() {
                        style.font = given(one);
                    }
                }
                Some("rPrDefault") => styles.defaults = given(one),
                _ => {}
            }
        }
        walk.past(one);
    }
    styles
}

impl Styles {
    /// The font a style gives, following what it is based on.
    fn font_of(&self, id: Option<&str>) -> Option<&Given> {
        let mut id = id?;
        for _ in 0..DEEPEST_STYLES {
            let style = self.by_id.get(id)?;
            if let Some(font) = &style.font {
                return Some(font);
            }
            id = style.based_on.as_deref()?;
        }
        None
    }
}

/// One run being read.
#[derive(Debug, Default)]
struct Run {
    /// Its own font.
    font: Option<Given>,
    /// Its character style.
    style: Option<String>,
    /// Whether it has text that shows.
    shows: bool,
}

/// A part with text in it, read into `original`.
fn text_part(
    read: &[Read],
    styles: &Styles,
    theme: &Theme,
    original: &mut Original,
) -> Result<(), NotInventoried> {
    let mut walk = Walk::default();
    let mut paragraph_style: Option<String> = None;
    let mut run: Option<Run> = None;
    let mut fields: Vec<(String, bool)> = Vec::new();

    for one in read {
        match one {
            Read::Opened { name, .. } => match name.as_str() {
                "p" => paragraph_style = None,
                "pStyle" if walk.parent() == Some("pPr") => {
                    paragraph_style = one.attribute("val").map(str::to_owned);
                }
                "r" => run = Some(Run::default()),
                "rFonts" if walk.parent() == Some("rPr") && walk.grandparent() == Some("r") => {
                    if let Some(run) = run.as_mut() {
                        run.font = given(one);
                    }
                }
                "rStyle" if walk.parent() == Some("rPr") && walk.grandparent() == Some("r") => {
                    if let Some(run) = run.as_mut() {
                        run.style = one.attribute("val").map(str::to_owned);
                    }
                }
                "fldSimple" => {
                    if let Some(field) = one.attribute("instr").and_then(field_of) {
                        original.has_field(field);
                    }
                }
                "fldChar" => match one.attribute("fldCharType") {
                    Some("begin") => fields.push((String::new(), false)),
                    Some("separate") => {
                        if let Some((instruction, done)) = fields.last_mut()
                            && !*done
                        {
                            *done = true;
                            if let Some(field) = field_of(instruction) {
                                original.has_field(field);
                            }
                        }
                    }
                    Some("end") => {
                        if let Some((instruction, done)) = fields.pop()
                            && !done
                            && let Some(field) = field_of(&instruction)
                        {
                            original.has_field(field);
                        }
                    }
                    _ => {}
                },
                "ins" | "del" | "moveFrom" | "moveTo" => original.has_tracked_changes(),
                _ => {}
            },
            Read::Text(text) => match walk.parent() {
                Some("t") if shows(one) => {
                    if let Some(run) = run.as_mut() {
                        run.shows = true;
                    }
                }
                Some("instrText") => {
                    if let Some((instruction, false)) = fields.last_mut() {
                        instruction.push_str(text);
                    }
                }
                _ => {}
            },
            Read::Closed { .. } => {}
        }
        if one.closes("r")
            && let Some(done) = run.take()
            && done.shows
        {
            let font = done
                .font
                .as_ref()
                .or_else(|| styles.font_of(done.style.as_deref()))
                .or_else(|| {
                    styles.font_of(
                        paragraph_style
                            .as_deref()
                            .or(styles.default_paragraph.as_deref()),
                    )
                })
                .or(styles.defaults.as_ref());
            let family = match font {
                Some(Given::Named(family)) => Some(family.as_str()),
                Some(Given::Theme(slot)) => theme.family(*slot),
                None => None,
            };
            if let Some(family) = family {
                original.sets_text_in(family)?;
            }
        }
        walk.past(one);
    }
    Ok(())
}

/// What a field instruction is, when it is one whose value depends on when or
/// where the document is open.
fn field_of(instruction: &str) -> Option<Field> {
    let first = instruction.split_whitespace().next()?.to_ascii_uppercase();
    match first.as_str() {
        "DATE" => Some(Field::Date),
        "TIME" => Some(Field::Time),
        "FILENAME" => Some(Field::FileName),
        "AUTHOR" | "USERNAME" | "USERINITIALS" | "LASTSAVEDBY" => Some(Field::Author),
        "MERGEFIELD" => Some(Field::MergeField),
        _ => None,
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::testing::a_zip;

    /// A minimal document part around some body XML.
    fn a_document(body: &str) -> Vec<u8> {
        format!("<w:document xmlns:w=\"w\"><w:body>{body}</w:body></w:document>").into_bytes()
    }

    /// The families some parts inventory to.
    fn families(parts: &[(&str, &[u8])]) -> Vec<String> {
        let bytes = a_zip(parts);
        let mut zipped = Zipped::of(&bytes).unwrap();
        let mut original = Original::default();
        inventory(&mut zipped, &mut original).unwrap();
        original
            .fonts()
            .map(|font| font.as_str().to_owned())
            .collect()
    }

    /// **A family declared and set on no text is not counted**: a theme and a
    /// style name fonts that a run given its own is never shown in.
    #[test]
    fn a_family_set_on_no_text_is_not_counted() {
        let styles = br#"<w:styles xmlns:w="w"><w:docDefaults><w:rPrDefault><w:rPr><w:rFonts w:asciiTheme="minorHAnsi"/></w:rPr></w:rPrDefault></w:docDefaults></w:styles>"#;
        let theme = br#"<a:theme xmlns:a="a"><a:themeElements><a:fontScheme><a:majorFont><a:latin typeface="Aptos Display"/></a:majorFont><a:minorFont><a:latin typeface="Aptos"/></a:minorFont></a:fontScheme></a:themeElements></a:theme>"#;
        let document = a_document(
            r#"<w:p><w:r><w:rPr><w:rFonts w:ascii="Garamond"/></w:rPr><w:t>Hello</w:t></w:r><w:r><w:rPr><w:rFonts w:ascii="Unseen"/></w:rPr><w:t>  </w:t></w:r></w:p>"#,
        );
        assert_eq!(
            families(&[
                ("word/document.xml", &document),
                ("word/styles.xml", styles),
                ("word/theme/theme1.xml", theme),
            ]),
            ["Garamond"]
        );

        // And a run with no font of its own is in the theme's body font.
        let plain = a_document(r#"<w:p><w:r><w:t>Hello</w:t></w:r></w:p>"#);
        assert_eq!(
            families(&[
                ("word/document.xml", &plain),
                ("word/styles.xml", styles),
                ("word/theme/theme1.xml", theme),
            ]),
            ["Aptos"]
        );
    }

    /// **A style chain is followed**, and one that loops ends rather than
    /// hanging.
    #[test]
    fn a_style_chain_is_followed_and_a_loop_ends() {
        let styles = br#"<w:styles xmlns:w="w">
            <w:style w:type="paragraph" w:default="1" w:styleId="Normal"><w:rPr><w:rFonts w:ascii="Body Face"/></w:rPr></w:style>
            <w:style w:type="paragraph" w:styleId="Quote"><w:basedOn w:val="Fancy"/></w:style>
            <w:style w:type="paragraph" w:styleId="Fancy"><w:rPr><w:rFonts w:ascii="Fancy Face"/></w:rPr></w:style>
            <w:style w:type="character" w:styleId="A"><w:basedOn w:val="B"/></w:style>
            <w:style w:type="character" w:styleId="B"><w:basedOn w:val="A"/></w:style>
        </w:styles>"#;
        let document = a_document(
            r#"<w:p><w:pPr><w:pStyle w:val="Quote"/></w:pPr><w:r><w:t>quoted</w:t></w:r></w:p>
               <w:p><w:r><w:t>plain</w:t></w:r></w:p>
               <w:p><w:r><w:rPr><w:rStyle w:val="A"/></w:rPr><w:t>looping</w:t></w:r></w:p>"#,
        );
        assert_eq!(
            families(&[
                ("word/document.xml", &document),
                ("word/styles.xml", styles)
            ]),
            ["Body Face", "Fancy Face"]
        );
    }

    /// **A field split across runs is still read**, and only fields whose value
    /// depends on the moment or the place are counted.
    #[test]
    fn fields_are_read_however_they_are_split() {
        let document = a_document(
            r#"<w:p>
            <w:r><w:fldChar w:fldCharType="begin"/></w:r><w:r><w:instrText> FILE</w:instrText></w:r><w:r><w:instrText>NAME \p </w:instrText></w:r><w:r><w:fldChar w:fldCharType="separate"/></w:r><w:r><w:t>a.docx</w:t></w:r><w:r><w:fldChar w:fldCharType="end"/></w:r>
            <w:fldSimple w:instr=" MERGEFIELD Name "><w:r><w:t>«Name»</w:t></w:r></w:fldSimple>
            <w:r><w:fldChar w:fldCharType="begin"/></w:r><w:r><w:instrText> PAGE </w:instrText></w:r><w:r><w:fldChar w:fldCharType="end"/></w:r>
            <w:ins><w:r><w:t>added</w:t></w:r></w:ins>
            </w:p>"#,
        );
        let bytes = a_zip(&[("word/document.xml", &document)]);
        let mut zipped = Zipped::of(&bytes).unwrap();
        let mut original = Original::default();
        inventory(&mut zipped, &mut original).unwrap();
        assert_eq!(
            original.fields().iter().copied().collect::<Vec<_>>(),
            [Field::FileName, Field::MergeField]
        );
        assert!(original.tracked_changes());
        assert!(!original.comments());
    }
}
