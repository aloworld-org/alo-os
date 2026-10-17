//! **alo's own terminal, opened by its own chord and drawn through a real
//! compositor** — the first check anywhere that a terminal reaches pixels.
//!
//! Until 2026-10-17 nothing did. `direct_desktop` drew the terminal from
//! `drawn_terminal`, and `Nested::submit_with_desktop` passed `None` in the same
//! argument's place, so the only backend a build machine can run drew no
//! terminal at all. The effect was not a slow test or a flaky one: **there was
//! no road by which any gate could see alo's own terminal**, and the four
//! promises it keeps — a terminal reaches a person, who is speaking, measured
//! apart from worked out, scrollback that moves — were checkable on certified
//! hardware and nowhere else.
//!
//! # What this settles, and what it cannot
//!
//! It settles that pressing `Ctrl`+`Alt`+`T` on this seat opens a terminal and
//! that the terminal **reaches a frame**: the same desktop is drawn twice, once
//! with the terminal read from the server and once without, and the two frames
//! must differ.
//!
//! **That second draw is the control, and it is the point of the shape.** A
//! fixture that drew one frame and asserted it was not blank would pass on a
//! machine where the terminal never reached the frame at all, because a desktop
//! paints a dock and a status area whatever else is true. Two draws that must
//! differ cannot pass that way: if `drawn_terminal` returns `None` — no chord,
//! no shell, no terminal — the frames are byte-identical and this fails.
//!
//! It does not settle what the terminal *says*. The shell's first output arrives
//! whenever `bash` gets round to it, and nothing outside this crate can pump the
//! pseudoterminal, so the words on the screen are
//! `alo-terminal`'s own tests' business and the boot walk's. What is new here is
//! that the window is drawn by the compositor rather than asserted about.
//!
//! # What the first frame actually showed, 2026-10-17
//!
//! Written out and looked at rather than inferred from a count, which is how the
//! first version of the check below was found to be wrong:
//!
//! - the gutter word **`program`**, lower case, at the top left of the terminal,
//!   with the cursor block beside it — [ADR 0113]'s gutter, drawn;
//! - **`bash · UTF-8`** along the foot — the shell's own name and the encoding,
//!   which is what step 2 of the boot walk tells a walker to look for;
//! - **no panel and no border.** The terminal draws on the desktop's own ground,
//!   so the only pixels that differ from a desktop without it are the ones
//!   carrying ink. That is why this file measures the *extent* of the difference
//!   and not its area.
//!
//! [ADR 0113]: ../../../docs/decisions/0113-the-terminal-owns-a-gutter-that-says-who-is-speaking.md
//!
//! # Not a certified machine
//!
//! The parent is `weston --backend=headless` and the renderer is llvmpipe, so
//! this says nothing about a real display, a real keyboard or a real GPU. The
//! evening of 2026-10-18 is for that. What it removes is the case where the boot
//! is the *first* time anybody finds out.

use alo_desktops::{DisplayId as DesktopDisplay, Promises};
use alo_dividing::{Area, Point, WindowId, area::Size};
use alo_shell::{
    DesktopLook, EgressStatus, FillingWindow, Nested, RunningWindow, Server, TheTerminalToDraw,
    WindowControlLabels,
};
use smithay::backend::input::KeyState;
use std::os::unix::fs::PermissionsExt as _;

/// The size the design's frames are drawn at.
///
/// A figure from a frame, used here as the parent's size and nothing else —
/// every rectangle inside the frame is derived from it by the code under test,
/// which is what *nothing is built to one screen size* asks.
const HOW_BIG: (u32, u32) = (1440, 960);

/// `Ctrl`, as Linux evdev numbers keys.
const LEFT_CONTROL: u32 = 29;

/// `Alt`, the same.
const LEFT_ALT: u32 = 56;

/// `T`, the same.
const THE_T_KEY: u32 = 20;

/// How this display is named where an appearance is asked about it.
const THE_DISPLAY: &str = "the-terminal-checks-screen";

/// The pixels of the frame just drawn, copied out before the next one replaces
/// them.
fn frame_just_drawn(nested: &Nested, moment: &str) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let kept = nested
        .the_frame_just_drawn()
        .ok_or_else(|| format!("{moment}: no frame was kept, so nothing can be read back"))?;
    Ok(kept.pixels().to_vec())
}

/// How many pixels of a frame were painted something other than black.
///
/// A blank frame is the shape a refused or empty draw comes back as, and it is
/// worth telling apart from *a frame that differs* — the two failures have
/// different causes and a reader of this fixture's output should not have to
/// guess which happened.
fn painted(pixels: &[u8]) -> usize {
    let (pixels, _) = pixels.as_chunks::<4>();
    pixels
        .iter()
        .filter(|pixel| pixel[0] != 0 || pixel[1] != 0 || pixel[2] != 0)
        .count()
}

/// Where two frames differ, as a rectangle, and how many pixels inside it did.
///
/// **A count on its own is not enough to believe**, which this fixture learned
/// the moment it first passed: 829 differing pixels out of 1,382,400 is 0.06% of
/// the screen, far too little for a terminal window and entirely consistent with
/// a glyph cache warming up between two draws. A rectangle can be argued with —
/// a terminal is a region, so its difference has to be one.
fn where_they_differ(
    with: &[u8],
    without: &[u8],
    width: usize,
) -> Option<(usize, usize, usize, usize, usize)> {
    let mut left = usize::MAX;
    let mut top = usize::MAX;
    let mut right = 0;
    let mut bottom = 0;
    let mut how_many = 0;
    let (with, _) = with.as_chunks::<4>();
    let (without, _) = without.as_chunks::<4>();
    for (at, (a, b)) in with.iter().zip(without).enumerate() {
        if a != b {
            let (x, y) = (at % width, at / width);
            left = left.min(x);
            top = top.min(y);
            right = right.max(x);
            bottom = bottom.max(y);
            how_many += 1;
        }
    }
    (how_many > 0).then_some((left, top, right, bottom, how_many))
}

/// Press one key on this seat.
///
/// **What comes back is deliberately ignored, and that is worth a sentence.**
/// `keyboard_key` answers *was a client told*, not *was a chord taken* — a chord
/// is intercepted at the seat and never forwarded, so a chord that worked
/// perfectly answers `false`. A first version of this fixture read that `false`
/// as a failure and reported the chord broken while it was fine. What a chord
/// did is visible in what gets drawn, which is what the two frames below are
/// for.
fn key(server: &mut Server, code: u32, at: u32) -> Result<(), Box<dyn std::error::Error>> {
    server.keyboard_key(code, KeyState::Pressed, at)?;
    Ok(())
}

/// A Dock whose words nobody has said, for a fixture that is not about them.
///
/// A static rather than a temporary, because the frame borrows it.
fn no_words() -> &'static alo_shell::the_docks_words::TheDocksWords {
    static IT: std::sync::OnceLock<alo_shell::the_docks_words::TheDocksWords> =
        std::sync::OnceLock::new();
    IT.get_or_init(alo_shell::the_docks_words::TheDocksWords::default)
}

/// An empty panel, for the same reason.
fn nothing_put_aside() -> &'static alo_put_aside::Panel {
    static EMPTY: std::sync::OnceLock<alo_put_aside::Panel> = std::sync::OnceLock::new();
    EMPTY.get_or_init(alo_put_aside::Panel::new)
}

/// Open a terminal by its chord and draw the desktop twice: with it and without.
pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    let mut nested = Nested::new("alo — a terminal on a real compositor", HOW_BIG)?;
    nested.keep_each_frame(true);

    let words = alo_strings::Strings::of(alo_saying::everything_this_machine_can_say()?);
    let dock = alo_dock::Dock::shipped();
    let running = RunningWindow::closed();
    let filling = FillingWindow::closed();
    let mut labels = WindowControlLabels::new()?;
    let look = DesktopLook::of(
        &alo_appearance::Appearance::shipped(),
        &alo_access::TurnedOn::nothing(),
        alo_appearance::TimeOfDay::checked(9, 41).map_err(|why| format!("{why:?}"))?,
        alo_strings::Direction::LeftToRight,
        &alo_appearance::DisplayId::named(THE_DISPLAY).map_err(|why| format!("{why:?}"))?,
    );

    // The indicator is told that nothing is leaving. A desktop whose indicator
    // has never been told anything is refused before any of it is drawn, which
    // is `frame_pictures`' own rule.
    let mut egress = EgressStatus::on_an_output();
    alo_indicator::Indicating::nowhere().show(Some(&mut egress), &alo_egress::Indicator::default());

    let runtime = tempfile::tempdir()?;
    std::fs::set_permissions(runtime.path(), std::fs::Permissions::from_mode(0o700))?;
    let mut server = Server::bind_keyboard(
        runtime.path(),
        "the-terminal-check",
        smithay::input::keyboard::XkbConfig {
            layout: "us",
            ..Default::default()
        },
    )?;
    // **Without these the chord matches nothing.** `the_chord_a_key_makes`
    // answers `None` for a server that was never given shortcuts, so a fixture
    // that forgot this would press three keys and conclude the terminal was
    // broken.
    server.the_shortcuts_are(alo_shortcuts::Shortcuts::shipped());

    let a_screen = Area::of(Point::at(0, 0), Size::of(HOW_BIG.0, HOW_BIG.1))?;
    server.display_arrived(
        DesktopDisplay::from_compositor(1),
        THE_DISPLAY,
        a_screen,
        Promises::of(
            WindowId::from_compositor(1),
            WindowId::from_compositor(2),
            WindowId::from_compositor(3),
        )?,
        &|_| None,
    )?;

    // **The chord, through the one road both backends share.** `keyboard_key`
    // is where `libinput`'s keys and this backend's keys both arrive, so a
    // terminal opened here is a terminal opened the way a person opens one.
    key(&mut server, LEFT_CONTROL, 1)?;
    key(&mut server, LEFT_ALT, 2)?;
    key(&mut server, THE_T_KEY, 3)?;
    server.keyboard_key(THE_T_KEY, KeyState::Released, 4)?;
    server.keyboard_key(LEFT_ALT, KeyState::Released, 5)?;
    server.keyboard_key(LEFT_CONTROL, KeyState::Released, 6)?;

    let division = alo_dividing::Division::of(a_screen);
    let nothing_offered = alo_dividing::Offer::Nothing;
    let desktop = || alo_shell::DesktopFrame {
        tasks: &[],
        // Nothing opened the market here, and a shut window and a display it is
        // not on are the same thing to a drawing.
        market: None,
        the_dock_has_open: &alo_shell::NOTHING,
        the_docks_words: no_words(),
        dock: &dock,
        look,
        strings: &words,
        egress: &egress,
        running: &running,
        filling: &filling,
        in_use: &[],
        notifications: &[],
        capturing: None,
        division: &division,
        offer: &nothing_offered,
        windows: &[],
        put_aside: nothing_put_aside(),
        panel_is_revealed: true,
        filling_the_screen: false,
        display_scale: 100,
        on_the_dock: &[],
        in_more_apps: &[],
    };

    // **The control first.** The same desktop with the terminal argument left
    // out, which is what every nested caller passed before this change.
    nested.pump()?;
    nested.submit_with_desktop(
        &[],
        &[],
        &alo_shell::Cursor::Default,
        &[],
        &mut labels,
        desktop(),
        None,
        None,
        None,
    )?;
    let without = frame_just_drawn(&nested, "the desktop with no terminal argument")?;

    // **The control, drawn a second time.** Without this the fixture cannot tell
    // the terminal from anything else that changes between two draws — a glyph
    // cache filling, a clock, a cursor blink. Two identical controls say the
    // draw is deterministic, which is what makes a difference in the third draw
    // attributable to the one argument that changed.
    nested.pump()?;
    nested.submit_with_desktop(
        &[],
        &[],
        &alo_shell::Cursor::Default,
        &[],
        &mut labels,
        desktop(),
        None,
        None,
        None,
    )?;
    let again = frame_just_drawn(&nested, "the desktop with no terminal argument, again")?;
    if again != without {
        let drifted = where_they_differ(&again, &without, HOW_BIG.0 as usize);
        return Err(format!(
            "the same desktop drawn twice with no terminal came back different, so this fixture \
             cannot attribute anything to the terminal: {drifted:?} (left, top, right, bottom, \
             how many)"
        )
        .into());
    }

    // And the same desktop again, with the terminal read from the server the
    // chord opened it on.
    nested.pump()?;
    nested.submit_with_desktop(
        &[],
        &[],
        &alo_shell::Cursor::Default,
        &[],
        &mut labels,
        desktop(),
        None,
        None,
        Some(TheTerminalToDraw {
            server: &server,
            display: THE_DISPLAY,
        }),
    )?;
    let with = frame_just_drawn(&nested, "the desktop with alo's own terminal")?;

    if painted(&with) == 0 {
        return Err("the frame holding the terminal came back blank, so nothing was drawn".into());
    }
    if with == without {
        return Err(format!(
            "the frame with alo's own terminal is byte-identical to the frame drawn without it, \
             so no terminal reached the compositor — {} pixels painted either way. Two causes \
             look the same here and both are real: Ctrl+Alt+T opened nothing (the shortcuts, the \
             seat, or no shell on this machine), or it opened a terminal that the frame does not \
             draw. `open_alos_own_terminal` prints its own reason on the error stream, so a \
             machine with no usable shell says so above this line.",
            painted(&with)
        )
        .into());
    }

    // **Written out only when somebody asks**, the way `saving_frames` guards
    // itself: a gate that wrote a file every run would be a gate that fills a
    // disk, and nothing here is the screenshot promise.
    if let Ok(into) = std::env::var("ALO_TERMINAL_CHECK_WRITE") {
        let header = format!("P6\n{} {}\n255\n", HOW_BIG.0, HOW_BIG.1);
        let mut out = header.into_bytes();
        let (whole, _) = with.as_chunks::<4>();
        for pixel in whole {
            out.extend_from_slice(&[pixel[2], pixel[1], pixel[0]]);
        }
        std::fs::write(&into, out)?;
        eprintln!("terminal: the frame with the terminal was written to {into}");
    }

    let (left, top, right, bottom, how_many) =
        where_they_differ(&with, &without, HOW_BIG.0 as usize)
            .ok_or("the frames differ as byte slices but no pixel differs, which cannot happen")?;
    let (wide, tall) = (right - left + 1, bottom - top + 1);

    // **Height, and deliberately not area or width.**
    //
    // The first version of this check asked for a region at least a tenth of the
    // screen wide and failed at 106 pixels — and it was the check that was
    // wrong, found by writing the frame out and looking at it. alo's own
    // terminal draws **on the desktop's own ground**: no panel, no border, no
    // fill of its own, so the only pixels that differ are the ones carrying ink.
    // On a terminal whose shell has not written a prompt yet that is two short
    // runs of text — the gutter word at the top and `bash · UTF-8` at the foot —
    // which is 829 pixels and perfectly correct.
    //
    // What those two runs *do* prove is extent: they sit at opposite ends of the
    // terminal, so the difference spans most of the display's height. A caption,
    // a cursor blink or a warming glyph cache cannot do that, which is what this
    // needs to exclude.
    let least_tall = HOW_BIG.1 as usize / 2;
    if tall < least_tall {
        return Err(format!(
            "what differs is {wide}x{tall} pixels at ({left}, {top})-({right}, {bottom}), and \
             its height is under {least_tall} — too little to span a terminal, whose gutter and \
             its `bash · UTF-8` line sit at opposite ends of it. {how_many} pixels differed in \
             all. Something drew; on this evidence it is not a terminal."
        )
        .into());
    }

    eprintln!(
        "terminal: Ctrl+Alt+T opened one and it reached the frame — ink across a {wide}x{tall} \
         extent from ({left}, {top}), {how_many} pixels differing from the same desktop drawn \
         twice identically without it"
    );
    Ok(())
}
