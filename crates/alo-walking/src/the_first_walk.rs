//! **The first walk**: a fresh disk from its first screen to the desktop, the
//! applications it installs on first boot, the screen reader switched on in
//! Settings, and then the pointer (`crate::the_pointer_walk`) — each check with
//! its evidence.
//!
//! Every key below was read from the code it reaches, not remembered:
//!
//! - **The first-start overview**: Tab moves the ring between the list and
//!   Continue, and Enter on Continue accepts what the overview names
//!   (`alo_shell::the_first_start`). A machine whose clock names no region is
//!   read as the United States, which has several zones, so Continue first
//!   asks which; Escape leaves that sheet setting none, and the next Continue
//!   goes on (`continue_accepts_the_language_region_and_zone_the_overview_names`).
//! - **The first account**: Enter on the name goes on to the PIN; Tab three
//!   times reaches *Use password only*; the password is typed twice, Tab
//!   between, and Enter makes the account (`the_first_account_tests.rs`, and
//!   the walk by hand of 2026-11-14).
//! - **The session opens when the account is made**, before *How much can
//!   alo do*, which is drawn inside it: measured on the first run on GitHub,
//!   2026-11-14, `alo-sessiond: a desktop is running` at 52 s and the question
//!   answered at 56 s. So the session's sentence is checked after the
//!   password, and the question's own (`how much can alo do: …`) after it.
//! - **How much can alo do**: Tab, then Enter, as the walk by hand did.
//! - **Settings**: Super+I opens it; End is Accessibility in the side column;
//!   Tab enters the page at its first row, which is *Screen reader*, first on
//!   the page on purpose; Space turns it over (`alo_shell::settings_window`,
//!   `settings_page_accessibility`).
//!
//! - **The terminal at rest**: Escape closes Settings; Ctrl+Alt+T opens the
//!   terminal (`alo_shortcuts`' defaults), and its band must draw no controls
//!   while the pointer is elsewhere.
//! - **The terminal maximised**: Alt+F3 opens its Window actions panel on
//!   Left (`alo_shell`'s `a_chord_places_alos_own_window`), Right moves to
//!   Maximise, Enter carries it out, and the shell says it fills the work area.
//!
//! **The provider section** types an AI provider's key from
//! [`THE_PROVIDER_KEY`], and runs only when that is set: on every walk after
//! a merge to main, and by hand when the box is ticked. Its keys are the walk
//! by hand's of 2026-11-14 (`w28-provider.sh`); from where this walk leaves
//! Settings they were seen to reach *Add a provider* and save `openai` on
//! run 43635265001 (2026-11-14), in its pictures.
//! **Since the terminal steps it starts from the terminal instead**: Escape
//! there goes to the shell, and Super+I opens Settings again. Not yet seen on a
//! run; the provider's first picture shows where it began.

use std::time::Duration;

use crate::keys::Chord;
use crate::settling::Settling;
use crate::step::{Since, Walk};

/// The environment variable the provider's key is read from — an Actions
/// secret of the same name, when the owner adds one.
pub const THE_PROVIDER_KEY: &str = "ALO_WALK_PROVIDER_KEY";

/// The name the walk gives its account.
const NAME: &str = "walker";

/// The password it gives that account. A walk machine is thrown away with
/// its disk, so this is not a secret.
const PASSWORD: &str = "walkpass1";

/// The first walk.
#[must_use]
pub fn the_first_walk() -> Walk {
    let look = Settling::UNDER_KVM;
    let minutes = |it: u64| Duration::from_secs(it * 60);
    let tab = Chord::key("tab");
    let walk = Walk::named("first-start")
        .expect_line(
            "the sign-in screen goes up",
            "the sign-in screen is going up",
            Since::Boot,
            minutes(20),
        )
        .look("first-start", look.at_most(minutes(2)))
        .press(tab.clone())
        .look("on-continue", look)
        .press(Chord::key("ret"))
        .look("which-time-zone", look)
        .press(Chord::key("esc"))
        .look("back-on-the-overview", look)
        .press(Chord::key("ret"))
        .look("the-name", look)
        .type_text(NAME)
        .look("the-name-typed", look)
        .press(Chord::key("ret"))
        .look("the-pin", look)
        .press_times(&tab, 3)
        .look("on-use-password-only", look)
        .press(Chord::key("ret"))
        .look("the-password", look)
        .type_text(PASSWORD)
        .press(tab.clone())
        .type_text(PASSWORD)
        .look("the-password-typed-twice", look)
        .mark()
        .press(Chord::key("ret"))
        .expect_line(
            "the account is made and its session runs",
            "alo-sessiond: a desktop is running",
            Since::Mark,
            minutes(3),
        )
        .look("how-much-can-alo-do", look.at_most(minutes(3)))
        .mark()
        .press(tab.clone())
        .press(Chord::key("ret"))
        .expect_line(
            "how much alo can do is answered",
            "how much can alo do:",
            Since::Mark,
            minutes(1),
        )
        .look("the-desktop", look.at_most(minutes(3)))
        .expect_line_saying(
            "the applications arrived",
            "of this machine's applications arrived",
            ", 0 still owed",
            Since::Boot,
            minutes(60),
        )
        .look(
            "the-desktop-with-its-applications",
            look.at_most(minutes(2)),
        )
        .press(Chord::with("meta_l", "i"))
        .look("settings", look.at_most(minutes(2)))
        .press(Chord::key("end"))
        .look("accessibility", look)
        .press(tab)
        .look("on-the-screen-reader", look)
        .mark()
        .press(Chord::key("spc"))
        .look("the-screen-reader-on", look)
        .expect_line(
            "the screen reader started",
            "Started alo-screen-reader.service",
            Since::Mark,
            minutes(1),
        )
        .expect_no_line(
            "the screen reader stays up, quietly",
            &[
                "strange signature",
                "Traceback",
                "alo-screen-reader.service: Failed",
                "alo-screen-reader.service: Main process exited",
            ],
            Since::Mark,
            minutes(2),
        )
        .look("two-minutes-later", look);
    let walk = then_the_terminal_at_rest(walk);
    // **The pointer**, before anything that needs a provider: what a person
    // would notice about it, on the desktop and over applications alo did not
    // make (`crate::the_pointer_walk`).
    let walk = crate::the_pointer_walk::then_the_pointer(walk);
    then_the_provider(walk)
}

/// Then close Settings and open the terminal with the pointer nowhere near
/// it, and check its band draws **no controls at rest**
/// (`docs/design/the-window-controls.md`). The walk never moves the pointer,
/// so a band that shows its controls here is showing them to nobody pointing.
///
/// Read from the line the shell says when a band over one of alo's windows
/// changes what it draws (`alo_shell`'s `the_bands_drawn_are_said`), since
/// the mark before the chord, so a terminal opened earlier cannot answer.
fn then_the_terminal_at_rest(walk: Walk) -> Walk {
    let look = Settling::UNDER_KVM;
    walk.press(Chord::key("esc"))
        .look("settings-closed", look)
        .mark()
        .press(Chord::holding(&["ctrl", "alt"], "t"))
        .expect_line(
            "the terminal's band draws no controls at rest",
            THE_TERMINAL_AT_REST,
            Since::Mark,
            Duration::from_secs(60),
        )
        .look("the-terminal-at-rest", look)
        .mark()
        .press(Chord::with("alt", "f3"))
        .look("the-terminals-panel", look)
        .press(Chord::key("right"))
        .press(Chord::key("ret"))
        .expect_line(
            "Window actions maximises the terminal to fill the work area",
            THE_TERMINAL_MAXIMISED,
            Since::Mark,
            Duration::from_secs(60),
        )
        .look("the-terminal-maximised", look)
}

/// What the shell says when the terminal's band draws no controls.
pub const THE_TERMINAL_AT_REST: &str = "window band over the terminal: 0 controls drawn";

/// What the shell says when the window panel has maximised the terminal and
/// it fills the work area of its display — the start of the line, which goes
/// on to say the size.
pub const THE_TERMINAL_MAXIMISED: &str =
    "window panel: the terminal maximised, filling the work area";

/// Then add a provider, typing its key from [`THE_PROVIDER_KEY`] — and
/// nothing at all without it.
fn then_the_provider(walk: Walk) -> Walk {
    let look = Settling::UNDER_KVM;
    let tab = Chord::key("tab");
    let walk = walk
        .only_with(THE_PROVIDER_KEY)
        .press(Chord::key("esc"))
        .press(Chord::with("meta_l", "i"))
        .look("settings-again", look)
        .press(Chord::key("home"))
        .press(tab.clone())
        .press_times(&Chord::key("down"), 4)
        .look("on-add-a-provider", look)
        .press(Chord::key("ret"))
        .look("the-provider-form", look)
        .type_text("openai")
        .press(tab.clone())
        .type_text("https://api.openai.com")
        .press(tab.clone())
        .type_secret(THE_PROVIDER_KEY)
        .press(tab)
        .type_text("gpt-4o-mini")
        .look("the-provider-form-filled", look)
        .press(Chord::key("ret"))
        .look("the-provider-saved", look);
    // With a provider, alo can be asked: Demo 2's second road, at full access.
    let walk = crate::writing_in_vscodium::at_full_access(walk);
    // And then every other application a fresh machine ships, where asked for.
    let walk = crate::in_every_shipped_application::at_full_access(walk);
    // And then at *ask each time*: alo asks for an application nobody gave
    // it, on the sheet, and the grant goes with its task.
    crate::asking_for_an_application_at_each_ask::for_this_task(walk)
}

#[cfg(test)]
#[path = "the_first_walk_tests.rs"]
mod tests;
