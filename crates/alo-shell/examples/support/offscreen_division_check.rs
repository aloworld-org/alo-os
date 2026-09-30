//! **A display divided between two real clients, read back off the frame.**
//!
//! Task 17 of `docs/autonomy/v0-5-the-shell-plan.md`, and the hole task 16 left.
//!
//! # What went missing, and how
//!
//! `offscreen_check.rs` walked stages 23 to 28 with a real client through real
//! GLES pixels: a window put on half an output, its configure acknowledged, its
//! buffer attached at the tiled size, and the committed origin read back off the
//! frame. Task 16 removed `crate::window_tiling` and those six stages went with
//! it. **Nothing replaced them.** What a chord does now was held by
//! `alo_shell::window_dividing`'s unit tests, by `tests/shortcut_dispatch/layout.rs`
//! with one client, and by `tests/one_layout_decider.rs` reading the source — and
//! by no pixels at all.
//!
//! They were removed rather than rewritten for a good reason: a division divides
//! *between* two windows, `Division::divide_with_next` takes the focused window and
//! the next one, and the probe drove **one** client. There was no second window to
//! divide with, so the six stages had no honest translation. This gives the probe
//! its second client.
//!
//! # Every rectangle compared comes from the division
//!
//! **Not from numbers written here.** The acceptance is explicit about it, and the
//! reason is the one this repository keeps rediscovering: a probe holding its own
//! copy of the expected geometry is a probe that agrees with itself. So each stage
//! asks `alo_dividing::Division::shares` where the windows belong and compares the
//! frame against that. If the layout changes, this follows it; if the layout is
//! wrong, this says so about the layout rather than about a number.
//!
//! This file decides no layout, which is this task's constraint. It drives clients
//! and reads pixels.
//!
//! # What it cannot tick
//!
//! A physical display. The parent is `weston --backend=headless` or WSLg, both of
//! which composite to nothing. **This is what was drawn and never what a panel
//! showed.**

use alo_dividing::Side;
use alo_shell::{FrameTarget, RenderError, Server};
use smithay::{
    backend::renderer::gles::GlesRenderer,
    reexports::wayland_server::protocol::wl_surface::WlSurface,
    utils::{Physical, Size},
};

/// A sink that submits a real frame and asserts nothing about the picture.
///
/// **Deliberately not `offscreen_client::Target`.** That one verifies the probe's
/// own pixel map — a specific blue-and-green window at specific coordinates — and
/// reusing it here failed on `pixel (0,0)` with white where blue belonged, because
/// these are different windows. A division is checked against
/// `Division::shares` rather than against a stored picture, so what this target
/// owes is only that the production path really submitted.
struct Submitted<'a> {
    /// The real graphics context.
    renderer: &'a mut GlesRenderer,
}

impl FrameTarget for Submitted<'_> {
    fn size(&self) -> Size<i32, Physical> {
        THE_OUTPUT.into()
    }
    fn submit(&mut self, roots: &[WlSurface]) -> Result<Vec<WlSurface>, RenderError> {
        self.submit_popups(roots, &[], &alo_shell::Cursor::Hidden)
    }
    fn submit_popups(
        &mut self,
        roots: &[WlSurface],
        popups: &[alo_shell::Popup],
        cursor: &alo_shell::Cursor,
    ) -> Result<Vec<WlSurface>, RenderError> {
        let prepared =
            alo_shell::render_scanout(self.renderer, THE_OUTPUT.into(), roots, popups, cursor)?;
        Ok(prepared.into_parts().1)
    }
    /// **Retirement is supported, because these stages have to give the output
    /// back.** The trait's default refuses with *target does not support output
    /// retirement*, which is right for a target that owns a real display and wrong
    /// for a fixture sink: there is nothing here to tear down, and the session's own
    /// bookkeeping is what `Server::retire_output` is being called for.
    fn retire(&mut self) -> Result<(), RenderError> {
        Ok(())
    }
}

/// The output every stage here divides, in pixels.
///
/// The probe's own 33x32, so the division is laid out on the same display the rest
/// of this walk uses rather than on one invented for these three stages.
const THE_OUTPUT: (i32, i32) = (33, 32);

/// The display every stage here divides, as the compositor numbers it.
fn the_display() -> alo_desktops::DisplayId {
    alo_desktops::DisplayId::from_compositor(1)
}

/// Drive one division stage and check the frame against the division's own shares.
///
/// # Errors
/// A sentence naming what disagreed: the division's refusal, a share the frame
/// does not show, or a window the division placed somewhere the compositor did not.
pub fn stage(
    server: &mut Server,
    renderer: &mut GlesRenderer,
    stage: u8,
) -> Result<(), Box<dyn std::error::Error>> {
    match stage {
        33 => one_client_is_refused(server, renderer),
        31 => divide(server, renderer, Side::Left, "left and right"),
        32 => {
            // **Checked one stage after it was asked for.** A division sends each
            // client a configure and a window moves when its client answers and
            // commits — so the frame at the end of stage 31 still had both windows
            // where they were, and the first version of this compared the shares
            // against that and reported *the division's shares begin at [(0, 0),
            // (16, 0)] and the compositor drew the windows at [(0, 0), (0, 0)]*.
            //
            // Nothing is wrong there except when it was asked. The client cannot
            // answer while it is waiting for this stage to be acknowledged, so a
            // stage that waited for the answer would deadlock — which is why this
            // probe's other checks are shaped the same way: `interactive_resize`
            // presses at 13, moves at 14 and inspects each stage's *previous*
            // state.
            check(server, renderer, Side::Left, "left and right")?;
            divide(server, renderer, Side::Top, "top and bottom")
        }
        34 => {
            check(server, renderer, Side::Top, "top and bottom")?;
            put_the_display_back(server, renderer)
        }
        other => Err(format!("stage {other} is not a division stage").into()),
    }
}

/// Retire the output these stages needed, so the rest of the walk starts where it
/// used to.
///
/// **The state this borrowed was load-bearing for stages it does not own.** These
/// four run first because a division needs a known window population, and the
/// probe's empty display was the one moment that could be stated without
/// qualification. But *empty* meant more than *no windows*: it meant no output had
/// been submitted, and `window_control_scene_check` at stage 29 asserts that
/// maximising refuses with `OutputUnavailable` — which stopped being true the
/// moment these stages submitted a frame to get themselves a display.
///
/// So the output goes back. The reasoning that put these stages first was right
/// about the window population and never asked what else the starting state
/// carried, which is the whole of the fault: *the cleanest state to borrow* and
/// *a state nothing else depends on* are different claims.
fn put_the_display_back(
    server: &mut Server,
    renderer: &mut GlesRenderer,
) -> Result<(), Box<dyn std::error::Error>> {
    server.retire_output(&mut Submitted { renderer })?;
    println!("The output these stages needed is retired; the walk resumes with none");
    Ok(())
}

/// Submit one frame, which is how the session learns it has a display at all.
///
/// `Server::render` reaches `display_lifecycle`, which gives a display its
/// desktops. Without it every stage here refused with *this session has no display
/// to divide* — and the refusal stage **passed on that**, claiming a chord with one
/// window refuses while proving only that a chord with no display refuses.
fn a_display_to_divide(
    server: &mut Server,
    renderer: &mut GlesRenderer,
) -> Result<(), Box<dyn std::error::Error>> {
    server.render(&mut Submitted { renderer }, 0)?;
    Ok(())
}

/// Divide on this side and require the frame to agree with `Division::shares`.
///
/// **Both windows, both origins and both sizes.** A check that looked at the
/// focused one only would pass a division that laid out half of itself, which is
/// the more likely fault: the focused window is the one the chord names and the
/// other is the one the tree has to find.
fn divide(
    server: &mut Server,
    renderer: &mut GlesRenderer,
    side: Side,
    named: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    a_display_to_divide(server, renderer)?;
    let roots: Vec<_> = server.mapped_surfaces().cloned().collect();
    if roots.len() < 2 {
        return Err(format!(
            "dividing {named} needs two mapped windows and this probe has {}",
            roots.len()
        )
        .into());
    }
    let focused = roots.first().ok_or("a window to focus")?.clone();
    server.keyboard_focus(Some(&focused))?;
    server
        .divide_focused_with_next(&focused, side)
        .map_err(|why| format!("dividing {named} refused: {why}"))?;
    println!("Asked for {named}; the clients answer their configures next");
    Ok(())
}

/// Require the frame to agree with `Division::shares`, now the clients have answered.
fn check(
    server: &mut Server,
    renderer: &mut GlesRenderer,
    side: Side,
    named: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let roots: Vec<_> = server.mapped_surfaces().cloned().collect();
    let focused = roots.first().ok_or("a window to focus")?.clone();
    let division = server
        .dividing(the_display())
        .ok_or_else(|| format!("dividing {named} left no division on the display"))?;
    let shares = division.shares();
    if shares.len() != 2 {
        return Err(format!("dividing {named} made {} shares, not two", shares.len()).into());
    }

    // **The two shares must actually differ on the axis divided**, before anything
    // is compared to the frame. Two identical shares would satisfy every
    // per-window check below while being no division at all — one window exactly
    // on top of the other.
    let (first, second) = (
        shares.first().ok_or("a first share")?.area(),
        shares.get(1).ok_or("a second share")?.area(),
    );
    let apart = match side {
        Side::Left | Side::Right => first.x() != second.x() || first.width() != second.width(),
        Side::Top | Side::Bottom => first.y() != second.y() || first.height() != second.height(),
    };
    if !apart {
        return Err(format!(
            "dividing {named} gave both windows the same share, which is not a division"
        )
        .into());
    }

    // **Both windows stand where the division put them.**
    //
    // Matched as two sets rather than share-by-share, because nothing public maps
    // an `alo_dividing::WindowId` back to a `WlSurface` — `window_number` is a
    // private module of the shell. Two sets agreeing is a weaker claim than each
    // window named, and on its own it would pass a division that swapped the two,
    // so the side the chord actually named is checked separately below. Together
    // they say what naming each would have said.
    let mut wanted: Vec<(i64, i64)> = shares
        .iter()
        .map(|share| {
            let area = share.area();
            (i64::from(area.x()), i64::from(area.y()))
        })
        .collect();
    let mut drawn: Vec<(i64, i64)> = roots
        .iter()
        .map(|root| {
            let at = alo_shell::window_buffer_origin(root);
            (at.x.round() as i64, at.y.round() as i64)
        })
        .collect();
    wanted.sort_unstable();
    drawn.sort_unstable();
    if wanted != drawn {
        return Err(format!(
            "dividing {named}: the division's shares begin at {wanted:?} and the \
             compositor drew the windows at {drawn:?}"
        )
        .into());
    }

    // **And the focused window is on the side the chord named**, which is the half
    // the set comparison above cannot see. A division that laid out both windows
    // correctly and swapped them would satisfy every rectangle and still put a
    // person's window on the wrong side of their screen.
    let focused_at = alo_shell::window_buffer_origin(&focused);
    let other_at = roots
        .iter()
        .find(|root| *root != &focused)
        .map(alo_shell::window_buffer_origin)
        .ok_or("a second window to compare against")?;
    let right_way_round = match side {
        Side::Left => focused_at.x < other_at.x,
        Side::Right => focused_at.x > other_at.x,
        Side::Top => focused_at.y < other_at.y,
        Side::Bottom => focused_at.y > other_at.y,
    };
    if !right_way_round {
        return Err(format!(
            "dividing {named} put the focused window at ({}, {}) and the other at \
             ({}, {}), which is {side:?} the wrong way round",
            focused_at.x, focused_at.y, other_at.x, other_at.y
        )
        .into());
    }

    // And the frame really submits, through the production path, with both windows
    // in it. The pixels are the point of this probe existing.
    let prepared = alo_shell::render_scanout(
        renderer,
        (33, 32).into(),
        &roots,
        &server.popup_surfaces(),
        &alo_shell::Cursor::Hidden,
    )?;
    let painted = prepared
        .pixels()
        .pixels()
        .as_chunks::<4>()
        .0
        .iter()
        .filter(|[b, g, r, _]| *b != 0 || *g != 0 || *r != 0)
        .count();
    if painted == 0 {
        return Err(format!("dividing {named} submitted a frame with nothing in it").into());
    }
    println!(
        "Divided {named} between two real clients: {} shares from the division, \
         both origins read back off the compositor, {painted} pixels submitted; \
         physical display unverified",
        shares.len()
    );
    Ok(())
}

/// **A chord with one client refuses, and the refusal is the stage.**
///
/// A division divides *between* two windows, so with one there is nothing to
/// divide with and `alo-dividing` says so by name. This is the case task 16's
/// design deliberately refuses, and it is worth a stage because *the refusal path
/// tested as carefully as the happy path* is the gate's own words — and because a
/// division that silently did something with one window would be a layout decided
/// somewhere nobody is looking.
fn one_client_is_refused(
    server: &mut Server,
    renderer: &mut GlesRenderer,
) -> Result<(), Box<dyn std::error::Error>> {
    a_display_to_divide(server, renderer)?;
    let roots: Vec<_> = server.mapped_surfaces().cloned().collect();
    let [only] = roots.as_slice() else {
        return Err(format!(
            "this stage asks what happens with one window and there are {}",
            roots.len()
        )
        .into());
    };
    server.keyboard_focus(Some(only))?;
    // **How many windows have a share**, counting no division at all as none.
    //
    // This read `Option<usize>` and compared the options, and it reported *a
    // refused division changed the display's shares from None to Some(0)* — the
    // attempt creates the division object and lays nothing out in it. That is a
    // distinction without a difference for what this stage claims: *no window got a
    // share* is true of both, and the check now says that instead of comparing the
    // shapes of two options.
    let laid_out = |server: &Server| {
        server
            .dividing(the_display())
            .map_or(0, |it| it.shares().len())
    };
    let before = laid_out(server);
    match server.divide_focused_with_next(only, Side::Left) {
        Ok(()) => Err("a chord divided one window, which is not a division".into()),
        // **The refusal has to be the right refusal**, and the first version of
        // this stage did not check which. It passed on
        // `NotDivided::NoDisplay` — *this session has no display to divide* —
        // because the stage ran before the probe had submitted a frame and so
        // before the session had a display at all. So it asserted *a chord with one
        // window refuses* while proving only that a chord with no display refuses,
        // and it would have passed with one-window division completely broken.
        //
        // `Refused::NothingToShareWith` is what this stage is about: *there is no
        // second window to divide the screen with*. Anything else is a different
        // fact and is refused as one.
        Err(alo_shell::NotDivided::Dividing(alo_dividing::Refused::NothingToShareWith)) => {
            // And it changed nothing. A refusal that had already moved the window
            // would be a refusal in name only.
            let after = laid_out(server);
            if after != before || after != 0 {
                return Err(format!(
                    "a refused division left {after} windows with a share where {before} \
                     had one; a refusal must lay nothing out"
                )
                .into());
            }
            println!(
                "A chord with one window refused with `NothingToShareWith` and changed \
                 nothing; physical display unverified"
            );
            Ok(())
        }
        Err(other) => Err(format!(
            "a chord with one window refused, but for the wrong reason: {other}. This \
             stage is about there being nothing to divide with, and any other refusal \
             would let it pass while one-window division was broken"
        )
        .into()),
    }
}
