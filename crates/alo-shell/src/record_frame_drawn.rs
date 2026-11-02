//! The record window's frame: its shadow, its title strip with its icon, its
//! title and Close, its body, and the footer at the body's foot — everything
//! that is the same whatever the record holds.
//!
//! **A window as the owner approved windows on 2026-11-01**: the title always
//! shown with the window's icon (`557:456`, the first answer), drawn in the
//! focused look because the record window is the one the keyboard is in while
//! it is open. Strip and body take that look's colours from
//! `crate::record_palette` and its shadow, *Elevation/Floating*, from the
//! frame.
//!
//! **Minimise and maximise are not drawn**, where `560:177` draws them beside
//! Close: the record window is not a window a person can minimise or make
//! larger yet, and a control that does nothing is worse than none. Close is
//! drawn and does what Escape does.

use cosmic_text::FontSystem;
use smithay::utils::Rectangle;

use crate::record_measures as m;
use crate::record_palette::RecordPalette;
use crate::record_room::Room;
use crate::settings_canvas::{AShadow, Area, Canvas};
use crate::settings_type::{TheType, written};
use crate::the_records_icons::ARecordIcon;

/// *Elevation/Floating*, as `560:176` casts it: 16 down, 40 of blur, 4 in,
/// 14 per cent.
const FLOATING: (i32, i32, i32, f32) = (16, 40, -4, 0.14);

/// *Elevation/Quiet*, the shadow every window the keys are not going to casts
/// (`crate::window_edge_look`): 8 down, 24 of blur, none in, 8 per cent.
const QUIET: (i32, i32, i32, f32) = (8, 24, 0, 0.08);

/// Who draws the window's title strip.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum TheStrip {
    /// The window draws its own, in the focused look — the record fixed to the
    /// display, as the nested lane still draws it.
    ItsOwn,
    /// **The canvas edge draws it** (ADR 0145, stage 4): the band is left to
    /// `crate::an_alo_windows_edge`, the edge every one of alo's windows wears,
    /// and the body wears the focused look while the keys go to the window and
    /// the unfocused one otherwise.
    TheCanvasEdge {
        /// Whether the keys go to the window.
        focused: bool,
    },
}

/// What the frame drew that something else needs to know about.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct TheFrameDrawn {
    /// Close's target, where it is drawn.
    pub(crate) close: Area,
    /// The footer's top: where the list must stop.
    pub(crate) footer_top: i32,
}

/// Draw the frame: the shadow, the strip, the body's ground and edge, and the
/// footer saying `footer`.
pub(crate) fn drawn(
    canvas: &mut Canvas,
    fonts: &mut FontSystem,
    (room, palette): (Room, RecordPalette),
    (title, footer): (&str, &str),
    strip: TheStrip,
) -> TheFrameDrawn {
    let px = |at_one: i32| room.px(at_one);
    let whole = Rectangle::new(
        room.strip.loc,
        (
            room.window.size.w,
            room.body.loc.y + room.body.size.h - room.strip.loc.y,
        )
            .into(),
    );
    let focused = match strip {
        TheStrip::ItsOwn => true,
        TheStrip::TheCanvasEdge { focused } => focused,
    };
    let (falls, blurs, spreads, darkness) = if focused { FLOATING } else { QUIET };
    canvas.shadow(
        whole,
        px(m::THE_BODYS_CORNER) as f32,
        AShadow {
            falls: px(falls) as f32,
            blurs: px(blurs) as f32,
            spreads: -(px(-spreads) as f32),
            darkness,
        },
    );
    if strip == TheStrip::ItsOwn {
        the_strip(canvas, fonts, room, palette, title);
    }
    let line = room.hairline as f32;
    let corner = px(m::THE_BODYS_CORNER) as f32;
    canvas.fill(room.body, corner, palette.surface, 1.0);
    // **The body's line in the look it wears**: `border/strong` focused,
    // `border/default` not — the two looks every window wears.
    let body_line = if focused { palette.frame } else { palette.edge };
    canvas.border(room.body, corner, line, body_line);
    let footer_top = the_footer(canvas, fonts, room, palette, footer);
    let (target, from_the_end, _, _) = m::CLOSE;
    let close = room.as_drawn(Rectangle::new(
        (
            room.window.loc.x + room.window.size.w - px(from_the_end) - px(target),
            room.window.loc.y,
        )
            .into(),
        (px(target), px(target)).into(),
    ));
    TheFrameDrawn { close, footer_top }
}

/// The title strip: top corners rounded, its icon on a tile, its title, and
/// Close at its end.
fn the_strip(
    canvas: &mut Canvas,
    fonts: &mut FontSystem,
    room: Room,
    palette: RecordPalette,
    title: &str,
) {
    let px = |at_one: i32| room.px(at_one);
    let corner = px(m::THE_STRIPS_CORNER);
    // **Rounded at the top only**: the shape runs a corner's depth below the
    // strip and is clipped back to it, so its foot is square.
    let deeper = Rectangle::new(
        room.strip.loc,
        (room.strip.size.w, room.strip.size.h + corner).into(),
    );
    canvas.clipped_to(room.strip);
    canvas.fill(deeper, corner as f32, palette.surface, 1.0);
    canvas.border(deeper, corner as f32, room.hairline as f32, palette.frame);
    canvas.unclipped();
    canvas.fill(
        Rectangle::new(
            (
                room.strip.loc.x,
                room.strip.loc.y + room.strip.size.h - room.hairline,
            )
                .into(),
            (room.strip.size.w, room.hairline).into(),
        ),
        0.0,
        palette.frame,
        1.0,
    );
    let (x, y, icon, tile_corner) = m::THE_ICON;
    let tile = Rectangle::new(
        (room.window.loc.x + px(x), room.window.loc.y + px(y)).into(),
        (px(icon), px(icon)).into(),
    );
    canvas.fill(tile, px(tile_corner) as f32, palette.cool, 1.0);
    ARecordIcon::WhatAloDid.drawn(canvas, tile, palette.ink);
    let (title_x, title_y) = m::THE_TITLE;
    let (target, from_the_end, (mark_x, mark_y), mark) = m::CLOSE;
    let room_for_it = room.window.size.w - px(title_x) - px(target) - px(from_the_end) - px(8);
    let shaped = written(
        fonts,
        title,
        TheType::Label,
        room_for_it,
        room.measure,
        (palette.surface, palette.ink),
    );
    canvas.words(
        room.window.loc.x + px(title_x),
        room.window.loc.y + px(title_y),
        &shaped,
    );
    let close_x = room.window.loc.x + room.window.size.w - px(from_the_end) - px(target);
    ARecordIcon::Close.drawn(
        canvas,
        Rectangle::new(
            (close_x + px(mark_x), room.window.loc.y + px(mark_y)).into(),
            (px(mark), px(mark)).into(),
        ),
        palette.ink,
    );
}

/// The footer at the body's foot: a line over it, the padlock and the words,
/// each in the middle of its height. Answers the footer's top.
fn the_footer(
    canvas: &mut Canvas,
    fonts: &mut FontSystem,
    room: Room,
    palette: RecordPalette,
    footer: &str,
) -> i32 {
    let px = |at_one: i32| room.px(at_one);
    let (tall, _, icon, words_x) = m::THE_FOOTER;
    let shaped = written(
        fonts,
        footer,
        TheType::SmallLabel,
        room.body.size.w - px(words_x) - px(m::PAD),
        room.measure,
        (palette.surface, palette.muted),
    );
    let tall = px(tall).max(shaped.height + 2 * px(8));
    let top = room.body.loc.y + room.body.size.h - tall;
    canvas.fill(
        Rectangle::new(
            (room.body.loc.x, top).into(),
            (room.body.size.w, room.hairline).into(),
        ),
        0.0,
        palette.edge,
        1.0,
    );
    let middle = |height: i32| top + (tall - height) / 2;
    ARecordIcon::Padlock.drawn(
        canvas,
        Rectangle::new(
            (room.body.loc.x + px(m::PAD), middle(px(icon))).into(),
            (px(icon), px(icon)).into(),
        ),
        palette.muted,
    );
    canvas.words(
        room.body.loc.x + px(words_x),
        middle(shaped.height),
        &shaped,
    );
    top
}
