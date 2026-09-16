//! An Excel workbook, inventoried: the fonts its cells are shown in, the
//! formulas whose value depends on the moment, its comments and its tracked
//! changes.
//!
//! # How a cell finds its font
//!
//! A cell names a cell format by number (`s`, or the first when it names none);
//! the format names a font by number; the font names its family. **Only a cell
//! with a value is counted**, because a format given to an empty cell shows no
//! text — the owner's workbook has one of exactly those. A run of rich text in
//! a shared string that names a family of its own is counted too.
//!
//! **Where it stops following**: conditional formats, and a shared string's
//! runs are counted whether or not a cell uses that string.

use crate::carried::Field;
use crate::inventory::original::{NotInventoried, Original};
use crate::inventory::theme::shows;
use crate::xml::{self, Read, Walk};
use crate::zip::Zipped;

/// The part an Excel workbook cannot be without.
const THE_WORKBOOK: &str = "xl/workbook.xml";

/// The functions whose value is the moment the workbook is calculated.
const THE_CURRENT_MOMENT: [&str; 2] = ["NOW", "TODAY"];

/// The functions whose value is a random number.
const A_RANDOM_NUMBER: [&str; 3] = ["RAND", "RANDBETWEEN", "RANDARRAY"];

/// A workbook's fonts and cell formats.
#[derive(Debug, Default)]
struct Formats {
    /// Every font's family, in order, [`None`] for one that names none.
    fonts: Vec<Option<String>>,
    /// Every cell format's font, by number.
    cells: Vec<Option<usize>>,
}

/// Inventory an Excel workbook into `original`.
///
/// # Errors
/// [`NotInventoried`].
pub fn inventory(zipped: &mut Zipped<'_>, original: &mut Original) -> Result<(), NotInventoried> {
    if !zipped.holds(THE_WORKBOOK) {
        return Err(NotInventoried::Missing("workbook part"));
    }
    let formats = match zipped.read("xl/styles.xml")? {
        Some(bytes) => formats(&xml::read(&bytes)?),
        None => Formats::default(),
    };
    if let Some(bytes) = zipped.read("xl/sharedStrings.xml")? {
        rich_text(&xml::read(&bytes)?, original)?;
    }

    let names: Vec<String> = zipped.names().map(str::to_owned).collect();
    let mut sheets: Vec<&String> = names
        .iter()
        .filter(|name| {
            name.starts_with("xl/worksheets/") && is_one_xml_file(name, "xl/worksheets/")
        })
        .collect();
    sheets.sort();
    for sheet in sheets {
        if let Some(bytes) = zipped.read(sheet)? {
            cells(&xml::read(&bytes)?, &formats, original)?;
        }
    }

    for name in &names {
        if name.starts_with("xl/revisions/") {
            original.has_tracked_changes();
        }
        let comments = (name.starts_with("xl/comments") && name.ends_with(".xml"))
            .then_some("comment")
            .or(
                (name.starts_with("xl/threadedComments/") && name.ends_with(".xml"))
                    .then_some("threadedComment"),
            );
        if let Some(element) = comments
            && let Some(bytes) = zipped.read(name)?
            && xml::read(&bytes)?.iter().any(|read| read.opens(element))
        {
            original.has_comments();
        }
    }
    Ok(())
}

/// Whether a name is one XML file directly inside a folder.
fn is_one_xml_file(name: &str, folder: &str) -> bool {
    name.strip_prefix(folder)
        .is_some_and(|file| !file.contains('/') && file.ends_with(".xml"))
}

/// A styles part, read for its fonts and cell formats.
fn formats(read: &[Read]) -> Formats {
    let mut formats = Formats::default();
    let mut walk = Walk::default();
    for one in read {
        match walk.parent() {
            Some("fonts") if one.opens("font") => formats.fonts.push(None),
            Some("font") if one.opens("name") && walk.grandparent() == Some("fonts") => {
                if let Some(font) = formats.fonts.last_mut() {
                    *font = one.attribute("val").map(str::to_owned);
                }
            }
            Some("cellXfs") if one.opens("xf") => formats.cells.push(
                one.attribute("fontId")
                    .and_then(|id| id.parse::<usize>().ok()),
            ),
            _ => {}
        }
        walk.past(one);
    }
    formats
}

/// A shared strings part, read for runs that name a family of their own.
fn rich_text(read: &[Read], original: &mut Original) -> Result<(), NotInventoried> {
    let mut walk = Walk::default();
    let mut run: Option<(Option<String>, bool)> = None;
    for one in read {
        if one.opens("r") && walk.parent() == Some("si") {
            run = Some((None, false));
        } else if one.opens("rFont") && walk.parent() == Some("rPr") {
            if let Some((family, _)) = run.as_mut() {
                *family = one.attribute("val").map(str::to_owned);
            }
        } else if walk.parent() == Some("t")
            && shows(one)
            && let Some((_, showing)) = run.as_mut()
        {
            *showing = true;
        }
        if one.closes("r")
            && let Some((Some(family), true)) = run.take()
        {
            original.sets_text_in(&family)?;
        }
        walk.past(one);
    }
    Ok(())
}

/// A worksheet, read for the fonts of cells with values and the fields of
/// cells with formulas.
fn cells(read: &[Read], formats: &Formats, original: &mut Original) -> Result<(), NotInventoried> {
    let mut walk = Walk::default();
    let mut cell: Option<(usize, bool)> = None;
    for one in read {
        if one.opens("c") {
            let format = one
                .attribute("s")
                .and_then(|format| format.parse::<usize>().ok())
                .unwrap_or(0);
            cell = Some((format, false));
        } else if let Read::Text(formula) = one
            && walk.parent() == Some("f")
            && let Some(field) = field_of(formula)
        {
            original.has_field(field);
        } else if shows(one)
            && (walk.parent() == Some("v") || walk.within("is"))
            && let Some((_, showing)) = cell.as_mut()
        {
            *showing = true;
        }
        if one.closes("c")
            && let Some((format, true)) = cell.take()
            && let Some(Some(font)) = formats.cells.get(format)
            && let Some(Some(family)) = formats.fonts.get(*font)
        {
            original.sets_text_in(family)?;
        }
        walk.past(one);
    }
    Ok(())
}

/// What a formula is, when it calls for the moment or for chance.
///
/// A function counts where its name stands on its own before a bracket, so
/// `NOW()` is the moment and `KNOWN()` and a sheet called `NOW` are not.
fn field_of(formula: &str) -> Option<Field> {
    let formula = formula.to_ascii_uppercase();
    let calls = |function: &str| {
        formula.match_indices(function).any(|(at, _)| {
            let before = formula.get(..at).and_then(|before| before.chars().last());
            let after = formula.get(at + function.len()..);
            before.is_none_or(|letter| !(letter.is_ascii_alphanumeric() || letter == '_'))
                && after.is_some_and(|after| after.trim_start().starts_with('('))
        })
    };
    if THE_CURRENT_MOMENT.iter().any(|function| calls(function)) {
        return Some(Field::TheCurrentMoment);
    }
    A_RANDOM_NUMBER
        .iter()
        .any(|function| calls(function))
        .then_some(Field::ARandomNumber)
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::testing::a_zip;

    /// **The moment and chance are found by function**, and what only looks
    /// like one is not.
    #[test]
    fn the_moment_and_chance_are_found_by_function() {
        assert_eq!(field_of("NOW()"), Some(Field::TheCurrentMoment));
        assert_eq!(field_of("A1+today ()"), Some(Field::TheCurrentMoment));
        assert_eq!(field_of("_xlfn.RANDARRAY(3)"), Some(Field::ARandomNumber));
        assert_eq!(field_of("RANDBETWEEN(1,6)"), Some(Field::ARandomNumber));
        assert_eq!(field_of("A2*2"), None);
        assert_eq!(field_of("KNOWN(A1)"), None);
        assert_eq!(field_of("NOW!A1"), None);
        assert_eq!(field_of("SUM(NOWHERE)"), None);
    }

    /// **A format given to a cell with no value shows no text**, so its font
    /// is not counted.
    #[test]
    fn a_format_on_an_empty_cell_is_not_counted() {
        let styles = br#"<styleSheet><fonts count="3"><font><name val="Aptos Narrow"/></font><font><name val="Garamond"/></font><font><name val="Never Shown"/></font></fonts>
            <cellXfs count="3"><xf fontId="0"/><xf fontId="1"/><xf fontId="2"/></cellXfs></styleSheet>"#;
        let sheet = br#"<worksheet><sheetData><row r="1"><c r="A1" s="1"><v>1</v></c><c r="B1" s="2"/><c r="C1" s="2"><v></v></c></row></sheetData></worksheet>"#;
        let bytes = a_zip(&[
            ("xl/workbook.xml", b"<workbook/>"),
            ("xl/styles.xml", styles),
            ("xl/worksheets/sheet1.xml", sheet),
        ]);
        let mut zipped = Zipped::of(&bytes).unwrap();
        let mut original = Original::default();
        inventory(&mut zipped, &mut original).unwrap();
        let families: Vec<&str> = original.fonts().map(|font| font.as_str()).collect();
        assert_eq!(families, ["Garamond"]);
        assert!(!original.comments());
    }
}
