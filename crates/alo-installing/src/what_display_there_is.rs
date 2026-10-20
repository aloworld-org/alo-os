//! What display the firmware left, read from what the kernel already says.
//!
//! The boot environment has screens of its own
//! ([ADR 0128](../../../docs/decisions/0128-whether-a-wi-fi-password-may-be-written-to-the-staging-partition.md)),
//! and the desktop lane lays them out as **proportions of the display** rather
//! than in logical pixels. So the first thing anything here needs is how large
//! the display is — and nothing in this environment knew.
//!
//! # No ioctls, because none are needed for this
//!
//! A `drm_mode_get_connector` would answer it, and that is a hand-transcribed
//! kernel struct with an ABI to get wrong, in a program that is statically
//! linked against musl and has no `libdrm` to borrow one from. The kernel
//! already publishes all three answers as files:
//!
//! | | |
//! |---|---|
//! | is anything plugged in | `/sys/class/drm/<card>-<connector>/status` — `connected` |
//! | what mode it is in | `…/modes` — one per line, **the preferred one first** |
//! | how large the glass is | `…/edid` — bytes 21 and 22, in whole centimetres |
//!
//! So this file is three parsers and no `unsafe`, which is the shape a test can
//! hold. Drawing *into* that mode needs ioctls and is the next item; asking
//! about it does not.
//!
//! # Where the millimetres go, and where they must not
//!
//! The desktop lane takes them **as a legibility check and never as a layout
//! driver** — with a physical size known it can say *this body text is 2.2 mm
//! tall* and raise it; with none, the proportion stands on its own. That is why
//! [`Glass`] is an `Option` all the way out rather than a zero: a screen that
//! reports nothing must not arrive as a number something divides by.
//!
//! `alo_displays::Scale::worked_out_for` takes exactly these two values and is
//! **deliberately not called here.** Its arithmetic is right for a desktop,
//! where a person can change the answer afterwards; the environment has no
//! Settings and no next boot, and a panel that reports no size would be drawn
//! at 100% with no remedy. Proportions do not ask the question, so they cannot
//! be wrong about it.
//!
//! # What is measured and what is not
//!
//! **The parsers are tested; the files have not been read on a machine.** This
//! build machine has no DRM device at all — no `/dev/dri`, no card, no driver —
//! so the fixtures below are **constructed from the EDID 1.4 specification and
//! from the kernel's documented sysfs layout, not captured from hardware.** A
//! real `edid` and `modes` from the certified machine is the measurement this
//! file is owed, and until it arrives the parsers are a careful reading of a
//! specification rather than of a screen.

/// How large a screen's glass is, in whole millimetres.
///
/// Whole millimetres because EDID carries whole **centimetres** in the two
/// bytes this reads, so a tenth of a millimetre would be precision nobody
/// measured.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Glass {
    /// Across.
    across: u32,
    /// Down.
    down: u32,
}

impl Glass {
    /// A glass of this size, in millimetres.
    ///
    /// **For a caller that already has the measurement**, which today is a
    /// test: the one road a machine takes is [`the_glass`], which reads it out
    /// of what the display reported. Nought is accepted rather than refused
    /// because nought is a thing displays really say — a virtual output, and
    /// some real panels — and whoever reads it has to decide what to do about
    /// that rather than be handed a [`None`] that hides which of the two it
    /// was.
    #[must_use]
    pub const fn of(across: u32, down: u32) -> Self {
        Self { across, down }
    }

    /// Across, in millimetres.
    #[must_use]
    pub const fn across(self) -> u32 {
        self.across
    }

    /// Down, in millimetres.
    #[must_use]
    pub const fn down(self) -> u32 {
        self.down
    }
}

/// What display the firmware left.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WhatDisplayThereIs {
    /// Pixels across.
    across: u32,
    /// Pixels down.
    down: u32,
    /// How large the glass is, where the screen said.
    glass: Option<Glass>,
}

impl WhatDisplayThereIs {
    /// Pixels across.
    #[must_use]
    pub const fn across(self) -> u32 {
        self.across
    }

    /// Pixels down.
    #[must_use]
    pub const fn down(self) -> u32 {
        self.down
    }

    /// How large the glass is, where the screen said so.
    ///
    /// [`None`] is ordinary rather than exceptional: a virtual output reports
    /// nothing, and some panels report zero. Whoever draws takes this as a
    /// legibility check and never as a layout driver.
    #[must_use]
    pub const fn glass(self) -> Option<Glass> {
        self.glass
    }
}

/// Whether this connector has something plugged into it.
///
/// The kernel writes exactly `connected`, `disconnected` or `unknown`, each
/// with a newline. `unknown` is **not** connected: it is what a connector that
/// could not be probed says, and drawing onto a screen nobody could probe is
/// not something to do on a guess.
#[must_use]
pub fn is_connected(status: &str) -> bool {
    status.trim() == "connected"
}

/// The mode a connector is in, from the kernel's own list.
///
/// **The first line, because the kernel puts the preferred mode first** — and
/// on a machine the firmware has already set a mode on, the preferred one is
/// the one on the screen. A line that is not `<across>x<down>` is not a mode;
/// the list also carries interlaced and doublescan suffixes on some drivers,
/// so a line is read up to the first character that is not a digit or an `x`.
#[must_use]
pub fn the_mode(modes: &str) -> Option<(u32, u32)> {
    let line = modes.lines().map(str::trim).find(|line| !line.is_empty())?;
    let kept: String = line
        .chars()
        .take_while(|at| at.is_ascii_digit() || *at == 'x')
        .collect();
    let (across, down) = kept.split_once('x')?;
    Some((across.parse().ok()?, down.parse().ok()?))
}

/// Where EDID carries the screen's size, in centimetres.
const ACROSS_IN_CENTIMETRES: usize = 21;

/// And the other one.
const DOWN_IN_CENTIMETRES: usize = 22;

/// What every EDID begins with, and the only thing that says these bytes are
/// one.
const EVERY_EDID_BEGINS: [u8; 8] = [0x00, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x00];

/// How large the glass is, from the screen's own EDID.
///
/// [`None`] rather than a guess in every case that is not an answer:
///
/// - **nothing plugged in.** The kernel leaves the file empty, so there are no
///   bytes at all;
/// - **not an EDID.** Without the header above, these bytes are something else
///   and reading byte 21 of something else is how a layout ends up derived from
///   a driver's debug blob;
/// - **the screen said zero**, which EDID defines as *undefined* — a projector,
///   or a panel whose maker left it out. Zero is the one value that must not
///   become a number, because it is the one something would divide by.
#[must_use]
pub fn the_glass(edid: &[u8]) -> Option<Glass> {
    if edid.len() <= DOWN_IN_CENTIMETRES {
        return None;
    }
    if edid.get(..EVERY_EDID_BEGINS.len()) != Some(&EVERY_EDID_BEGINS[..]) {
        return None;
    }
    let across = u32::from(*edid.get(ACROSS_IN_CENTIMETRES)?);
    let down = u32::from(*edid.get(DOWN_IN_CENTIMETRES)?);
    if across == 0 || down == 0 {
        return None;
    }
    Some(Glass {
        across: across * 10,
        down: down * 10,
    })
}

/// What display there is, from one connector's three files.
///
/// `edid` is taken as whatever was read, including nothing: a connector that is
/// connected but whose EDID could not be read is a display this can still draw
/// on, at a size nobody can check.
#[must_use]
pub fn from_a_connector(status: &str, modes: &str, edid: &[u8]) -> Option<WhatDisplayThereIs> {
    if !is_connected(status) {
        return None;
    }
    let (across, down) = the_mode(modes)?;
    Some(WhatDisplayThereIs {
        across,
        down,
        glass: the_glass(edid),
    })
}

#[cfg(test)]
#[expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None is the failure being reported"
)]
mod tests {
    use super::*;

    /// An EDID's first 23 bytes, with this size in centimetres.
    ///
    /// **Constructed from the EDID 1.4 specification, not captured from a
    /// screen** — this machine has no display to capture one from. Bytes 0–7
    /// are the header every EDID begins with, 8–20 are the maker and product
    /// fields this does not read, and 21 and 22 are the size.
    fn an_edid_saying(across: u8, down: u8) -> Vec<u8> {
        let mut bytes = EVERY_EDID_BEGINS.to_vec();
        bytes.extend_from_slice(&[0u8; 13]);
        bytes.push(across);
        bytes.push(down);
        bytes
    }

    /// **A connected screen in a mode, with a size.**
    #[test]
    fn a_connected_screen_is_read() {
        let found = from_a_connector(
            "connected\n",
            "1920x1080\n1680x1050\n1280x1024\n",
            &an_edid_saying(34, 19),
        )
        .expect("a connected screen in a mode is a display");
        assert_eq!((found.across(), found.down()), (1920, 1080));
        let glass = found.glass().expect("this screen reported its size");
        assert_eq!((glass.across(), glass.down()), (340, 190));
    }

    /// **The preferred mode is the first line**, not the largest and not the
    /// last.
    #[test]
    fn the_first_line_is_the_mode() {
        assert_eq!(the_mode("1366x768\n1920x1080\n"), Some((1366, 768)));
    }

    /// **A mode with a suffix is still a mode.** Some drivers write `i` for
    /// interlaced and `d` for doublescan after the numbers.
    #[test]
    fn a_mode_with_a_suffix_is_read() {
        assert_eq!(the_mode("1920x1080i\n"), Some((1920, 1080)));
        assert_eq!(the_mode("  2560x1440d  \n1920x1080\n"), Some((2560, 1440)));
    }

    /// **Nothing that is not a mode becomes one.**
    #[test]
    fn nothing_that_is_not_a_mode_becomes_one() {
        for not_a_mode in ["", "\n\n", "unknown\n", "x\n", "1920\n", "1920xabc\n"] {
            assert_eq!(the_mode(not_a_mode), None, "{not_a_mode:?}");
        }
    }

    /// **Only `connected` is connected**, and `unknown` is not.
    ///
    /// A connector the kernel could not probe says `unknown`, and drawing onto
    /// a screen nobody could probe is not a thing to do on a guess.
    #[test]
    fn only_connected_is_connected() {
        assert!(is_connected("connected\n"));
        assert!(is_connected("connected"));
        for not_connected in ["disconnected\n", "unknown\n", "", "Connected\n"] {
            assert!(!is_connected(not_connected), "{not_connected:?}");
        }
    }

    /// **A size of zero is not a size**, which is EDID's own meaning for it and
    /// the one value something would divide by.
    #[test]
    fn a_size_of_zero_is_not_a_size() {
        assert_eq!(the_glass(&an_edid_saying(0, 0)), None);
        assert_eq!(the_glass(&an_edid_saying(34, 0)), None);
        assert_eq!(the_glass(&an_edid_saying(0, 19)), None);
    }

    /// **Bytes that are not an EDID are not read as one.**
    ///
    /// Without the header, byte 21 of something else is how a layout ends up
    /// derived from a driver's debug blob. The control here is the second case:
    /// the same bytes **with** the header do answer, so the refusal is about
    /// the header and not about the length.
    #[test]
    fn bytes_that_are_not_an_edid_are_refused() {
        let mut wrong = an_edid_saying(34, 19);
        if let Some(first) = wrong.first_mut() {
            *first = 0x01;
        }
        assert_eq!(the_glass(&wrong), None);
        assert!(the_glass(&an_edid_saying(34, 19)).is_some(), "the control");
    }

    /// **A short read is not a size.** A connector with nothing plugged in
    /// leaves the file empty.
    #[test]
    fn a_short_read_is_not_a_size() {
        for len in 0..=DOWN_IN_CENTIMETRES {
            let mut short = an_edid_saying(34, 19);
            short.truncate(len);
            assert_eq!(the_glass(&short), None, "{len} bytes");
        }
        assert!(
            the_glass(&an_edid_saying(34, 19)).is_some(),
            "the control: the whole thing answers"
        );
    }

    /// **A screen that is connected and reports no size is still a display.**
    ///
    /// This is the case the whole proportion decision rests on: it draws, and
    /// the millimetres are absent rather than zero.
    #[test]
    fn a_screen_with_no_size_is_still_a_display() {
        let found = from_a_connector("connected\n", "3840x2160\n", &[])
            .expect("a connected screen in a mode is a display whatever it says about its size");
        assert_eq!((found.across(), found.down()), (3840, 2160));
        assert_eq!(found.glass(), None);
    }

    /// **Nothing plugged in is not a display**, however good its other files
    /// look.
    #[test]
    fn nothing_plugged_in_is_not_a_display() {
        assert_eq!(
            from_a_connector("disconnected\n", "1920x1080\n", &an_edid_saying(34, 19)),
            None
        );
    }

    /// **A connector with no mode is not a display either.**
    #[test]
    fn a_connector_with_no_mode_is_not_a_display() {
        assert_eq!(
            from_a_connector("connected\n", "", &an_edid_saying(34, 19)),
            None
        );
    }
}
