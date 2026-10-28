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
//! - **no panel and no border** on that day. The terminal drew on the desktop's
//!   own ground, so the only pixels that differed were ink. Since 2026-10-28 it
//!   has a white panel and a hairline, and `is_the_windows_own` takes those out
//!   of every count of ink. This file still measures the *extent* of the
//!   difference and not its area.
//!
//! # And then a person used it
//!
//! Since 2026-10-17 this also types `echo hello`, waits for the shell, and draws
//! again. The frame holds the prompt, the command, `hello` under it and a fresh
//! prompt with the cursor on it — so **keys reach a shell through the seat and
//! what the shell answers reaches the grid and then a frame**, which is the
//! whole of the terminal working and had never been shown anywhere.
//!
//! **The gutter says `program` for all of it, including the line the person
//! typed**, and that is correct rather than a fault. A pseudoterminal echoes
//! what is typed, so those bytes arrive from the program like any others, and
//! [ADR 0113] forbids reading bytes to decide who spoke. Telling a person's own
//! line from a program's output needs the shell to say so, which is
//! the subject of the shell-integration decision this lane proposed on
//! 2026-10-17, which has not landed and so is deliberately not cited by number
//! here — `alo-citing` refuses a pointer to a decision that is not in the tree,
//! and it is right to. A fixture that expected `you` here would be asking the
//! terminal to guess.
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

/// `c`, for the `Ctrl`+`C` that ends a command still in front of the shell.
const THE_C_KEY: u32 = 46;

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
///
/// **Counted over ink, not over every changed pixel**: since 2026-10-28 the
/// terminal's white panel differs from the desktop over the whole window, and a
/// rectangle drawn round the panel would pass with not one character on it.
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
        if is_ink(a, b) {
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

/// Which way a wheel notch goes.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Back {
    /// Pushed away from the person, into what is remembered.
    Yes,
    /// Pulled towards them, back to the newest line.
    No,
}

/// How many notches this fixture will send before calling a scroll endless.
const AT_MOST_NOTCHES: u32 = 200;

/// One wheel notch.
///
/// **What comes back is not the assertion, and a first version of this read it
/// as one.** `Server::pointer_axis` answers `true` when *something* consumed the
/// scroll, and with a pointer enabled and nothing focused the canvas consumes
/// whatever the terminal does not — so it answered `true` two hundred notches
/// past the oldest line. Whether the transcript moved is a question only the
/// drawn frame can answer.
fn a_notch(server: &mut Server, back: Back, time: u32) -> Result<bool, Box<dyn std::error::Error>> {
    /// A wheel notch, in the units an axis frame carries.
    const A_NOTCH: f64 = 15.0;
    let how_far = if back == Back::Yes { -A_NOTCH } else { A_NOTCH };
    let frame = smithay::input::pointer::AxisFrame::new(time)
        .source(smithay::backend::input::AxisSource::Wheel)
        .value(smithay::backend::input::Axis::Vertical, how_far);
    Ok(server.pointer_axis(frame)?)
}

/// Whether a pixel is one of the terminal window's own flat colours: its
/// panel or the hairline round it.
///
/// **Ink is what differs from the bare desktop and is not the window itself.**
/// Until 2026-10-28 the terminal had no panel, so every pixel that differed was
/// ink. It has a white panel and a hairline now, which differ from the desktop
/// over the whole window and would make every count below a constant.
///
/// The kept frame stores B, G, R and a zero (`crate::readback`), and this
/// fixture's desktop is the shipped appearance at 09:41, which is light.
fn is_the_windows_own(pixel: &[u8; 4]) -> bool {
    let [blue, green, red, _] = *pixel;
    [
        alo_appearance::Role::BgSurface.colour(),
        alo_appearance::Role::BorderDefault.colour(),
    ]
    .iter()
    .any(|it| [blue, green, red] == [it.blue(), it.green(), it.red()])
}

/// Whether a pixel of a frame is the terminal's ink: not the bare desktop's, and
/// not the window's own panel or hairline.
fn is_ink(here: &[u8; 4], there: &[u8; 4]) -> bool {
    here != there && !is_the_windows_own(here)
}

/// How much ink a frame has inside one band of rows.
///
/// **The band is measured, never written down.** It comes from where the answer
/// to *why is it waiting* appeared, so this fixture needs no y-coordinate of its
/// own — which would be a figure that holds at one display size and nowhere
/// else.
fn ink_in_band(frame: &[u8], bare: &[u8], top: usize, bottom: usize) -> usize {
    let (frame, _) = frame.as_chunks::<4>();
    let (bare, _) = bare.as_chunks::<4>();
    let wide = HOW_BIG.0 as usize;
    (top..=bottom)
        .flat_map(|y| (0..wide).map(move |x| y * wide + x))
        .filter(|at| match (frame.get(*at), bare.get(*at)) {
            (Some(here), Some(there)) => is_ink(here, there),
            _ => false,
        })
        .count()
}

/// How many rows of the screen hold something the bare desktop does not.
///
/// A row is a band of pixels: a line of text lights several of them, so this
/// counts bands that hold any difference and not pixels. What it is for is
/// telling *two more lines appeared* from *more ink appeared*, which a pixel
/// count cannot do.
fn rows_with_ink(frame: &[u8], bare: &[u8]) -> usize {
    let (frame, _) = frame.as_chunks::<4>();
    let (bare, _) = bare.as_chunks::<4>();
    let wide = HOW_BIG.0 as usize;
    let mut rows = 0;
    let mut lit = false;
    for y in 0..HOW_BIG.1 as usize {
        let any = (0..wide).any(|x| {
            frame.get(y * wide + x).is_some_and(|here| {
                bare.get(y * wide + x)
                    .is_some_and(|there| is_ink(here, there))
            })
        });
        if any && !lit {
            rows += 1;
        }
        lit = any;
    }
    rows
}

/// How many pixels of a frame differ from the bare desktop — the terminal's own
/// ink, however little of the screen it covers.
fn painted_ink(frame: &[u8], bare: &[u8]) -> usize {
    let (frame, _) = frame.as_chunks::<4>();
    let (bare, _) = bare.as_chunks::<4>();
    frame.iter().zip(bare).filter(|(a, b)| is_ink(a, b)).count()
}

/// `echo hello`, as Linux evdev numbers the keys.
const ECHO_HELLO: [u32; 10] = [18, 46, 35, 24, 57, 35, 18, 38, 38, 24];

/// `Enter`.
const THE_ENTER_KEY: u32 = 28;

/// `cat`, as Linux evdev numbers the keys — a command that waits on the
/// keyboard and prints nothing, which is walk step 5's whole point.
const THE_WORD_CAT: [u32; 3] = [46, 30, 20];

/// `W`, for *why is it waiting*.
const THE_W_KEY: u32 = 17;

/// Long enough for a shell to have started a command.
///
/// **A fixed wait, and the only one here.** Everything else waits for something
/// it can observe; `cat` prints nothing, so there is nothing to observe and no
/// honest way to wait for it. If this is ever too short the symptom is the
/// waiting answer saying the command is not waiting at all, which the check
/// below reports rather than passing.
const LONG_ENOUGH_TO_START: std::time::Duration = std::time::Duration::from_millis(600);

/// `D`, which with `Ctrl` ends a command reading the keyboard.
const THE_D_KEY: u32 = 32;

/// `Shift`.
const LEFT_SHIFT: u32 = 42;

/// The characters this fixture types, and the evdev key and shift for each.
///
/// **Only the characters it needs.** A general keyboard map belongs in
/// `alo-keyboards` and is not what a drawing fixture is for; this is the
/// smallest table that can type one `echo` of a line the result recogniser
/// knows, and a character outside it is a compile error rather than a silent
/// substitution.
fn a_key_for(letter: char) -> Option<(u32, bool)> {
    // **The marks that are a shifted key rather than one of their own**, on the
    // layout this fixture types against.
    match letter {
        ':' => return Some((39, true)),
        '(' => return Some((10, true)),
        ')' => return Some((11, true)),
        // **A diff is made of these three**, and none of them is a letter:
        // `+++ b/file`, `@@ -12,4 +12,4 @@`, and the `+` on every added line.
        '+' => return Some((13, true)),
        '@' => return Some((3, true)),
        '/' => return Some((53, false)),
        ',' => return Some((51, false)),
        '=' => return Some((13, false)),
        '{' => return Some((26, true)),
        '}' => return Some((27, true)),
        _ => {}
    }
    // **The whole alphabet and the ten digits, rather than the handful a
    // previous step happened to need.** This answered `None` for `B` until
    // 2026-10-18, which is a fixture refusing to type a capital letter — and a
    // step that needed one read as a product fault for a minute.
    let code = match letter.to_ascii_lowercase() {
        'a' => 30,
        'b' => 48,
        'c' => 46,
        'd' => 32,
        'e' => 18,
        'f' => 33,
        'g' => 34,
        'h' => 35,
        'i' => 23,
        'j' => 36,
        'k' => 37,
        'l' => 38,
        'm' => 50,
        'n' => 49,
        'o' => 24,
        'p' => 25,
        'q' => 16,
        'r' => 19,
        's' => 31,
        't' => 20,
        'u' => 22,
        'v' => 47,
        'w' => 17,
        'x' => 45,
        'y' => 21,
        'z' => 44,
        '1' => 2,
        '2' => 3,
        '3' => 4,
        '4' => 5,
        '5' => 6,
        '6' => 7,
        '7' => 8,
        '8' => 9,
        '9' => 10,
        '0' => 11,
        ' ' => 57,
        '.' => 52,
        ';' => 39,
        '\'' => 40,
        '-' => 12,
        _ => return None,
    };
    // A letter's own key, shifted when the letter is a capital.
    Some((code, letter.is_ascii_uppercase()))
}

/// Type a line, a character at a time, with `Shift` held where it is needed.
fn typing(server: &mut Server, line: &str, from: u32) -> Result<u32, Box<dyn std::error::Error>> {
    let mut at = from;
    for letter in line.chars() {
        let (code, shifted) =
            a_key_for(letter).ok_or_else(|| format!("this fixture has no key for {letter:?}"))?;
        if shifted {
            server.keyboard_key(LEFT_SHIFT, KeyState::Pressed, at)?;
        }
        typed(server, code, at + 1)?;
        if shifted {
            server.keyboard_key(LEFT_SHIFT, KeyState::Released, at + 3)?;
        }
        at += 4;
    }
    Ok(at)
}

/// How long this fixture waits for a shell to say anything.
///
/// **Generous on purpose.** A build machine under load starts `bash` slowly, and
/// the failure this bound is meant to catch is *nothing ever arrives*, not *it
/// took a while*.
const PATIENCE: std::time::Duration = std::time::Duration::from_secs(10);

/// Read the pseudoterminal until the shell says something, and answer whether
/// it did.
///
/// **This is the road `direct_desktop` takes once a frame**, and the nested
/// backend takes it nowhere, which is why no nested check had ever seen a
/// character a shell wrote.
fn until_the_shell_writes(server: &mut Server) -> bool {
    let until = std::time::Instant::now() + PATIENCE;
    while std::time::Instant::now() < until {
        if server.the_terminal_was_written_to() {
            return true;
        }
        std::thread::sleep(std::time::Duration::from_millis(20));
    }
    false
}

/// Keep reading until the shell has been quiet for a moment, so a frame is
/// drawn after the whole of a command's output rather than the first chunk of
/// it.
fn until_the_shell_is_quiet(server: &mut Server) {
    let until = std::time::Instant::now() + PATIENCE;
    let mut quiet_since = std::time::Instant::now();
    while std::time::Instant::now() < until {
        if server.the_terminal_was_written_to() {
            quiet_since = std::time::Instant::now();
        } else if quiet_since.elapsed() > std::time::Duration::from_millis(300) {
            return;
        }
        std::thread::sleep(std::time::Duration::from_millis(20));
    }
}

/// Type one key and release it.
fn typed(server: &mut Server, code: u32, at: u32) -> Result<(), Box<dyn std::error::Error>> {
    server.keyboard_key(code, KeyState::Pressed, at)?;
    server.keyboard_key(code, KeyState::Released, at + 1)?;
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

/// **One frame as a PPM, for somebody to look at rather than count.**
///
/// Guarded by its caller the way `saving_frames` guards itself: a gate that
/// wrote a file every run would be a gate that fills a disk, and nothing here
/// is the screenshot promise.
///
/// The frames arrive as `BGRx`, so the three bytes are reordered on the way
/// out. **Looking is not a nice-to-have in this file**: the width assertion
/// this check used to make was wrong for a working terminal and no pixel count
/// would have found it — somebody opening the frame did.
fn written_out(frame: &[u8], into: &str, moment: &str) -> Result<(), Box<dyn std::error::Error>> {
    let header = format!("P6\n{} {}\n255\n", HOW_BIG.0, HOW_BIG.1);
    let mut out = header.into_bytes();
    let (whole, _) = frame.as_chunks::<4>();
    for pixel in whole {
        out.extend_from_slice(&[pixel[2], pixel[1], pixel[0]]);
    }
    std::fs::write(into, out)?;
    eprintln!("terminal: {moment} written to {into}");
    Ok(())
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

    // **Wait for the shell to say something before drawing anything.** Without
    // this the terminal is drawn with an empty grid and the whole check is
    // about its chrome. With it, the frames below hold a real prompt.
    if !until_the_shell_writes(&mut server) {
        return Err(
            "no shell wrote anything within the time allowed, so either Ctrl+Alt+T opened no \
             terminal or this machine has no usable shell. `open_alos_own_terminal` prints its \
             own reason on the error stream above this line."
                .into(),
        );
    }
    until_the_shell_is_quiet(&mut server);

    let division = alo_dividing::Division::of(a_screen);
    let nothing_offered = alo_dividing::Offer::Nothing;
    let desktop = || alo_shell::DesktopFrame {
        tasks: &[],
        // Nothing opened the market here, and a shut window and a display it is
        // not on are the same thing to a drawing.
        market: None,
        what_each_application_draws: &alo_dock::artwork::NOTHING,
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
        the_keyboard_is_on_the_bar: None,
        the_bar: None,
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

    let (left, top, right, bottom, how_many) =
        where_they_differ(&with, &without, HOW_BIG.0 as usize)
            .ok_or("the frames differ as byte slices but no pixel differs, which cannot happen")?;
    let (wide, tall) = (right - left + 1, bottom - top + 1);

    // **Height, and deliberately not area or width.**
    //
    // The first version of this check asked for a region at least a tenth of the
    // screen wide and failed at 106 pixels — and it was the check that was
    // wrong, found by writing the frame out and looking at it. alo's own
    // terminal drew **on the desktop's own ground** then: no panel, no border, no
    // fill of its own. It has a panel since 2026-10-28, which would span the
    // height by itself, so `is_ink` leaves the panel out and this still measures
    // only the pixels carrying ink.
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

    // **And now something a person did.** Everything above is the terminal
    // existing; this is the terminal *working* — keys reaching a shell through
    // the seat, and what the shell answers reaching the grid and then a frame.
    for (which, code) in ECHO_HELLO.into_iter().enumerate() {
        let at = 10 + u32::try_from(which)? * 2;
        typed(&mut server, code, at)?;
    }
    typed(&mut server, THE_ENTER_KEY, 40)?;
    if !until_the_shell_writes(&mut server) {
        return Err(
            "nothing came back from the shell after `echo hello` and Enter, so either the keys \
             did not reach it or it did not answer. The terminal drew, so the first is the more \
             likely: `the_terminal_is_taking_the_keys` decides that, and it is false when no \
             shell is running."
                .into(),
        );
    }
    until_the_shell_is_quiet(&mut server);

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
    let typed_in = frame_just_drawn(&nested, "the terminal after `echo hello`")?;

    if typed_in == with {
        return Err(
            "the frame after typing `echo hello` and Enter is byte-identical to the frame \
             before, so nothing a person did reached the screen. The shell answered — this \
             check got past that — so what did not happen is the answer reaching the grid or \
             the grid reaching the frame."
                .into(),
        );
    }
    let (_, _, _, _, after_typing) = where_they_differ(&typed_in, &with, HOW_BIG.0 as usize)
        .ok_or("the frames differ as byte slices but no pixel differs, which cannot happen")?;

    // **More ink than before, not merely different.** A cursor moving one cell
    // changes a frame too; two lines of a command and its answer cannot be
    // mistaken for that.
    if painted_ink(&typed_in, &without) <= painted_ink(&with, &without) {
        return Err(format!(
            "the terminal has no more ink after `echo hello` than before it ({} against {}), so \
             what arrived was not a command and its answer",
            painted_ink(&typed_in, &without),
            painted_ink(&with, &without)
        )
        .into());
    }

    // **Walk step 5 and step 6: a command that waits, and alo saying why.**
    //
    // `cat` with no arguments reads the keyboard and prints nothing, so nothing
    // arrives to wait for and the only honest wait is a fixed one. Then
    // `Ctrl`+`Alt`+`W`, which is alo's own answer rather than anything the
    // shell said — the first time that answer has been drawn anywhere.
    for (which, code) in THE_WORD_CAT.into_iter().enumerate() {
        let at = 50 + u32::try_from(which)? * 2;
        typed(&mut server, code, at)?;
    }
    typed(&mut server, THE_ENTER_KEY, 60)?;
    until_the_shell_is_quiet(&mut server);
    std::thread::sleep(LONG_ENOUGH_TO_START);
    let _ = server.the_terminal_was_written_to();

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
    let waiting = frame_just_drawn(&nested, "the terminal with `cat` waiting")?;

    key(&mut server, LEFT_CONTROL, 70)?;
    key(&mut server, LEFT_ALT, 71)?;
    key(&mut server, THE_W_KEY, 72)?;
    server.keyboard_key(THE_W_KEY, KeyState::Released, 73)?;
    server.keyboard_key(LEFT_ALT, KeyState::Released, 74)?;
    server.keyboard_key(LEFT_CONTROL, KeyState::Released, 75)?;
    let _ = server.the_terminal_was_written_to();

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
    let answered = frame_just_drawn(&nested, "the terminal after Ctrl+Alt+W")?;

    if answered == waiting {
        return Err(
            "pressing Ctrl+Alt+W changed nothing on the screen, so the answer to *why is it \
             waiting* reached no frame. The chord is bound to Action::WhyIsItWaiting in \
             alo-shortcuts' shipped defaults, and `cat` was running, so what did not happen is \
             the answer being drawn."
                .into(),
        );
    }
    let said = painted_ink(&answered, &without);
    if said <= painted_ink(&waiting, &without) {
        return Err(format!(
            "Ctrl+Alt+W left no more ink than before it ({said} against {}), so whatever changed \
             was not a line of text being added",
            painted_ink(&waiting, &without)
        )
        .into());
    }

    // **And the answer is beside the gutter word, not inside it.**
    //
    // This is the assertion the whole fixture earned. Until 2026-10-17 alo's
    // sentence was drawn into the gutter's own 80-pixel label box and then
    // painted over by the word `alo`, so the picture held both runs, a person
    // saw `alo` beside nothing, and every count-based check passed. A chord
    // whose answer lands only in the gutter column is that fault returning.
    let (_, _, right, _, _) = where_they_differ(&answered, &waiting, HOW_BIG.0 as usize)
        .ok_or("the frames differ as byte slices but no pixel differs, which cannot happen")?;
    let the_gutter_ends_by = HOW_BIG.0 as usize / 4;
    if right < the_gutter_ends_by {
        return Err(format!(
            "everything Ctrl+Alt+W changed is left of x={the_gutter_ends_by} — the gutter column \
             — ending at x={right}. The word saying who spoke was drawn and the sentence it \
             labels was not, which is what a person reads as alo answering with nothing."
        )
        .into());
    }

    // The rows alo's answer occupies, measured rather than written down, so the
    // step below can ask whether it went away without knowing where it was.
    let (_, alos_top, _, alos_bottom, _) =
        where_they_differ(&answered, &waiting, HOW_BIG.0 as usize)
            .ok_or("the answer's own rows could not be measured")?;
    let while_waiting = ink_in_band(&answered, &without, alos_top, alos_bottom);

    // **Walk step 7: `Ctrl`+`D` ends `cat`, and asking again must take the line
    // away.** A reply about a command that has gone would sit on the screen
    // looking current, which is what `WhatAloSaid::nothing_to_say` exists to
    // prevent — and nothing had ever checked that it does on a screen.
    key(&mut server, LEFT_CONTROL, 80)?;
    key(&mut server, THE_D_KEY, 81)?;
    server.keyboard_key(THE_D_KEY, KeyState::Released, 82)?;
    server.keyboard_key(LEFT_CONTROL, KeyState::Released, 83)?;
    until_the_shell_is_quiet(&mut server);

    key(&mut server, LEFT_CONTROL, 84)?;
    key(&mut server, LEFT_ALT, 85)?;
    key(&mut server, THE_W_KEY, 86)?;
    server.keyboard_key(THE_W_KEY, KeyState::Released, 87)?;
    server.keyboard_key(LEFT_ALT, KeyState::Released, 88)?;
    server.keyboard_key(LEFT_CONTROL, KeyState::Released, 89)?;
    let _ = server.the_terminal_was_written_to();

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
    let nothing_waiting = frame_just_drawn(&nested, "the terminal after `cat` ended")?;
    let after_it_ended = ink_in_band(&nothing_waiting, &without, alos_top, alos_bottom);
    if after_it_ended >= while_waiting {
        return Err(format!(
            "`cat` ended and asking again left {after_it_ended} pixels of ink where the answer \
             was, against {while_waiting} while it really was waiting. The reply is still on the \
             screen for a command that has gone, which is worse than no reply: a person reads it \
             as current."
        )
        .into());
    }

    // **Walk steps 10 and 11: a result the terminal recognises in a stranger's
    // output.** One `echo` of the line `alo_terminal::a_test_result_in` knows
    // how to read. What should appear is a summary and, under it, the line
    // saying nobody checked it — both in the **program's** region, which is the
    // design's *PROGRAM includes parsed rich results*.

    // **Quoted, and the quotes are the point.** Typed bare, bash reads every `;`
    // as the end of a command: the screen fills with `1: command not found` and
    // what reaches the recogniser is a truncated line rather than the one the
    // walk types. Measured on 2026-10-17 by typing it both ways and reading the
    // screen back.
    let at = typing(&mut server, "echo 'test result", 100)?;
    let at = typing(&mut server, ": ok. 23 passed; 1 failed; 0 ignored;'", at)?;
    typed(&mut server, THE_ENTER_KEY, at + 2)?;
    if !until_the_shell_writes(&mut server) {
        return Err("nothing came back from the shell after the result line".into());
    }
    until_the_shell_is_quiet(&mut server);

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
    let recognised = frame_just_drawn(&nested, "the terminal with a recognised result")?;

    // **Rows of text, not pixels of ink.** One `echo` of a quoted line puts two
    // rows on the screen — the command and its output — and a result the
    // terminal recognises adds two more: the summary and the line saying nobody
    // checked it. A pixel count cannot tell four new rows from a shell printing
    // three errors, and both grow the ink.
    //
    // **What was drawn when this was read back on 2026-10-17**: `Tests · 23 tests
    // passed · 1 test failed` and `Source: cargo test output · Not independently
    // verified`, beneath the grid where alo's own lines go. Walk steps 10 and 11
    // are about those two.
    //
    // **That first line is no longer what is drawn, and this check cannot tell.**
    // #766 shortened the summary to the design's own `Tests · 23 passed · 1
    // failed` — `alo_terminal::words` carries it as `{passed} passed`, with a note
    // saying the two English plural forms are deliberately identical. The
    // assertion below counts **rows of ink** and never compares a string, so it
    // passed before #766 and passes after it. Measured 2026-10-18 by grepping
    // this file: the only mentions of the summary's words are the `echo` above
    // and this comment.
    //
    // **The shorter form has now been read off the pixels, 2026-10-18.** This
    // check was run on `009343a5` and the frame below was written out and
    // opened, and the band at y 620..860 of a 1440x960 frame holds, in two
    // lines:
    //
    // ```text
    // Tests · 23 passed · 1 failed
    // Source: cargo test output · Not independently verified
    // ```
    //
    // *That is what walk step 10 asks for, and until that frame was opened
    // nobody had confirmed it: the sentence in the lane order saying a machine
    // from `main` draws the shorter form was a reading of `alo_terminal::words`
    // and not of a screen.*
    //
    // **The assertion below still counts rows and still cannot see a word**, so
    // it would not catch the wording going back. Making it see one needs a
    // reference draw of the expected string to compare the band against; that
    // is still worth doing and is still not done. What changed is that the
    // claim is now a measurement — and a measurement of one frame on one day,
    // which is why `written_out` exists: the next person re-reads it rather
    // than trusting this comment.
    let rows_before = rows_with_ink(&nothing_waiting, &without);
    let rows_after = rows_with_ink(&recognised, &without);
    if rows_after < rows_before + 4 {
        return Err(format!(
            "the screen gained {} row(s) of text where a recognised result gives four — the \
             command, its output, the summary and the line under it. Either the shell did not \
             print the line, or nothing was recognised in what it printed.",
            rows_after - rows_before
        )
        .into());
    }
    let _ = said;

    // **Walk steps 12 and 13: what a recognised result does afterwards.**
    //
    // The band is the one alo's answer used at step 6, which is where the
    // result's first line now sits — so this asks about the summary without
    // ever naming a row. Step 12: another command runs and the summary
    // **stays**, because what is recognised is what the screen holds. Step 13:
    // `clear` takes the line it was read from away, and the summary **goes**
    // with it.
    let summary_showing = ink_in_band(&recognised, &without, alos_top, alos_bottom);
    if summary_showing == 0 {
        return Err("the recognised summary is not in the band this step measures".into());
    }

    let at = typing(&mut server, "ls", 500)?;
    typed(&mut server, THE_ENTER_KEY, at + 2)?;
    until_the_shell_writes(&mut server);
    until_the_shell_is_quiet(&mut server);
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
    let after_more_output = frame_just_drawn(&nested, "the terminal after `ls`")?;
    if ink_in_band(&after_more_output, &without, alos_top, alos_bottom) == 0 {
        return Err(
            "the recognised summary went away when another command ran, though the line it was \
             read from is still on the screen above. What is recognised is what the screen \
             holds, and this screen still holds it."
                .into(),
        );
    }

    let at = typing(&mut server, "clear", 600)?;
    typed(&mut server, THE_ENTER_KEY, at + 2)?;
    until_the_shell_writes(&mut server);
    until_the_shell_is_quiet(&mut server);
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
    let after_clear = frame_just_drawn(&nested, "the terminal after `clear`")?;
    let left_behind = ink_in_band(&after_clear, &without, alos_top, alos_bottom);
    if left_behind >= summary_showing {
        return Err(format!(
            "`clear` took the line away and the summary stayed: {left_behind} pixels of ink in \
             its band against {summary_showing} before. A summary read out of a line that is no \
             longer on the screen is a claim about nothing."
        )
        .into());
    }

    // **Walk step 9: scrollback moves, and stops at both ends.**
    //
    // `seq 1 40` puts more lines on the screen than the grid holds, so there is
    // something to scroll back into. A notch is 15 units of the vertical axis
    // and moves three lines, and `pointer_axis` answers whether alo's terminal
    // took it — which is the whole of *a notch that moves nothing is not
    // consumed*: at either end the scroll falls through instead of being
    // swallowed.
    server.enable_pointer()?;
    let at = typing(&mut server, "seq 1 40", 700)?;
    typed(&mut server, THE_ENTER_KEY, at + 2)?;
    until_the_shell_writes(&mut server);
    until_the_shell_is_quiet(&mut server);
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
    let at_the_newest = frame_just_drawn(&nested, "the terminal at the newest line")?;

    // **Notch by notch, drawing each time, until the picture stops changing.**
    // That is the only honest reading of *stops at both ends*: the question is
    // whether the transcript moved, and the frame is what knows.
    let drawn = |nested: &mut Nested,
                 server: &Server,
                 labels: &mut WindowControlLabels|
     -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        nested.pump()?;
        nested.submit_with_desktop(
            &[],
            &[],
            &alo_shell::Cursor::Default,
            &[],
            labels,
            desktop(),
            None,
            None,
            Some(TheTerminalToDraw {
                server,
                display: THE_DISPLAY,
            }),
        )?;
        frame_just_drawn(nested, "a frame while scrolling")
    };

    let mut back = at_the_newest.clone();
    let mut notches = 0;
    loop {
        a_notch(&mut server, Back::Yes, 800 + notches)?;
        let now = drawn(&mut nested, &server, &mut labels)?;
        if now == back {
            break;
        }
        back = now;
        notches += 1;
        if notches > AT_MOST_NOTCHES {
            return Err(format!(
                "the transcript moved for {AT_MOST_NOTCHES} notches without reaching the oldest \
                 line it remembers. It should stop, and a terminal at the end of its scrollback \
                 does not swallow scrolls."
            )
            .into());
        }
    }
    if notches == 0 {
        return Err(
            "a wheel notch back moved nothing at all, so a thousand remembered lines cannot be \
             reached. The design's own words: a terminal that remembers and cannot show has \
             forgotten, as far as a person is concerned."
                .into(),
        );
    }

    let mut forward = 0;
    let mut here = back;
    loop {
        a_notch(&mut server, Back::No, 900 + forward)?;
        let now = drawn(&mut nested, &server, &mut labels)?;
        if now == here {
            break;
        }
        here = now;
        forward += 1;
        if forward > AT_MOST_NOTCHES {
            return Err("the transcript moved forward without reaching the newest line".into());
        }
    }
    let back_at_the_newest = here;
    if back_at_the_newest != at_the_newest {
        return Err(format!(
            "scrolling back {notches} notches and forward {forward} did not return the \
             transcript to where it started, so one end is not where the other thinks it is"
        )
        .into());
    }

    // **Walk step 8: a clock, which is none of the five answers.** `sleep 30`
    // waits on a timer rather than on the keyboard, the network, another
    // process or the processor, and the walk's pass is the honest *waiting for
    // something alo cannot identify* rather than a confident wrong one. What is
    // asserted here is only that **an answer came back and reached the screen**
    // — which of the five it is belongs to `alo-terminal`'s own tests, and
    // asserting it from pixels would be this fixture reading English out of
    // glyphs.
    let at = typing(&mut server, "sleep 30", 300)?;
    typed(&mut server, THE_ENTER_KEY, at + 2)?;
    until_the_shell_is_quiet(&mut server);
    std::thread::sleep(LONG_ENOUGH_TO_START);
    let _ = server.the_terminal_was_written_to();

    key(&mut server, LEFT_CONTROL, 400)?;
    key(&mut server, LEFT_ALT, 401)?;
    key(&mut server, THE_W_KEY, 402)?;
    server.keyboard_key(THE_W_KEY, KeyState::Released, 403)?;
    server.keyboard_key(LEFT_ALT, KeyState::Released, 404)?;
    server.keyboard_key(LEFT_CONTROL, KeyState::Released, 405)?;
    let _ = server.the_terminal_was_written_to();

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
    let about_a_clock = frame_just_drawn(&nested, "the terminal with a clock waiting")?;

    // **`Ctrl`+`C` first, because `sleep 30` is still in front.** The step above
    // started it and never ended it, so a command typed now goes to *sleep*'s
    // standard input and the shell never sees it.
    //
    // *Found by looking at the frame rather than by counting rows.* The first
    // version of the step below typed four commands into a blocked terminal,
    // drew nothing, and failed with a message offering two causes — neither of
    // which was the one. The screen showed the commands sitting unrun under a
    // `$ sleep 30` prompt.
    key(&mut server, LEFT_CONTROL, 560)?;
    key(&mut server, THE_C_KEY, 561)?;
    server.keyboard_key(THE_C_KEY, KeyState::Released, 562)?;
    server.keyboard_key(LEFT_CONTROL, KeyState::Released, 563)?;
    until_the_shell_writes(&mut server);
    until_the_shell_is_quiet(&mut server);

    // **`clear` first, so the chart has room.** alo's own lines are drawn
    // beneath the grid, and a benchmark is seven of them — a summary, a
    // provenance, two rows of two lines each, and the scale note. On a screen
    // already full of output the later rows are clipped at the transcript's
    // foot, which is how the first run of this step drew one bar and not two.
    //
    // *That clipping is real and is not fixed here*: where alo's lines go when
    // they do not fit is a layout question the design does not answer, and it
    // is recorded in `docs/design/the-terminal-window.md` rather than invented.
    let at = typing(&mut server, "clear", 570)?;
    typed(&mut server, THE_ENTER_KEY, at + 2)?;
    until_the_shell_writes(&mut server);
    until_the_shell_is_quiet(&mut server);
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
    let before_the_benchmark = frame_just_drawn(&nested, "the terminal with a cleared screen")?;

    // **A benchmark comparison, recognised and drawn.** Four lines in the shape
    // `hyperfine` prints, typed into a real shell, so what is measured is the
    // whole road: the program's bytes, the recogniser, the scale and the bars.
    //
    // *Nothing here asserts the bar lengths.* `alo_terminal::measures` holds
    // those against the design's own figures, and a pixel count on a drawn
    // frame cannot tell 480 from 479. What this asks is the question a unit
    // test cannot: **did anything reach the screen at all.**
    let at = typing(
        &mut server,
        "echo 'Benchmark 1: previous'; echo '  Time (mean): 240.0 ms'; echo 'Benchmark 2: \
         current'; echo '  Time (mean): 180.0 ms'",
        600,
    )?;
    typed(&mut server, THE_ENTER_KEY, at + 2)?;
    until_the_shell_writes(&mut server);
    until_the_shell_is_quiet(&mut server);
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
    let a_benchmark = frame_just_drawn(&nested, "the terminal with a recognised benchmark")?;

    // **A file diff, recognised and drawn.** Printed rather than produced by
    // `git`, so the step measures the recogniser and the drawing rather than
    // whatever `git` happens to be installed here — and so it reads the same
    // on a machine with no repository in front of it.
    let at = typing(&mut server, "clear", 630)?;
    typed(&mut server, THE_ENTER_KEY, at + 2)?;
    until_the_shell_writes(&mut server);
    until_the_shell_is_quiet(&mut server);
    let at = typing(
        &mut server,
        "echo '+++ b/src/auth.ts'; echo '@@ -12,4 +12,4 @@'; echo ' export async function \
         signIn() {'; echo '   const response = await login();'; echo '-  return \
         response.user.name;'; echo '+  return response.user name;'; echo ' }'",
        660,
    )?;
    typed(&mut server, THE_ENTER_KEY, at + 2)?;
    until_the_shell_writes(&mut server);
    until_the_shell_is_quiet(&mut server);
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
    let a_diff = frame_just_drawn(&nested, "the terminal with a recognised diff")?;

    // **And alo's explanation of a command, drawn.** ADR 0124 term 4 and the
    // owner's decision of 2026-10-18: a person asks what a command will do
    // before running it, and the answer comes through the agent.
    //
    // **What this drives, and what it cannot.** `what_came_back` is the real
    // parser and `Server::alo_explains` is the real road to the grid, so the
    // reply's shape, its three marks and the drawing are all exercised. **The
    // socket round trip to `alo-agentd` is not**: there is no daemon in this
    // fixture and no model behind it, so the bytes below stand in for what one
    // sends back. Said here rather than left to be assumed — the step this
    // closes is *nothing had ever drawn an explanation*, not *the agent
    // answered*.
    //
    // The reply is written the way the request asks for it, `MARKS:` first, and
    // carries two of the three marks so that a frame with marks is distinct
    // from one without.
    //
    // **Two explanations are drawn, a short one and a long one, and the long
    // one must take more rows.** That comparison is the whole point of the
    // step, because it is the one thing truncation and wrapping answer
    // differently: a clipped line and a wrapped line both put ink on the
    // screen, and only the wrapped one puts it on a second row.
    //
    // Until 2026-10-22 alo's lines were laid out unbounded and cropped, so a
    // long explanation was cut off **mid-word with its tail lost**. Measured
    // here at 1440x960: 137 characters drew whole, and 286 drew as
    // `...so nothing can be restored afterwar` and stopped, losing *it can
    // reach files that are not yours*, which is the half a person deciding
    // whether to press Enter most needs. With `Wrap::None` restored this step
    // fails, which is the only reason to trust it.
    let at = typing(&mut server, "clear", 800)?;
    typed(&mut server, THE_ENTER_KEY, at + 2)?;
    until_the_shell_writes(&mut server);
    until_the_shell_is_quiet(&mut server);
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
    let before_the_explanation = frame_just_drawn(&nested, "the terminal before an explanation")?;

    let reply = "MARKS: destructive, elevated\nIt deletes the whole of /important and everything under it, and asks for administrator rights to do it. Nothing is copied anywhere first.";
    let Some(explained) = alo_shell::what_alo_asks_about_a_command::what_came_back(reply) else {
        return Err(
            "the reply the agent would send back parsed to no explanation at all, so either `what_came_back` stopped reading the shape the request asks for or the reply above no longer matches it. Nothing was drawn and nothing could be."
                .into(),
        );
    };
    server.alo_explains(explained);
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
    let an_explanation = frame_just_drawn(&nested, "the terminal with alo's explanation")?;

    // **Rows of text, against the frame immediately before.** The same rule the
    // benchmark step learned the hard way: comparing against an older screen
    // makes `clear` look like a loss.
    if rows_with_ink(&an_explanation, &without) <= rows_with_ink(&before_the_explanation, &without)
    {
        return Err(
            "alo's explanation of a command added no row of text. Either `alo_explains` reached no terminal — it does nothing when there is none — or `WhatAloSaid::explains` drew nothing for a `WhatACommandWillDo` that parsed correctly. Write the frame out with ALO_TERMINAL_CHECK_WRITE and look before believing either."
                .into(),
        );
    }
    if let Ok(into) = std::env::var("ALO_TERMINAL_CHECK_WRITE") {
        written_out(
            &an_explanation,
            &format!("{into}.explained.ppm"),
            "the frame with alo's explanation of a command",
        )?;
    }
    // **And now the pair that answers whether it wraps**, which the step above
    // cannot: it carries two marks, and alo's block has only four rows between
    // the grid's last line and the transcript's foot, so with two marks and a
    // disclaimer there is exactly **one** row left for the prose. A long
    // explanation there cannot wrap however well the wrapping works, and asking
    // it to would measure the room rather than the wrapping.
    //
    // So both of these carry **no** marks — which leaves three rows for the
    // prose — and differ **only** in how long the sentence is. That is the
    // whole control: a first version varied the marks as well as the length and
    // compared 6 rows against 7, because removing two marks removes two rows.
    // It failed, correctly, and the figure it printed is what showed the
    // comparison rather than the product was wrong.
    //
    // *That alo's block can want more rows than it has is a separate gap, and
    // `docs/autonomy/the-terminal-in-full-plan.md` carries it: giving it more
    // rows means taking them from the grid, which resizes the person's shell,
    // and `docs/design/the-terminal-window.md` leaves this family of overflow
    // open on purpose.*
    // **And the pair that answers whether the block grows upward**, which is the
    // owner's decision of 2026-10-22 on where alo's room comes from.
    //
    // Both carry **two marks**, so three of alo's four rows beneath the
    // transcript are spoken for and exactly one is left for the prose. Under the
    // old rule a long explanation there was capped at that one row however well
    // it wrapped, so both of these took the same number of rows. A block that
    // grows over the transcript above it takes more for the longer one, and that
    // difference is the whole of what this step asks.
    let marked_briefly = "MARKS: destructive, elevated\nIt deletes /important.";
    let marked_at_length = "MARKS: destructive, elevated\nIt removes the directory /important \
                            and every file beneath it, without moving anything to a wastebasket \
                            first, so nothing can be restored afterwards by ordinary means. It \
                            also asks for administrator rights, which means it can reach files \
                            that are not yours as well as the ones that are.";

    let briefly = "MARKS: none\nIt deletes /important and everything under it.";
    let at_length = "MARKS: none\nIt removes the directory /important and every file beneath \
                     it, without moving anything to a wastebasket first, so nothing can be \
                     restored afterwards by ordinary means. It also asks for administrator \
                     rights, which means it can reach files that are not yours as well as the \
                     ones that are.";

    let rows_for = |reply: &str,
                    moment: &'static str,
                    server: &mut Server,
                    nested: &mut Nested,
                    labels: &mut WindowControlLabels|
     -> Result<usize, Box<dyn std::error::Error>> {
        let Some(explained) = alo_shell::what_alo_asks_about_a_command::what_came_back(reply)
        else {
            return Err(format!("{moment}: the reply parsed to no explanation at all").into());
        };
        server.alo_explains(explained);
        nested.pump()?;
        nested.submit_with_desktop(
            &[],
            &[],
            &alo_shell::Cursor::Default,
            &[],
            labels,
            desktop(),
            None,
            None,
            Some(TheTerminalToDraw {
                server,
                display: THE_DISPLAY,
            }),
        )?;
        let frame = frame_just_drawn(nested, moment)?;
        if let Ok(into) = std::env::var("ALO_TERMINAL_CHECK_WRITE") {
            written_out(&frame, &format!("{into}.{moment}.ppm"), moment)?;
        }
        Ok(rows_with_ink(&frame, &without))
    };

    let marked_brief_rows = rows_for(
        marked_briefly,
        "marked-briefly",
        &mut server,
        &mut nested,
        &mut labels,
    )?;
    let marked_long_rows = rows_for(
        marked_at_length,
        "marked-at-length",
        &mut server,
        &mut nested,
        &mut labels,
    )?;
    if marked_long_rows <= marked_brief_rows {
        return Err(format!(
            "a long explanation carrying two marks took no more rows than a short one \
             ({marked_long_rows} against {marked_brief_rows}), so alo's block did not grow over \
             the transcript and the prose was capped at the one row the marks left it. The block \
             is placed by `starts_at` in `the_terminals_cells_are_drawn`: it is measured whole \
             and its foot kept, so a block too tall to sit beneath the transcript begins further \
             up instead of being cut."
        )
        .into());
    }

    let briefly_rows = rows_for(
        briefly,
        "explained-briefly",
        &mut server,
        &mut nested,
        &mut labels,
    )?;
    let at_length_rows = rows_for(
        at_length,
        "explained-at-length",
        &mut server,
        &mut nested,
        &mut labels,
    )?;

    // **The one thing truncation and wrapping answer differently.** Both replies
    // carry the same marks — none — and the same disclaimer, so every row but
    // the prose's is identical between the two frames. A clipped line and a
    // wrapped line both put ink on the screen; only the wrapped one puts it on a
    // second row.
    if at_length_rows <= briefly_rows {
        return Err(format!(
            "a 286-character explanation with three rows to wrap into took no more rows than a \
             58-character one ({at_length_rows} against {briefly_rows}), so it was cut off \
             rather than wrapped and a person never sees its tail. alo's lines are drawn by \
             `shaped_into_alos_row`; a `Wrap::None` there lays the sentence out unbounded and \
             `inked` crops whatever overruns the room the transcript leaves."
        )
        .into());
    }
    if let Ok(into) = std::env::var("ALO_TERMINAL_CHECK_WRITE") {
        written_out(
            &a_diff,
            &format!("{into}.diff.ppm"),
            "the frame with a recognised diff",
        )?;
    }
    if let Ok(into) = std::env::var("ALO_TERMINAL_CHECK_WRITE") {
        written_out(
            &a_benchmark,
            &format!("{into}.benchmark.ppm"),
            "the frame with a recognised benchmark",
        )?;
    }
    // **Against the frame just before it**, not against the clock's. The first
    // version compared with a screen that still held forty lines of `seq`
    // output, so clearing the screen made the count go *down* and the step
    // failed while the chart was on the screen behind it.
    if rows_with_ink(&a_benchmark, &without) <= rows_with_ink(&before_the_benchmark, &without) {
        return Err(
            "four lines of benchmark output added no row of text. Three things do that and \
             the message used to name only two: nothing was recognised in them, nothing was \
             drawn for what was recognised, or **the commands never ran at all** because \
             something was still in front of the shell. Write the frame out with \
             ALO_TERMINAL_CHECK_WRITE and look before believing either of the first two."
                .into(),
        );
    }
    if rows_with_ink(&about_a_clock, &without) <= rows_with_ink(&back_at_the_newest, &without) {
        return Err(
            "asking why a clock is waiting added no row of text, so alo answered nothing where \
             the walk expects it to say plainly that it cannot tell. A command this machine \
             cannot classify is the case the fifth answer exists for; silence is not it."
                .into(),
        );
    }

    // **Written out only when somebody asks**, the way `saving_frames` guards
    // itself: a gate that wrote a file every run would be a gate that fills a
    // disk, and nothing here is the screenshot promise.
    if let Ok(into) = std::env::var("ALO_TERMINAL_CHECK_WRITE") {
        written_out(&about_a_clock, &into, "the frame with the terminal")?;
        // **The frame step 10 is about**, written beside the other one because
        // the assertion above it counts rows of ink and cannot read a word. The
        // walk's pass for that step is two particular sentences, and the only
        // way anybody has confirmed which sentences are drawn is by looking.
        written_out(
            &recognised,
            &format!("{into}.recognised.ppm"),
            "the frame with a recognised test result",
        )?;
        // **Step 6's frame, which is the one the boot exists for.** The lane
        // order: *one boot, one terminal, one `cat`, one chord — and either it
        // says `waiting for the keyboard` on a screen or it does not.* That
        // sentence is a person reading words, so the frame holding them is
        // written out beside the other two.
        written_out(
            &answered,
            &format!("{into}.answered.ppm"),
            "the frame with alo's answer to why a command is waiting",
        )?;
        written_out(
            &a_benchmark,
            &format!("{into}.benchmark.ppm"),
            "the frame with a recognised benchmark",
        )?;
    }

    eprintln!(
        "terminal: Ctrl+Alt+T opened one and it reached the frame — ink across a {wide}x{tall} \
         extent from ({left}, {top}), {how_many} pixels differing from the same desktop drawn \
         twice identically without it; then `echo hello` and Enter changed {after_typing} more \
         pixels, so keys reached the shell and its answer reached the screen; then `cat` and \
         Ctrl+Alt+W drew alo's own answer to *why is it waiting*, {said} pixels of ink in all"
    );
    Ok(())
}
