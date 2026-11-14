//! **The first walk**: a fresh disk from its first screen to the desktop, the
//! applications it installs on first boot, and the screen reader switched on
//! in Settings — each check with its evidence.
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
//! **The provider section is unverified and switched off.** It types an AI
//! provider's key from [`THE_PROVIDER_KEY`], and runs only when that is set.
//! Its keys are the walk by hand's of 2026-11-14 (`w28-provider.sh`), which
//! opened Settings fresh; whether they reach the same rows from where this
//! walk leaves Settings is not yet seen.

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
    then_the_provider(walk)
}

/// Then add a provider, typing its key from [`THE_PROVIDER_KEY`] — and
/// nothing at all without it.
fn then_the_provider(walk: Walk) -> Walk {
    let look = Settling::UNDER_KVM;
    let tab = Chord::key("tab");
    walk.only_with(THE_PROVIDER_KEY)
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
        .look("the-provider-saved", look)
}

#[cfg(test)]
#[path = "the_first_walk_tests.rs"]
mod tests;
