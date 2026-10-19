//! **Every picture a desktop frame carries reaches the painter**, read from the
//! source this crate ships because nothing in a test can run that painter.
//!
//! # The fault this closes, measured on a booted machine
//!
//! On 2026-10-18 a person pressing the market's entry on their Dock saw
//! nothing happen. The press arrived, landed on the Dock, refused nothing, and
//! `MarketWindow::open` became true — proven on the machine, because the
//! market then **took the keys**: an Escape went to it instead of closing the
//! overflow list, which is how a window that was open and invisible was
//! caught.
//!
//! `DesktopPicture::market` was laid out by `where_the_market_opens`, rastered
//! by `market_raster::picture`, carried on every frame, and **`desktop_paint`
//! read `dock`, `panel` and `tasks` and never read it.** The window was drawn
//! nowhere. What a person met was a press that did nothing.
//!
//! It is this crate's recurring fault — finished, tested code that nothing
//! calls — and the market had already met it twice: its model landed with no
//! caller, its raster landed with no caller, and `the_market_opens`' own
//! rustdoc says *five surfaces in this crate were found in that state in one
//! day*. The task windows' words were the same: shaped, measured, asserted at
//! twelve, and never reaching a frame.
//!
//! # Why this is read rather than run
//!
//! `DesktopPicture::paint` takes a Smithay `Frame`, which is a renderer's and
//! has no fixture here. Every test that can run asks the **raster** whether a
//! picture was built, and not one of them would fail if the painter stopped
//! drawing it — which is exactly what happened. The question *is this field
//! painted* is answerable by reading and by nothing else available here.
//!
//! # It proves its own reading
//!
//! A name found in a comment is not a name the painter uses, and this file's
//! subject is a comment-heavy one. So the painter's text is stripped of
//! comments before it is searched, and the test asserts it still found the
//! fields it expects to find — a search that cannot fail finds nothing.

#![expect(
    clippy::expect_used,
    reason = "an unexpected None or Err here is the failure this test reports: a file \
              that is not where it says, or a struct it cannot find, both mean the \
              reading is wrong rather than the painter"
)]

use std::path::Path;

/// Where the picture declares what a frame carries.
const THE_PICTURE: &str = "src/desktop_raster.rs";

/// Where it is turned into what a renderer draws.
const THE_PAINTER: &str = "src/desktop_paint.rs";

/// Fields that carry no picture, with the reason each is not painted.
///
/// **Named one by one rather than inferred**, so that adding a field is a
/// decision somebody writes down. A new field is painted or it is listed here
/// with its reason; there is no third answer that leaves it silently undrawn.
const NOT_PICTURES: [(&str, &str); 3] = [
    (
        "size",
        "the extent the picture was laid out for, which the painter checks \
         rather than draws",
    ),
    (
        "top_controls",
        "where the window controls were put, kept for a press to ask about \
         later and drawn as part of the frame that owns them",
    ),
    (
        "surface",
        "the frame's clear colour, which the struct's own rustdoc says is \
         carried rather than drawn: the surface is behind every element, a \
         client's window included, so it is not one more picture composited \
         above them",
    ),
];

/// The crate's own directory, however the test was started from.
fn the_crate() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
}

/// A Rust source with its `//` and `///` lines taken out.
///
/// Block comments are not stripped and do not need to be: neither file uses
/// them, and the assertion below would notice if one started to, because a
/// field named only inside one would then be reported as painted when it is
/// not. Said here rather than left for a reader to wonder about.
fn without_comments(text: &str) -> String {
    text.lines()
        .filter(|line| !line.trim_start().starts_with("//"))
        .collect::<Vec<_>>()
        .join("\n")
}

/// Every `pub(crate)` field named in `DesktopPicture`.
fn what_a_frame_carries(text: &str) -> Vec<String> {
    let from = text
        .find("pub(crate) struct DesktopPicture")
        .expect("desktop_raster declares DesktopPicture");
    let rest = &text[from..];
    let to = rest.find("\n}").expect("the struct ends");
    rest[..to]
        .lines()
        .filter_map(|line| {
            let line = line.trim();
            let named = line.strip_prefix("pub(crate) ")?;
            let (name, _) = named.split_once(':')?;
            Some(name.trim().to_owned())
        })
        .collect()
}

/// **Every field that carries a picture is read by the painter.**
#[test]
fn every_picture_the_desktop_carries_is_painted() {
    let picture = std::fs::read_to_string(the_crate().join(THE_PICTURE))
        .expect("the raster is where it says");
    let painter = std::fs::read_to_string(the_crate().join(THE_PAINTER))
        .expect("the painter is where it says");
    let code = without_comments(&painter);

    let fields = what_a_frame_carries(&picture);
    assert!(
        fields.len() >= 4,
        "read {} fields off DesktopPicture, so this test is not reading the struct it \
         names and would pass whatever the painter did: {fields:?}",
        fields.len()
    );

    // **The search proves it runs.** `dock` has been painted since before this
    // file existed; if it is not found, the stripping or the path is wrong and
    // every other answer below is worthless.
    assert!(
        code.contains("self.dock"),
        "the painter's code does not mention `self.dock`, which it has drawn all along — \
         so this test is searching the wrong text and can only answer yes"
    );

    for field in &fields {
        if let Some((_, why)) = NOT_PICTURES.iter().find(|(named, _)| named == field) {
            assert!(
                !why.is_empty(),
                "`{field}` is excused from being painted with no reason given"
            );
            continue;
        }
        assert!(
            code.contains(&format!("self.{field}")),
            "a desktop frame carries `{field}` and nothing in the painter reads it, so it \
             is laid out and rastered every frame and drawn nowhere — which is what a \
             person meets as a control that does nothing. Paint it, or name it in \
             NOT_PICTURES with the reason it carries no picture."
        );
    }
}
