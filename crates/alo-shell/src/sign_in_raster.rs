//! The sign-in screen laid out and rasterised: solid shapes and inked text,
//! owned, for one output size.
//!
//! # What is drawn
//!
//! The ground, and then either one sentence alone — *make an account*, or
//! accounts that would not be read — or two fields with the last sentence the
//! greeting answered beneath them. The name field shows the name as typed,
//! scrolled so its end stays visible. The password field shows **whether**
//! something is typed, as one band of the same width however much is typed,
//! and never a mark per letter: a count of dots is the password's length on a
//! screen anybody behind the person can read.
//!
//! The waiting field is told apart by a thicker edge **and** a caret, never by
//! colour alone, and neither is deep teal, which means the agent and is not
//! on a screen nobody has signed in to yet.
//!
//! # What it reads
//!
//! `crate::SignInShows` and nothing else about the screen, so this file has no
//! way to reach the password even by mistake. Colours are `alo-appearance`'s
//! tokens and the scale is the person's `TextScale` times the screen's own
//! (`crate::the_sign_in_screens_scale`); the font is the bundled face
//! `crate::WindowControlLabels` already loads, never a host font.
//!
//! # The screen's own scale
//!
//! The design's numbers are logical, so on a screen drawn at 200 per cent the
//! screen is **laid out at half its pixels and drawn at twice the size**: the
//! layout, the rounded shapes, every measure and every line of type. At 100
//! each conversion is the identity, so the picture is the one it was before.
//! The first-start sheets follow it the same way (`crate::setup_sheet_in_pixels`),
//! held to the largest scale at which a sheet still fits
//! (`crate::a_scale_held_to_what_draws`), and so does the lock screen's
//! embedded form, whose every figure already went through [`Measure`].

use alo_appearance::{Scheme, TextScale};
use cosmic_text::{Buffer, FontSystem, Metrics, Shaping, Wrap};
use smithay::utils::{Logical, Physical, Rectangle, Scale, Size};

use crate::display_scale::DisplayScale;

use crate::painted::{Inked, Solid};
use crate::painted_text::{Shaped, inked, sentence};
use crate::{Contrast, RenderError, SignInField, SignInShows, WindowControlLabels};

/// How the screen looks: the scheme and the text scale the person chose.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SignInLook {
    /// Light or dark.
    pub scheme: Scheme,
    /// The person's text scale, applied to every measure on the screen.
    pub scale: TextScale,
    /// The design's palette, or the one high contrast decides
    /// (`crate::access_contrast`).
    pub contrast: Contrast,
}

/// The largest output side, in pixels, the screen is laid out for.
const LARGEST_SIDE: i32 = 16_384;

/// The whole screen for one output size, ready to paint.
///
/// Holds pixels of the name as typed and of sentences, and nothing about the
/// password but whether one is typed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SignInPicture {
    /// The output size it was laid out for.
    pub(crate) size: (i32, i32),
    /// Flat shapes, painted first, in order.
    pub(crate) solids: Vec<Solid>,
    /// Text, painted after the shapes, in order.
    pub(crate) inked: Vec<Inked>,
    /// The rounded shapes, as a raster, painted **under** the flats and the
    /// text.
    ///
    /// [`None`] where the screen has no area, and for the sentence a machine
    /// with no accounts shows - that screen is the owner's separate task and
    /// keeps the flat ground until it is built.
    pub(crate) surface: Option<crate::painted::Washed>,
    /// **What a press on this picture can land on**, where it was drawn: a
    /// first-start sheet's rows and actions (`crate::a_press_on_a_setup_sheet`).
    ///
    /// Empty for the sign-in screen, whose presses are answered by
    /// `crate::sign_in_layout`, and for a sentence, which has nothing to press.
    pub(crate) controls: Vec<crate::a_press_on_a_setup_sheet::SheetControl>,
}

/// The colours of one scheme.
#[derive(Clone, Copy)]
struct Palette {
    /// The screen's ground.
    ground: [u8; 3],
    /// Inside a field.
    ///
    /// **Unread since the sign-in screen was redrawn to its design on
    /// 2026-10-12**: the entry's inside is now drawn by
    /// `crate::sign_in_surface`, which takes `bg/surface` from the design's
    /// own token rather than from a contrast-derived palette. It is kept
    /// because the sentence screen a machine with no accounts shows still
    /// uses this palette, and that screen is the owner's separate task.
    field: [u8; 3],
    /// Text, edges and the caret.
    ink: [u8; 3],
}

impl Palette {
    /// The screen's three colours, from the one palette door. The ground is
    /// the one the dock is drawn on and a field's inside is a window's ground,
    /// which is how the two read as a field on a screen rather than two panels.
    fn of(scheme: Scheme, contrast: Contrast) -> Self {
        Self {
            ground: contrast.dock(scheme),
            field: contrast.ground(scheme),
            ink: contrast.ink(scheme),
        }
    }
}

/// Whether this picture is the whole screen or sits inside another.
///
/// **The sign-in screen and the lock screen share this raster**, and they are
/// not the same shape. `crate::direct_sign_in` and `crate::nested_sign_in`
/// hand over a whole output; `crate::lock_raster` hands over the middle third
/// of one, with lock facts above and below it.
///
/// The design the owner gave on 2026-10-12 — `176-10686.xml` — is a full-screen
/// composition: a wallpaper, a centred group and essential actions at the
/// edges. It has nowhere to put the lock screen's facts, so applying it to an
/// embedded form would redesign a surface nobody asked about. **A caller says
/// which it is, and the two roads stay apart until somebody rules on the lock
/// screen's own composition.**
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum WhoseScreen {
    /// The whole output, drawn to the design.
    ItsOwn,
    /// A region inside another screen, drawn as it was before the design
    /// arrived.
    InsideAnother,
}

/// Lay the screen out for an output of `size` drawn at `display`, and
/// rasterise it.
///
/// `display` is the screen's own scale (`crate::the_sign_in_screens_scale`):
/// every measure goes through it and the person's text size together. At
/// [`DisplayScale::ONE`] nothing changes.
///
/// # Errors
/// [`RenderError::SignInScene`] when the output is too small to hold two
/// fields, or larger than any output this screen is laid out for.
pub(crate) fn picture(
    shows: SignInShows<'_>,
    labels: &mut WindowControlLabels,
    words: &alo_strings::Strings,
    size: (i32, i32),
    look: SignInLook,
    whose: WhoseScreen,
    display: DisplayScale,
) -> Result<SignInPicture, RenderError> {
    let measure = Measure::at(look.scale, display);
    let (width, height) = size;
    if width > LARGEST_SIDE || height > LARGEST_SIDE {
        return Err(RenderError::SignInScene);
    }
    let column = (width - 2 * measure.px(16)).min(measure.px(440));
    let field_height = measure.px(48);
    let gap = measure.px(12);
    if column < measure.px(120) || height < 2 * field_height + gap + 2 * measure.px(16) {
        return Err(RenderError::SignInScene);
    }
    let palette = Palette::of(look.scheme, look.contrast);
    let left = (width - column) / 2;
    let mut drawn = SignInPicture {
        size,
        surface: None,
        solids: vec![Solid {
            area: Rectangle::from_size((width, height).into()),
            colour: palette.ground,
        }],
        inked: Vec::new(),
        controls: Vec::new(),
    };
    let fonts = &mut labels.fonts;

    match shows {
        // **The first-start sheet**, for a machine that knows nobody.
        //
        // A different composition from the sign-in screen, deliberately: read
        // top to bottom in a column at the left, where the sign-in screen is
        // answered in one place at the middle. `crate::setup_sheet_layout`
        // places it and `crate::first_start_sheet` fills it.
        SignInShows::FirstAccount {
            name,
            password_typed,
            field,
            said,
        } => {
            // **At the screen's own scale, or the largest that still holds the
            // sheet** — see `crate::a_scale_held_to_what_draws`.
            let sheet = crate::first_start_sheet::FirstStartSheet {
                on_the_pin: matches!(field, SignInField::Password) || password_typed,
                typed: name,
                password_typed,
                said,
            };
            let colours = crate::sign_in_surface::TheColours::of(look.contrast, look.scheme);
            return crate::a_scale_held_to_what_draws::at_the_largest_that_draws(display, |at| {
                crate::first_start_sheet::drawn(
                    &sheet,
                    fonts,
                    words,
                    (width, height),
                    colours,
                    &Measure::at(look.scale, at),
                )
            });
        }
        // **The first-start sheets**, which a fresh machine shows before it
        // asks for a name — so that a person has their own language, region
        // and keyboard before they type a password through it.
        SignInShows::FirstStart(first_start) => {
            let colours = crate::sign_in_surface::TheColours::of(look.contrast, look.scheme);
            // **At the screen's own scale, or the largest that still holds the
            // sheet** — see `crate::a_scale_held_to_what_draws`.
            return crate::a_scale_held_to_what_draws::at_the_largest_that_draws(display, |at| {
                let sheet_measure = Measure::at(look.scale, at);
                match first_start.sheet() {
                    crate::the_first_start::Sheet::Overview => crate::overview_sheet::drawn(
                        first_start,
                        fonts,
                        words,
                        (width, height),
                        colours,
                        &sheet_measure,
                    ),
                    crate::the_first_start::Sheet::Language(_) => crate::language_sheet::drawn(
                        &crate::language_sheet::LanguageSheet {
                            offered: first_start.languages(),
                        },
                        fonts,
                        words,
                        (width, height),
                        colours,
                        &sheet_measure,
                    ),
                    crate::the_first_start::Sheet::Region(offered) => crate::region_sheet::drawn(
                        offered,
                        fonts,
                        words,
                        (width, height),
                        colours,
                        &sheet_measure,
                    ),
                    crate::the_first_start::Sheet::Zone(picker) => crate::zone_sheet::drawn(
                        picker,
                        fonts,
                        words,
                        (width, height),
                        colours,
                        &sheet_measure,
                    ),
                    crate::the_first_start::Sheet::Keyboard(picker) => {
                        crate::keyboard_sheet::drawn(
                            first_start,
                            picker,
                            fonts,
                            words,
                            (width, height),
                            colours,
                            &sheet_measure,
                        )
                    }
                }
            });
        }
        SignInShows::Sentence(said) => {
            let text = sentence(
                fonts,
                said.text(),
                column,
                measure.metrics(),
                palette.ground,
                palette.ink,
            );
            let top = ((height - text.height) / 2).max(measure.px(16));
            drawn.inked.extend(text.placed(left, top, height));
        }
        SignInShows::Fields {
            whose: who,
            name,
            password_typed,
            field,
            said,
        } => match whose {
            // **The design the owner gave on 2026-10-12.** A full-screen
            // composition: one entry with its submit arrow inside it, the
            // greeting above, the essential actions at the edges.
            WhoseScreen::ItsOwn => {
                let _ = field;

                // **The palette the person chose**, not seven light-mode
                // constants. `crate::sign_in_surface::TheColours` says what
                // each is and why four of them are mixed.
                let colours = crate::sign_in_surface::TheColours::of(look.contrast, look.scheme);

                // **The design's own layout**, in logical pixels: the output's
                // pixels through the screen's own scale, so a 200 per cent
                // screen is laid out at half its pixels and drawn at twice the
                // size. `crate::sign_in_layout` is the only thing that decides
                // where anything goes, so a test can hold it against
                // `176-10686.xml` without rendering a pixel.
                let logical = Size::<i32, Logical>::from((
                    i32::try_from(display.logical(width)).unwrap_or(i32::MAX),
                    i32::try_from(display.logical(height)).unwrap_or(i32::MAX),
                ));
                let Some(laid) = crate::sign_in_layout::sign_in_laid_out(logical) else {
                    return Err(RenderError::SignInScene);
                };
                let scale = Scale::from(display.factor());
                let at = crate::sign_in_in_pixels::SignInInPixels::of(&laid, scale);
                let refused = if said.is_some() {
                    crate::sign_in_surface::TheEntryIs::Refused
                } else {
                    crate::sign_in_surface::TheEntryIs::Waiting
                };
                // **The flat ground goes: the raster below *is* the ground.**
                //
                // This picture starts with one full-screen `Solid` of
                // `palette.ground`, which the other arms still need.
                // `SignInPicture::paint` draws the surface first and the solids
                // on top of it, so leaving that rectangle here painted white
                // over the whole design.
                //
                // **That is what a booted machine showed on 2026-10-12**: a
                // white screen with forty pixels of ink on it — the greeting,
                // drawn after the solid and so the only thing left. Every test
                // passed, because they asked whether the frame was more than one
                // colour and whether it changed with what was typed, and a
                // rectangle with a word on it is both.
                drawn.solids.clear();

                // The ground, the monogram and the entry, with their corners. The
                // flat `Solid` cannot draw an 18-unit radius, which is why this is
                // a raster — see `crate::sign_in_surface`.
                drawn.surface = crate::sign_in_surface::sign_in_surface(
                    &laid,
                    (width, height),
                    refused,
                    scale,
                    colours,
                );

                // **The initial, in the monogram.** The person's own first letter,
                // and their casing: `docs` is theirs and `Disan` is theirs, and
                // upper-casing it here would be this file deciding how somebody's
                // name looks.
                if let Some(initial) = who.chars().next() {
                    let shaped = sentence(
                        fonts,
                        &initial.to_string(),
                        at.monogram.size.w,
                        measure.metrics(),
                        // **The square it sits in, not the palette ground.** The
                        // box behind a glyph is drawn, so shaping against a
                        // colour the screen does not use leaves a rectangle.
                        colours.monogram,
                        colours.ink,
                    )
                    .cropped_to_its_ink();
                    let x = at.monogram.loc.x + (at.monogram.size.w - shaped.width) / 2;
                    let y = at.monogram.loc.y + (at.monogram.size.h - shaped.height) / 2;
                    drawn.inked.extend(shaped.placed(x, y, height));
                }

                // **The greeting, in the design's own words.**
                //
                // `alo, {name}` through the vocabulary, not a name with a comma
                // written in front of it here: a language may put the name
                // first or greet another way, and the gap is a translator's to
                // move. The name is the person's own, and `disan` in the design
                // file is sample content.
                //
                // It is also the largest line on the screen, which it was not
                // until 2026-10-12: drawn at the ordinary size, it whispered
                // somebody's name instead of greeting them with it.
                let said_hello = words.say(
                    &crate::sign_in_words::GREETING.key(),
                    &alo_strings::Filling::of("name", who),
                );
                let greeting = sentence(
                    fonts,
                    said_hello.text(),
                    at.account.size.w,
                    measure.greeting_metrics(),
                    // **The design's ground**, which is what the raster drew
                    // behind this. `palette.ground` is white, and a white box
                    // on the folded light is a bar across the wallpaper.
                    colours.ground,
                    colours.ink,
                )
                .cropped_to_its_ink();
                let greeting_x = at.account.loc.x + (at.account.size.w - greeting.width) / 2;
                drawn
                    .inked
                    .extend(greeting.placed(greeting_x, at.account.loc.y, height));

                // **What the entry says before anything is typed into it.**
                // Nothing typed, so nothing to mask: the placeholder is drawn
                // only while the field is empty, and never over the dots.
                if !password_typed {
                    let hint = words.say(
                        &crate::sign_in_words::ENTER_YOUR_PIN.key(),
                        &alo_strings::Filling::nothing(),
                    );
                    let shaped = sentence(
                        fonts,
                        hint.text(),
                        // The submit target as drawn, which is
                        // `THE_SUBMIT_TARGET_IS` through the screen's scale.
                        at.entry.size.w - measure.px(42) - at.submit.size.w,
                        measure.metrics(),
                        colours.entry,
                        colours.muted,
                    )
                    .cropped_to_its_ink();
                    let y = at.entry.loc.y + (at.entry.size.h - shaped.height) / 2;
                    drawn
                        .inked
                        .extend(shaped.placed(at.entry.loc.x + measure.px(21), y, height));
                }

                // **The row below the entry**, and the row of essential actions
                // along the bottom. Each was laid out by `crate::sign_in_layout`
                // and answered by `crate::sign_in_pointing` from the morning of
                // 2026-10-12, while **nothing drew a single one of their
                // labels** — four controls a person could press and could not
                // see. This is the half that was missing.
                for (word, box_of, centred, metrics) in [
                    (
                        crate::sign_in_words::OTHER_WAYS,
                        at.other_ways,
                        true,
                        measure.metrics(),
                    ),
                    (
                        crate::sign_in_words::ACCESSIBILITY,
                        at.accessibility,
                        false,
                        measure.label_metrics(),
                    ),
                    (
                        crate::sign_in_words::SWITCH_PERSON,
                        at.switch_person,
                        false,
                        measure.label_metrics(),
                    ),
                ] {
                    if box_of.size.w <= 0 {
                        continue;
                    }
                    let said = words.say(&word.key(), &alo_strings::Filling::nothing());
                    let shaped = sentence(
                        fonts,
                        said.text(),
                        box_of.size.w,
                        metrics,
                        colours.ground,
                        colours.ink,
                    )
                    .cropped_to_its_ink();
                    let x = if centred {
                        box_of.loc.x + (box_of.size.w - shaped.width) / 2
                    } else {
                        box_of.loc.x
                    };
                    let y = box_of.loc.y + (box_of.size.h - shaped.height) / 2;
                    drawn.inked.extend(shaped.placed(x, y, height));
                }

                // **What is typed, masked.** A band per nothing: the design shows
                // four dots and the owner's direction is explicit that they do not
                // prescribe a length, so what is drawn says *something is typed*
                // and never how much.
                if password_typed {
                    let dot = measure.px(8);
                    let gap_between = measure.px(8);
                    for i in 0..4 {
                        drawn.solids.push(Solid {
                            area: Rectangle::new(
                                (
                                    at.entry.loc.x + measure.px(21) + i * (dot + gap_between),
                                    at.entry.loc.y + (at.entry.size.h - dot) / 2,
                                )
                                    .into(),
                                (dot, dot).into(),
                            ),
                            colour: colours.ink,
                        });
                    }
                }

                // **The three pieces of artwork the design draws**, each a
                // glyph: the submit arrow inside the entry, the chevron that
                // goes back to the lock screen, and the power control at the
                // far right of the essential actions.
                //
                // Glyphs rather than shapes, because a `Solid` is an
                // axis-aligned rectangle and every one of these has a diagonal
                // or a curve in it. The submit arrow was one flat bar until
                // 2026-10-12 — a dash where the design draws an arrow, and the
                // only thing on the screen a photograph could find.
                //
                // **A font without one of these draws nothing**, silently,
                // which is how this screen lost all of its text once already.
                // `the_artwork_is_drawn_and_not_merely_placed` reads the ink
                // back for each.
                for (mark, box_of, size_of) in [
                    ("\u{2192}", at.submit, at.arrow.size.w),
                    ("\u{2039}", at.back, at.back.size.w / 2),
                ] {
                    let shaped = sentence(
                        fonts,
                        mark,
                        box_of.size.w.max(1),
                        Metrics::new(size_of.max(8) as f32, (size_of.max(8) as f32) * 1.25),
                        colours.ground,
                        colours.ink,
                    )
                    .cropped_to_its_ink();
                    let x = box_of.loc.x + (box_of.size.w - shaped.width) / 2;
                    let y = box_of.loc.y + (box_of.size.h - shaped.height) / 2;
                    drawn.inked.extend(shaped.placed(x, y, height));
                }

                // **A refusal, said where the design says it**, between the entry
                // and the row below it.
                if let Some(said) = said {
                    let shaped = sentence(
                        fonts,
                        said.text(),
                        at.feedback.size.w,
                        measure.metrics(),
                        colours.ground,
                        colours.ink,
                    )
                    .cropped_to_its_ink();
                    let x = at.feedback.loc.x + (at.feedback.size.w - shaped.width) / 2;
                    drawn
                        .inked
                        .extend(shaped.placed(x, at.feedback.loc.y, height));
                }
            }
            // **As it was drawn before that design arrived.** The lock
            // screen embeds this form in the middle third of its own
            // screen, which the design has nowhere to put - so it keeps
            // the two fields until somebody rules on its composition.
            WhoseScreen::InsideAnother => {
                let under = said.map(|said| {
                    sentence(
                        fonts,
                        said.text(),
                        column,
                        measure.metrics(),
                        palette.ground,
                        palette.ink,
                    )
                });
                let under_height = under
                    .as_ref()
                    .map_or(0, |text| measure.px(24) + text.height);
                let whole = 2 * field_height + gap + under_height;
                // A refusal must fit whole; clipping can otherwise omit it entirely.
                if whole + 2 * measure.px(16) > height {
                    return Err(RenderError::SignInScene);
                }
                let top = ((height - whole) / 2).max(measure.px(16));

                let name_box = Rectangle::new((left, top).into(), (column, field_height).into());
                let password_box = Rectangle::new(
                    (left, top + field_height + gap).into(),
                    (column, field_height).into(),
                );
                let name_waits = field == SignInField::Name;
                framed(&mut drawn.solids, name_box, name_waits, &measure, palette);
                framed(
                    &mut drawn.solids,
                    password_box,
                    !name_waits,
                    &measure,
                    palette,
                );

                let inside = column - 2 * measure.px(14);
                let line = measure.px(24);
                let name_top = top + (field_height - line) / 2;
                let typed = typed_name(fonts, name, inside, &measure, palette.field, palette.ink);
                let caret_at = left + measure.px(14) + typed.width.min(inside);
                drawn
                    .inked
                    .extend(typed.placed(left + measure.px(14), name_top, height));
                if name_waits {
                    drawn
                        .solids
                        .push(caret(caret_at, name_top, line, &measure, palette));
                }

                let password_top = password_box.loc.y + (field_height - line) / 2;
                let band_width = inside - measure.px(6);
                if password_typed {
                    drawn.solids.push(Solid {
                        area: Rectangle::new(
                            (
                                left + measure.px(14),
                                password_top + (line - measure.px(8)) / 2,
                            )
                                .into(),
                            (band_width, measure.px(8)).into(),
                        ),
                        colour: palette.ink,
                    });
                }
                if !name_waits {
                    let after = if password_typed {
                        band_width + measure.px(3)
                    } else {
                        0
                    };
                    drawn.solids.push(caret(
                        left + measure.px(14) + after,
                        password_top,
                        line,
                        &measure,
                        palette,
                    ));
                }

                if let Some(text) = under {
                    let below = password_box.loc.y + field_height + measure.px(24);
                    drawn.inked.extend(text.placed(left, below, height));
                }
            }
        },
    }
    Ok(drawn)
}

/// Measures scaled by the person's text scale, never below one pixel.
pub(crate) struct Measure {
    /// The scale, in percent: the person's text size times the display's.
    percent: i32,
    /// The display's own scale, for a surface that lays itself out in logical
    /// pixels and converts — the first-start sheets.
    display: DisplayScale,
}

impl Measure {
    /// Measures for this scale at one-to-one.
    ///
    /// **For tests only** since every surface that draws with these measures
    /// follows its screen's scale through [`Measure::at`].
    #[cfg(test)]
    pub(crate) fn of(scale: TextScale) -> Self {
        Self {
            percent: i32::from(scale.as_percent()),
            display: DisplayScale::ONE,
        }
    }

    /// Measures for this text size on a screen drawn at `display`: the two
    /// multiplied, so a person who asked for larger text on a dense screen
    /// gets both. At a display of one this is [`Measure::of`] exactly.
    pub(crate) fn at(scale: TextScale, display: DisplayScale) -> Self {
        Self {
            percent: i32::from(scale.as_percent()) * i32::from(display.per_cent()) / 100,
            display,
        }
    }

    /// The display's own scale these measures were made for.
    pub(crate) const fn display(&self) -> DisplayScale {
        self.display
    }

    /// `at_one` pixels at the ordinary scale, scaled.
    pub(crate) fn px(&self, at_one: i32) -> i32 {
        (at_one * self.percent / 100).max(1)
    }

    /// The text metrics most lines on this screen use.
    pub(crate) fn metrics(&self) -> Metrics {
        let factor = self.percent as f32 / 100.0;
        Metrics::new(16.0 * factor, 24.0 * factor)
    }

    /// The greeting.s own size, larger than everything else on the screen
    /// because it is the one line that says whose machine this is.
    ///
    /// `176-10686.xml` gives its box as 44 tall at `176:10998`, which is what
    /// sets the line height. Drawn at the ordinary 16 until 2026-10-12, which
    /// is why a booted machine whispered the person.s name instead of
    /// greeting them with it.
    pub(crate) fn greeting_metrics(&self) -> Metrics {
        let factor = self.percent as f32 / 100.0;
        Metrics::new(32.0 * factor, 44.0 * factor)
    }

    /// The title on a first-start sheet: the same size the sign-in screen's
    /// greeting is drawn at, because both are the one line that says what the
    /// screen is for.
    pub(crate) fn title_metrics(&self) -> Metrics {
        self.greeting_metrics()
    }

    /// What the row of essential actions is lettered at: the smallest text
    /// this screen draws, and still a readable size at every scale.
    pub(crate) fn label_metrics(&self) -> Metrics {
        let factor = self.percent as f32 / 100.0;
        Metrics::new(14.0 * factor, 20.0 * factor)
    }
}

/// A field's edge and inside; the waiting one has the thicker edge.
fn framed(
    solids: &mut Vec<Solid>,
    area: Rectangle<i32, Physical>,
    waiting: bool,
    measure: &Measure,
    palette: Palette,
) {
    let edge = if waiting {
        measure.px(3)
    } else {
        measure.px(1)
    };
    solids.push(Solid {
        area,
        colour: palette.ink,
    });
    solids.push(Solid {
        area: Rectangle::new(
            (area.loc.x + edge, area.loc.y + edge).into(),
            (area.size.w - 2 * edge, area.size.h - 2 * edge).into(),
        ),
        colour: palette.field,
    });
}

/// The caret at the end of what is typed.
fn caret(x: i32, top: i32, line: i32, measure: &Measure, palette: Palette) -> Solid {
    Solid {
        area: Rectangle::new(
            (x, top + measure.px(2)).into(),
            (measure.px(2), line - 2 * measure.px(4)).into(),
        ),
        colour: palette.ink,
    }
}

/// The name as typed, on one line, scrolled so that its end is in view.
fn typed_name(
    fonts: &mut FontSystem,
    name: &str,
    width: i32,
    measure: &Measure,
    ground: [u8; 3],
    ink: [u8; 3],
) -> Shaped {
    let mut buffer = Buffer::new(fonts, measure.metrics());
    buffer.set_wrap(fonts, Wrap::None);
    buffer.set_size(fonts, None, None);
    buffer.set_text(
        fonts,
        name,
        // **alo's own face, as every other surface asks for it.** This said
        // `Family::SansSerif` until 2026-10-12, so a person's own name was the
        // one string on the sign-in screen drawn in whatever sans the font
        // database offered first, beside a greeting in Manrope. One place
        // decides the face — `painted_text::alos_own_text` — so a measurement
        // and a drawing cannot disagree about what they meant.
        &crate::painted_text::alos_own_text(),
        Shaping::Advanced,
    );
    buffer.shape_until_scroll(fonts, false);
    let wide = buffer
        .layout_runs()
        .map(|run| run.line_w)
        .fold(0.0, f32::max)
        .ceil() as i32;
    let shift = (wide - width).max(0);
    let mut shaped = inked(fonts, &buffer, (width, measure.px(24)), -shift, ground, ink);
    shaped.width = wide.min(width);
    shaped
}

#[cfg(test)]
#[path = "sign_in_raster_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "the_first_start_at_one_hundred_tests.rs"]
mod the_first_start_at_one_hundred_tests;

#[cfg(test)]
#[path = "the_first_start_at_two_hundred_tests.rs"]
mod the_first_start_at_two_hundred_tests;
