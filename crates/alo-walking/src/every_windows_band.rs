//! **Every kind of window's band, pointed at, and each of its controls
//! pressed** — the walk the window-controls work is measured against.
//!
//! The investigation of 2026-11-21 found the walk never pointed at or clicked
//! a window control: it closed everything with Alt+F4, and the journal said
//! *8 key presses and 0 button presses*. So a band that was not on the screen
//! at all, as Text Editor's was, was never seen. For each kind of window this
//! keeps a picture of it at rest, points at its band where the shell says it is
//! drawn (`alo_acting::said_of_a_band`), keeps a picture of the controls
//! revealed, reads which controls are drawn, and presses each in the middle of
//! where it is drawn — then waits for what the press did:
//!
//! | section | window | opened by | controls pressed |
//! |---|---|---|---|
//! | `application` | Dolphin, whose band the shell draws | the launcher, Super+Space | Window actions, Close, Minimise |
//! | `application-own-header` | Text Editor | the launcher | Window actions |
//! | `terminal` | the terminal | Ctrl+Alt+T | Window actions, Close, Minimise |
//! | `settings` | Settings | Super+I | Close, Minimise |
//! | `record` | the record | Super+R | Close, Minimise |
//! | `task` | a task window | Super+A and a question | Window actions, Close, Minimise |
//!
//! Settings and the record have no Window actions: their pages lay out at one
//! size, so nothing in the panel could place them (the window-controls lane,
//! 2026-11-21). Text Editor draws its own Minimise and Close.
//!
//! **A section for each kind**, so one kind's band failing does not hide the
//! others; and **first among the sections**, before the providers, so no
//! failure of the long sections after can leave them not reached. They need no
//! provider: the task window's question is answered by nothing, and the window
//! is there all the same.
//!
//! **Two pictures of each kind by a fixed name**, for the owner's before and
//! after (the window-controls lane, 2026-11-21): `band-<kind>-at-rest`, the
//! pointer away from the band, and `band-<kind>-revealed`, the pointer in it
//! before anything is pressed. The first is kept whatever the shell says; the
//! second only once the shell says where the band is drawn.

use std::time::Duration;

use alo_acting::said_of_a_band::{self as band, TheKind};

use crate::a_place_on_the_screen::Target;
use crate::keys::Chord;
use crate::settling::Settling;
use crate::step::{Pointing, Since, Walk};

/// How long a band, its controls or what a press did is waited for.
const WITHIN: Duration = Duration::from_secs(60);

/// The window as every line names it, to match a line by: its kind and id —
/// and for a task window only `task "task `, its number being a place in a
/// list that moves.
fn the_window(kind: TheKind, id: &str) -> String {
    if kind == TheKind::Task {
        format!("{} \"task ", kind.word())
    } else {
        format!("{} {id:?}", kind.word())
    }
}

/// One kind of window, as this section walks it.
struct AKind {
    /// The kind's name in its section and pictures.
    label: &'static str,
    /// Which kind the lines name.
    kind: TheKind,
    /// Its id, as the lines say it.
    id: &'static str,
    /// The controls it has, every one.
    controls: &'static [&'static str],
    /// How a person opens it.
    open: fn(Walk, &str) -> Walk,
}

/// The kinds, in the order walked.
const THE_KINDS: [AKind; 6] = [
    AKind {
        label: "application",
        kind: TheKind::Application,
        id: "org.kde.dolphin",
        controls: &[band::WINDOW_ACTIONS, band::MINIMISE, band::CLOSE],
        open: dolphin,
    },
    AKind {
        label: "application-own-header",
        kind: TheKind::Application,
        id: "org.gnome.TextEditor",
        controls: &[band::WINDOW_ACTIONS],
        open: text_editor,
    },
    AKind {
        label: "terminal",
        kind: TheKind::Terminal,
        id: "terminal",
        controls: &[band::WINDOW_ACTIONS, band::MINIMISE, band::CLOSE],
        open: the_terminal,
    },
    AKind {
        label: "settings",
        kind: TheKind::Settings,
        id: "settings",
        controls: &[band::MINIMISE, band::CLOSE],
        open: settings,
    },
    AKind {
        label: "record",
        kind: TheKind::Record,
        id: "record",
        controls: &[band::MINIMISE, band::CLOSE],
        open: the_record,
    },
    AKind {
        label: "task",
        kind: TheKind::Task,
        id: "task",
        controls: &[band::WINDOW_ACTIONS, band::MINIMISE, band::CLOSE],
        open: a_task,
    },
];

/// The name of the section for `kind`.
#[must_use]
pub fn the_section_for(label: &str) -> String {
    format!("the band and controls of {label}")
}

impl AKind {
    /// Point at the band, keep `picture`, read the controls, and press `name`:
    /// the press is said by the pointer, and then `then` is said.
    fn pressing(&self, walk: Walk, picture: &str, name: &str, then: &str) -> Walk {
        let look = Settling::UNDER_KVM;
        let what = name.to_lowercase().replace(' ', "-");
        let window = the_window(self.kind, self.id);
        walk.point_at_the_band(self.kind, self.id, WITHIN)
            .look(picture, look)
            .the_controls_are(self.kind, self.id, self.controls, WITHIN)
            .mark()
            .click_the_control(name)
            .expect_line_saying(
                &format!("{name} on {window} is pressed by the pointer"),
                &format!("{} {window}", band::A_BANDS_CONTROL_WAS_PRESSED),
                &format!("{name:?} by pointer"),
                Since::Mark,
                WITHIN,
            )
            .expect_line(
                &format!("{name} did what it says on {window}"),
                &format!("{then} {window}"),
                Since::Mark,
                WITHIN,
            )
            .look(&format!("band-{}-after-{what}", self.label), look)
    }

    /// Opened, kept at rest with the pointer away, then Window actions where
    /// it has it (the panel opens, and Escape closes it), then Close (it
    /// closes), then — opened again — Minimise (it goes aside). The first
    /// pointing keeps the picture of the controls revealed.
    fn every_control(&self, walk: Walk) -> Walk {
        let look = Settling::UNDER_KVM;
        let label = self.label;
        let walk = (self.open)(walk.press(Chord::key("esc")).mark(), label)
            .pointer(Pointing::PointAt {
                check: format!("the pointer is away from the band of {label}"),
                at: Target::Middle,
            })
            .look_as_it_settles(&format!("band-{label}-at-rest"), look);
        let mut pictures = std::iter::once(format!("band-{label}-revealed")).chain(
            ["for-close", "for-minimise"]
                .iter()
                .map(|what| format!("band-{label}-pointed-{what}")),
        );
        let mut next = || pictures.next().unwrap_or_default();
        let mut walk = walk;
        if self.controls.contains(&band::WINDOW_ACTIONS) {
            walk = self
                .pressing(
                    walk,
                    &next(),
                    band::WINDOW_ACTIONS,
                    band::THE_WINDOW_PANEL_OPENED_FOR,
                )
                .press(Chord::key("esc"))
                .look(&format!("band-{label}-the-panel-closed"), look);
        }
        if self.controls.contains(&band::CLOSE) {
            walk = self.pressing(walk, &next(), band::CLOSE, band::A_WINDOW_WAS_CLOSED);
        }
        if self.controls.contains(&band::MINIMISE) {
            let opened = (self.open)(walk.mark(), &format!("{label}-again"));
            walk = self.pressing(
                opened,
                &next(),
                band::MINIMISE,
                band::A_WINDOW_WAS_PUT_ASIDE,
            );
        }
        // **Text Editor draws its own Close**: Alt+F4 closes it after, and it
        // holds no unsaved work.
        if !self.controls.contains(&band::CLOSE) {
            walk = walk.press(Chord::with("alt", "f4"));
        }
        walk
    }
}

/// Open an application by its name, the way a person does: the launcher, its
/// name typed, Enter.
///
/// **The launcher's look may find the screen as it was**: Super+Space puts a
/// caret in the Dock's field and changes nothing else (run 44900882154,
/// picture 49). The band's line that follows proves the application opened.
fn by_the_launcher(walk: Walk, picture: &str, name: &str) -> Walk {
    walk.press(Chord::with("meta_l", "spc"))
        .look_as_it_settles(
            &format!("band-{picture}-the-launcher"),
            Settling::UNDER_KVM.at_most(Duration::from_secs(60)),
        )
        .type_text(name)
        .press(Chord::key("ret"))
}

/// Dolphin, by the launcher.
fn dolphin(walk: Walk, picture: &str) -> Walk {
    by_the_launcher(walk, picture, "Dolphin")
}

/// Text Editor, by the launcher.
fn text_editor(walk: Walk, picture: &str) -> Walk {
    by_the_launcher(walk, picture, "Text Editor")
}

/// The terminal, by its chord.
fn the_terminal(walk: Walk, _: &str) -> Walk {
    walk.press(Chord::holding(&["ctrl", "alt"], "t"))
}

/// Settings, by its chord.
fn settings(walk: Walk, _: &str) -> Walk {
    walk.press(Chord::with("meta_l", "i"))
}

/// The record, by its chord.
fn the_record(walk: Walk, _: &str) -> Walk {
    walk.press(Chord::with("meta_l", "r"))
}

/// A task window: alo asked something, which nothing answers before a provider
/// is added — and the window is there all the same.
fn a_task(walk: Walk, picture: &str) -> Walk {
    walk.press(Chord::with("meta_l", "a"))
        .look(
            &format!("band-{picture}-the-bar"),
            Settling::UNDER_KVM.at_most(Duration::from_secs(60)),
        )
        .type_text("Hello.")
        .press(Chord::key("ret"))
}

/// Then a section for each kind of window's band and controls.
#[must_use]
pub fn every_windows_band(walk: Walk) -> Walk {
    THE_KINDS.iter().fold(walk, |walk, kind| {
        walk.section(&the_section_for(kind.label), &[], |walk| {
            kind.every_control(walk)
        })
    })
}
