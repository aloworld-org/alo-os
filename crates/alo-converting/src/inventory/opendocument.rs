//! An OpenDocument, inventoried: the families its text is set in, the fields
//! whose value depends on when it is opened, the content it links rather than
//! carries, its comments and its tracked changes.
//!
//! **One file for all three formats, on purpose.** A text document, a
//! spreadsheet and a presentation written to this standard are the same
//! document with a different body: the same `content.xml` and `styles.xml`, the
//! same styles, the same annotation, the same way of writing down a link. Three
//! files here would be three copies of one reader, and the copy nobody touched
//! would be the one that stopped agreeing with the other two.
//!
//! Where the three genuinely differ, it is one condition rather than one file:
//! only a spreadsheet has a formula, and only a presentation writes its comment
//! in a namespace of its own.
//!
//! # How a run of text finds its family
//!
//! A style names a family in its text properties, and may be based on a parent
//! style that names one instead. An element that shows text names a style: a
//! span, the paragraph around it, the cell around that, the frame around that.
//! So the family is resolved **from the innermost style outwards**, each one
//! followed up its parents, and the first that names a family wins — which is
//! the order the standard applies them in.
//!
//! Where nothing on the way out names one, the default style for the family of
//! the innermost element is used, because that is what the document itself falls
//! back to.
//!
//! **Where it stops following**: a list style, a page layout, and a style that a
//! master page applies to a placeholder nothing was typed into. None of those
//! adds a family to text somebody wrote, so none can report a loss that is not
//! there — which is the same standard `word` and `excel` hold themselves to.
//!
//! # Where an OpenDocument says what it links
//!
//! Not in a part beside every part, the way the three Office formats do
//! (`crate::inventory::linked`), but **in the element that shows the content**,
//! as an `href`. So this reader gathers links as it walks, and
//! `crate::inventory::original` does not run the relationships reader for these
//! three — one that found no `.rels` part would report *nothing was linked*
//! about a document full of links.
//!
//! A link counts when the package does not hold what it names. An embedded
//! picture is written as `Pictures/…` and is in the zip; a linked one is an
//! address of somewhere else, and ADR 0039's service has no network and no
//! files, so it is not fetched and is reported as not carried.
//!
//! **A hyperlink is not linked content**, here as everywhere: nothing is fetched
//! to show one.

use crate::carried::{Field, Linked};
use crate::inventory::formula::field_of;
use crate::inventory::original::{NotInventoried, Original};
use crate::inventory::theme::shows;
use crate::xml::{self, Read, Walk};
use crate::zip::Zipped;

/// The part an OpenDocument cannot be without.
const THE_CONTENT: &str = "content.xml";

/// The part its named styles are in, which every OpenDocument has and which is
/// read when it is there.
const THE_STYLES: &str = "styles.xml";

/// What a document's own description of itself is in.
const THE_META: &str = "meta.xml";

/// The deepest chain of parent styles followed.
const DEEPEST_STYLES: usize = 32;

/// The most columns of one table whose default cell style is kept.
///
/// A sheet writes its trailing empty columns as one element repeated sixteen
/// thousand times, and a document handed to this service was written by
/// somebody else, who may have written a larger number still. A column past
/// this one holds no text anybody typed, so what is lost by stopping here is
/// nothing, and what is gained is that a number in a stranger's file cannot ask
/// for an allocation.
const MOST_COLUMNS: usize = 1024;

/// One style: the family it names, and the style it is based on.
#[derive(Debug, Default, Clone)]
struct Style {
    /// The family its text properties name.
    font: Option<String>,
    /// The style it is based on.
    parent: Option<String>,
}

/// Every style a document declares, by name, and the default for each family.
#[derive(Debug, Default)]
struct Styles {
    /// By style name.
    by_name: std::collections::HashMap<String, Style>,
    /// By style family: `paragraph`, `text`, `table-cell`, `graphic`.
    defaults: std::collections::HashMap<String, Style>,
}

impl Styles {
    /// The family a style names, or the nearest one its parents name.
    fn font_of(&self, named: &str) -> Option<&str> {
        let mut at = named;
        for _ in 0..DEEPEST_STYLES {
            let style = self.by_name.get(at)?;
            if let Some(font) = &style.font {
                return Some(font);
            }
            at = style.parent.as_deref()?;
        }
        None
    }

    /// The family the default style for this family of styles names.
    fn default_font_of(&self, family: &str) -> Option<&str> {
        self.defaults.get(family)?.font.as_deref()
    }

    /// Read the styles one part declares, adding to what is already known.
    ///
    /// Both `styles.xml` and `content.xml` declare styles — the named ones and
    /// the automatic ones a document makes as it is edited — and a name is
    /// unique across the document, so one map holds both.
    fn read(&mut self, read: &[Read]) {
        let mut walk = Walk::default();
        let mut open: Option<(String, Style)> = None;
        let mut default: Option<(String, Style)> = None;
        for one in read {
            if one.opens("style") {
                open = one.attribute("name").map(|name| {
                    (
                        name.to_owned(),
                        Style {
                            font: None,
                            parent: one.attribute("parent-style-name").map(str::to_owned),
                        },
                    )
                });
            } else if one.opens("default-style") {
                default = one
                    .attribute("family")
                    .map(|family| (family.to_owned(), Style::default()));
            } else if one.opens("text-properties")
                && let Some(font) = one.attribute("font-name")
            {
                if let Some((_, style)) = open.as_mut() {
                    style.font = Some(font.to_owned());
                } else if let Some((_, style)) = default.as_mut() {
                    style.font = Some(font.to_owned());
                }
            }
            if one.closes("style")
                && let Some((name, style)) = open.take()
            {
                self.by_name.insert(name, style);
            }
            if one.closes("default-style")
                && let Some((family, style)) = default.take()
            {
                self.defaults.insert(family, style);
            }
            walk.past(one);
        }
    }
}

/// Inventory an OpenDocument into `original`.
///
/// # Errors
/// [`NotInventoried`].
pub fn inventory(zipped: &mut Zipped<'_>, original: &mut Original) -> Result<(), NotInventoried> {
    let Some(content) = zipped.read(THE_CONTENT)? else {
        return Err(NotInventoried::Missing("content part"));
    };
    let content = xml::read(&content)?;

    let mut styles = Styles::default();
    if let Some(bytes) = zipped.read(THE_STYLES)? {
        styles.read(&xml::read(&bytes)?);
    }
    styles.read(&content);

    // The names in the package, so a link can be told from something carried.
    let held: Vec<String> = zipped.names().map(str::to_owned).collect();

    body(&content, &styles, &held, original)?;

    if let Some(bytes) = zipped.read(THE_META)?
        && xml::read(&bytes)?
            .iter()
            .any(|read| read.opens("template") && read.attribute("href").is_some())
    {
        original.has_linked(Linked::Template);
    }
    Ok(())
}

/// Where in a sheet's columns a walk is, and what each column says a cell in it
/// is styled by when the cell says nothing.
///
/// **A spreadsheet's text finds its family differently from a paragraph's**, and
/// this is the difference. A cell that nobody has styled individually carries no
/// style at all; the family comes from the column it is in, which said
/// `table:default-cell-style-name` once, before any row was written — so it is a
/// sibling of the rows rather than an element around the cell, and nothing about
/// what is open around a cell can find it.
///
/// Measured against `tests/documents/sample.ods`, where the application that
/// wrote it put Garamond on the column and left every cell unstyled. An inventory that looked only at
/// what was open around the text read that sheet as setting text in no family at
/// all, and would have reported that a conversion substituting every one of its
/// fonts had lost nothing.
///
/// **Where it stops following**: a row's own `table:default-cell-style-name`,
/// which takes precedence over a column's for a cell in both, and a cell covered
/// by another. Neither can turn a family that is there into one that is not; the
/// most either costs is a family named twice by two routes, and a family is kept
/// once.
#[derive(Debug, Default)]
struct Columns {
    /// Each column's default cell style, in order.
    default_cell_style: Vec<Option<String>>,
    /// Which column the next cell is in.
    at: usize,
}

impl Columns {
    /// The style the column the walk is in gives a cell that names none.
    fn here(&self) -> Option<String> {
        self.default_cell_style.get(self.at)?.clone()
    }

    /// Move past one element that opened.
    fn past(&mut self, one: &Read, name: &str) {
        let repeated = |attribute: &str| {
            one.attribute(attribute)
                .and_then(|how_many| how_many.parse::<usize>().ok())
                .unwrap_or(1)
                .max(1)
        };
        match name {
            // A new table: its columns are not the last table's.
            "table" => *self = Self::default(),
            "table-column" => {
                let style = one.attribute("default-cell-style-name").map(str::to_owned);
                let how_many = repeated("number-columns-repeated")
                    .min(MOST_COLUMNS.saturating_sub(self.default_cell_style.len()));
                self.default_cell_style
                    .extend(std::iter::repeat_n(style, how_many));
            }
            // Every row begins at the first column again.
            "table-row" => self.at = 0,
            "table-cell" | "covered-table-cell" => {
                self.at = self.at.saturating_add(repeated("number-columns-repeated"));
            }
            _ => {}
        }
    }
}

/// Which family of styles an element that names a style belongs to, so that the
/// right default can stand in where no style on the way out names a family.
fn family_of(element: &str) -> &'static str {
    match element {
        "table-cell" => "table-cell",
        "span" => "text",
        "frame" | "custom-shape" | "text-box" => "graphic",
        _ => "paragraph",
    }
}

/// The style an element names, if it names one.
///
/// A span and a paragraph say `text:style-name`, a cell says
/// `table:style-name`, a frame says `draw:text-style-name` for the text inside
/// it — and every one of those is `style-name` or `text-style-name` once the
/// prefix is gone.
fn style_named(one: &Read) -> Option<&str> {
    one.attribute("style-name")
        .or_else(|| one.attribute("text-style-name"))
}

/// The body of a document, read for everything an inventory holds.
fn body(
    read: &[Read],
    styles: &Styles,
    held: &[String],
    original: &mut Original,
) -> Result<(), NotInventoried> {
    let mut walk = Walk::default();
    // One entry for every element open around what is being read, innermost
    // last, holding the style it named and the family of styles it belongs to —
    // or nothing, for an element that named no style. An entry for every open
    // element rather than only for the ones that named one, because that is
    // what makes a close a pop and keeps the two in step without counting.
    let mut around: Vec<Option<(String, &'static str)>> = Vec::new();
    let mut columns = Columns::default();
    for one in read {
        match one {
            Read::Opened { name, empty, .. } => {
                // Asked before the walk moves past this element, because a cell
                // that names no style is styled by the column it is in and
                // moving past it is what leaves that column.
                if !*empty {
                    let named = style_named(one)
                        .map(str::to_owned)
                        .or_else(|| (name == "table-cell").then(|| columns.here()).flatten());
                    around.push(named.map(|named| (named, family_of(name))));
                }
                columns.past(one, name);
                noted(one, name, held, &walk, original);
            }
            Read::Closed { .. } => {
                around.pop();
            }
            Read::Text(_) => {
                if shows(one)
                    && !walk.within("annotation")
                    && let Some(family) = family_around(styles, &around)
                {
                    original.sets_text_in(family)?;
                }
            }
        }
        walk.past(one);
    }
    Ok(())
}

/// The family the innermost style that names one gives, or the default for the
/// family of styles the innermost element that named a style belongs to.
fn family_around<'a>(
    styles: &'a Styles,
    around: &[Option<(String, &'static str)>],
) -> Option<&'a str> {
    let named: Vec<&(String, &'static str)> = around.iter().flatten().collect();
    for (name, _) in named.iter().rev() {
        if let Some(font) = styles.font_of(name) {
            return Some(font);
        }
    }
    let family = named.last().map_or("paragraph", |(_, family)| family);
    styles.default_font_of(family)
}

/// What one element says, other than about a family.
fn noted(one: &Read, name: &str, held: &[String], walk: &Walk, original: &mut Original) {
    // A comment. Writer and Calc write `office:annotation`, Impress writes
    // `officeooo:annotation` in a namespace of its own — and they are one local
    // name, which is why ADR 0039 §5's reader hands on local names and a reader
    // that kept prefixes would read a presentation as having no comments.
    if name == "annotation" {
        original.has_comments();
    }
    // A change somebody made with tracking on. The element that holds them is
    // written whether or not there are any; a region inside it is a change.
    if name == "changed-region" {
        original.has_tracked_changes();
    }
    // A field whose value is not the same tomorrow — and never one inside a
    // comment, because a comment carries the day it was written as its own
    // `date`, under the same local name as the field, and that day does not
    // change when the document is opened.
    if !walk.within("annotation")
        && let Some(field) = field_named(name, one)
    {
        original.has_field(field);
    }
    // A formula calling for the moment or for chance.
    if let Some(formula) = one.attribute("formula")
        && let Some(field) = field_of(formula)
    {
        original.has_field(field);
    }
    // Content shown from somewhere the package does not hold. A reference
    // beginning with `#` is a place in this document, and nothing is fetched.
    if let Some(kind) = links(name)
        && let Some(href) = one.attribute("href")
        && !href.starts_with('#')
        && !held.iter().any(|held| held == href)
    {
        original.has_linked(kind);
    }
}

/// The field an element is, where it is one whose value depends on when or
/// where the document is open.
///
/// A date or a time marked `fixed` was written down once and does not change,
/// so a copy carries it exactly and it is not a loss.
fn field_named(element: &str, one: &Read) -> Option<Field> {
    let fixed = one.attribute("fixed") == Some("true");
    match element {
        "date" if !fixed => Some(Field::Date),
        "time" if !fixed => Some(Field::Time),
        "file-name" => Some(Field::FileName),
        // The person who wrote it and the person who has it open now are two
        // questions, and both are answered at the moment it is opened.
        "author-name" | "author-initials" | "initial-creator" => Some(Field::Author),
        "database-display" => Some(Field::MergeField),
        _ => None,
    }
}

/// What an element that carries an `href` shows, or [`None`] where an `href` on
/// it is not content taken from elsewhere.
fn links(element: &str) -> Option<Linked> {
    match element {
        "image" | "background-image" => Some(Linked::Picture),
        "table-source" => Some(Linked::Data),
        "object" | "object-ole" => Some(Linked::SomethingElse),
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

    /// A document of these parts, as bytes an inventory can be run over.
    fn a_document(parts: &[(&str, &[u8])]) -> Vec<u8> {
        a_zip(parts)
    }

    /// The families some inventory found.
    fn families(original: &Original) -> Vec<String> {
        original
            .fonts()
            .map(|font| font.as_str().to_owned())
            .collect()
    }

    /// **A span's family beats the paragraph's**, and a paragraph's beats the
    /// default — which is the order the standard applies them in.
    #[test]
    fn the_innermost_style_that_names_a_family_wins() {
        let styles = br#"<document-styles><styles>
            <default-style family="paragraph"><text-properties font-name="Fallback"/></default-style>
            <style name="Standard" family="paragraph"><text-properties font-name="Garamond"/></style>
            <style name="Quiet" family="text"><text-properties font-name="Whisper"/></style>
            <style name="Plain" family="paragraph"/>
        </styles></document-styles>"#;
        let content = br#"<document-content><body><text>
            <p style-name="Standard">outer<span style-name="Quiet">inner</span></p>
            <p style-name="Plain">nothing names one</p>
        </text></body></document-content>"#;
        let bytes = a_document(&[("content.xml", content), ("styles.xml", styles)]);
        let mut zipped = Zipped::of(&bytes).unwrap();
        let mut original = Original::default();
        inventory(&mut zipped, &mut original).unwrap();
        let mut found = families(&original);
        found.sort();
        assert_eq!(found, ["Fallback", "Garamond", "Whisper"]);
    }

    /// **A style with no family of its own takes its parent's.**
    #[test]
    fn a_style_follows_its_parent_for_a_family() {
        let styles = br#"<document-styles><styles>
            <style name="Base" family="paragraph"><text-properties font-name="Garamond"/></style>
            <style name="Derived" family="paragraph" parent-style-name="Base"/>
        </styles></document-styles>"#;
        let content =
            br#"<document-content><body><text><p style-name="Derived">words</p></text></body></document-content>"#;
        let bytes = a_document(&[("content.xml", content), ("styles.xml", styles)]);
        let mut zipped = Zipped::of(&bytes).unwrap();
        let mut original = Original::default();
        inventory(&mut zipped, &mut original).unwrap();
        assert_eq!(families(&original), ["Garamond"]);
    }

    /// **A picture the package holds is carried; one it does not is linked.**
    ///
    /// The distinction is the whole of it: an embedded picture is in the zip
    /// and nothing is fetched to show it, and a linked one is an address the
    /// service cannot reach and must therefore report.
    #[test]
    fn a_picture_in_the_package_is_not_a_link() {
        let content = br#"<document-content><body><text>
            <p><frame><image href="Pictures/carried.png"/></frame></p>
            <p><frame><image href="../../elsewhere/absent.png"/></frame></p>
            <p><a href="https://example.invalid/">a hyperlink is not content</a></p>
        </text></body></document-content>"#;
        let bytes = a_document(&[
            ("content.xml", content),
            ("Pictures/carried.png", b"not really a picture"),
        ]);
        let mut zipped = Zipped::of(&bytes).unwrap();
        let mut original = Original::default();
        inventory(&mut zipped, &mut original).unwrap();
        assert_eq!(
            original.linked(),
            &std::collections::BTreeSet::from([Linked::Picture])
        );
    }

    /// **A document with no content part is not inventoried**, rather than
    /// inventoried as holding nothing.
    #[test]
    fn a_document_without_its_content_is_not_inventoried() {
        let bytes = a_document(&[("styles.xml", b"<document-styles/>")]);
        let mut zipped = Zipped::of(&bytes).unwrap();
        let mut original = Original::default();
        assert!(matches!(
            inventory(&mut zipped, &mut original),
            Err(NotInventoried::Missing(_))
        ));
    }
}
