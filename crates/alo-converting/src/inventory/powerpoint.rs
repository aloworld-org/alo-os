//! A PowerPoint presentation, inventoried: the fonts its slides' text is shown
//! in, its date fields and its comments.
//!
//! # How a run of text finds its font
//!
//! A run's own Latin typeface when it names one; otherwise the slide master's
//! style for the kind of shape it is in — the title style for a title
//! placeholder, the body style for any other placeholder, and the style for
//! other text for a shape that is no placeholder. A typeface of `+mj-lt` or
//! `+mn-lt` is the theme's heading or body font.
//!
//! **Where it stops following**: a layout's or a shape's own list styles, and
//! a slide under a second master. A run those would decide is counted in the
//! first master's style, which can name a family that run does not use; the
//! report says so.

use crate::carried::Field;
use crate::inventory::original::{NotInventoried, Original};
use crate::inventory::theme::{Slot, Theme, shows};
use crate::xml::{self, Read, Walk};
use crate::zip::Zipped;

/// The part a presentation cannot be without.
const THE_PRESENTATION: &str = "ppt/presentation.xml";

/// The first slide master, whose styles text falls back to.
const THE_MASTER: &str = "ppt/slideMasters/slideMaster1.xml";

/// The theme that master is drawn in.
const THE_THEME: &str = "ppt/theme/theme1.xml";

/// What kind of shape text sits in, which decides the master style it falls
/// back to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Shape {
    /// A title placeholder.
    Title,
    /// Any other placeholder.
    Body,
    /// A shape that is no placeholder.
    Other,
}

/// The typefaces a master's three text styles give.
#[derive(Debug, Default)]
struct Master {
    /// For titles.
    title: Option<String>,
    /// For other placeholders.
    body: Option<String>,
    /// For everything else.
    other: Option<String>,
}

/// Inventory a presentation into `original`.
///
/// # Errors
/// [`NotInventoried`].
pub fn inventory(zipped: &mut Zipped<'_>, original: &mut Original) -> Result<(), NotInventoried> {
    if !zipped.holds(THE_PRESENTATION) {
        return Err(NotInventoried::Missing("presentation part"));
    }
    let theme = match zipped.read(THE_THEME)? {
        Some(bytes) => Theme::of(&bytes)?,
        None => Theme::default(),
    };
    let master = match zipped.read(THE_MASTER)? {
        Some(bytes) => master(&xml::read(&bytes)?),
        None => Master::default(),
    };

    let names: Vec<String> = zipped.names().map(str::to_owned).collect();
    let mut slides: Vec<&String> = names
        .iter()
        .filter(|name| {
            name.strip_prefix("ppt/slides/")
                .is_some_and(|file| !file.contains('/') && file.ends_with(".xml"))
        })
        .collect();
    slides.sort();
    for slide in slides {
        if let Some(bytes) = zipped.read(slide)? {
            text(&xml::read(&bytes)?, &master, &theme, original)?;
        }
    }

    for name in &names {
        if name.starts_with("ppt/comments/")
            && name.ends_with(".xml")
            && let Some(bytes) = zipped.read(name)?
            && xml::read(&bytes)?.iter().any(|read| read.opens("cm"))
        {
            original.has_comments();
        }
    }
    Ok(())
}

/// A master, read for the first typeface each of its text styles gives.
fn master(read: &[Read]) -> Master {
    let mut master = Master::default();
    let mut walk = Walk::default();
    for one in read {
        if one.opens("latin") {
            let typeface = one.attribute("typeface").map(str::to_owned);
            let style = if walk.within("titleStyle") {
                Some(&mut master.title)
            } else if walk.within("bodyStyle") {
                Some(&mut master.body)
            } else if walk.within("otherStyle") {
                Some(&mut master.other)
            } else {
                None
            };
            if let Some(style) = style
                && style.is_none()
            {
                *style = typeface;
            }
        }
        walk.past(one);
    }
    master
}

/// A slide, read into `original`.
fn text(
    read: &[Read],
    master: &Master,
    theme: &Theme,
    original: &mut Original,
) -> Result<(), NotInventoried> {
    let mut walk = Walk::default();
    let mut shape = Shape::Other;
    let mut run: Option<(Option<String>, bool)> = None;
    for one in read {
        if one.opens("sp") || one.opens("graphicFrame") {
            shape = Shape::Other;
        } else if one.opens("ph") {
            shape = match one.attribute("type") {
                Some("title" | "ctrTitle") => Shape::Title,
                _ => Shape::Body,
            };
        } else if one.opens("r") || one.opens("fld") {
            if one.opens("fld")
                && one
                    .attribute("type")
                    .is_some_and(|kind| kind.starts_with("datetime"))
            {
                original.has_field(Field::Date);
            }
            run = Some((None, false));
        } else if one.opens("latin") && walk.parent() == Some("rPr") {
            if let Some((typeface, _)) = run.as_mut() {
                *typeface = one.attribute("typeface").map(str::to_owned);
            }
        } else if walk.parent() == Some("t")
            && shows(one)
            && let Some((_, showing)) = run.as_mut()
        {
            *showing = true;
        }
        if (one.closes("r") || one.closes("fld"))
            && let Some((typeface, true)) = run.take()
        {
            let typeface = typeface.or_else(|| match shape {
                Shape::Title => master.title.clone(),
                Shape::Body => master.body.clone(),
                Shape::Other => master.other.clone(),
            });
            let family = match typeface.as_deref() {
                Some("+mj-lt") => theme.family(Slot::Major),
                Some("+mn-lt") => theme.family(Slot::Minor),
                Some(other) if other.starts_with('+') => None,
                other => other,
            };
            if let Some(family) = family {
                original.sets_text_in(family)?;
            }
        }
        walk.past(one);
    }
    Ok(())
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::testing::a_zip;

    /// **A run with no typeface of its own is in the master's style for its
    /// shape**, and a date field is a field.
    #[test]
    fn a_run_without_a_typeface_follows_the_master() {
        let master = br#"<p:sldMaster xmlns:p="p" xmlns:a="a"><p:txStyles>
            <p:titleStyle><a:lvl1pPr><a:defRPr><a:latin typeface="+mj-lt"/></a:defRPr></a:lvl1pPr></p:titleStyle>
            <p:bodyStyle><a:lvl1pPr><a:defRPr><a:latin typeface="+mn-lt"/></a:defRPr></a:lvl1pPr></p:bodyStyle>
            <p:otherStyle><a:lvl1pPr><a:defRPr><a:latin typeface="Other Face"/></a:defRPr></a:lvl1pPr></p:otherStyle>
        </p:txStyles></p:sldMaster>"#;
        let theme = br#"<a:theme xmlns:a="a"><a:majorFont><a:latin typeface="Heading Face"/></a:majorFont><a:minorFont><a:latin typeface="Body Face"/></a:minorFont></a:theme>"#;
        let slide = br#"<p:sld xmlns:p="p" xmlns:a="a"><p:cSld><p:spTree>
            <p:sp><p:nvSpPr><p:nvPr><p:ph type="title"/></p:nvPr></p:nvSpPr><p:txBody><a:p><a:r><a:t>Title</a:t></a:r></a:p></p:txBody></p:sp>
            <p:sp><p:nvSpPr><p:nvPr/></p:nvSpPr><p:txBody><a:p><a:fld type="datetime1"><a:t>9/16/2026</a:t></a:fld></a:p></p:txBody></p:sp>
        </p:spTree></p:cSld></p:sld>"#;
        let bytes = a_zip(&[
            ("ppt/presentation.xml", b"<p:presentation xmlns:p=\"p\"/>"),
            ("ppt/slideMasters/slideMaster1.xml", master),
            ("ppt/theme/theme1.xml", theme),
            ("ppt/slides/slide1.xml", slide),
        ]);
        let mut zipped = Zipped::of(&bytes).unwrap();
        let mut original = Original::default();
        inventory(&mut zipped, &mut original).unwrap();
        let families: Vec<&str> = original.fonts().map(|font| font.as_str()).collect();
        assert_eq!(families, ["Heading Face", "Other Face"]);
        assert!(original.fields().contains(&Field::Date));
    }
}
