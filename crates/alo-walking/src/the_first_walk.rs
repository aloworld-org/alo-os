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

use alo_acting::said_in_settings as said;

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
    // **Every window's band and controls, first among the sections** — a
    // section for each kind, so one band failing does not hide the others,
    // and before the long ones, so none can leave them not reached (the
    // coordinator, 2026-11-21). They need no provider.
    let walk = crate::every_windows_band::every_windows_band(walk);
    // **The providers, a section of their own**, which every section alo is
    // asked in needs: a section runs to the next, so without one of their own
    // they would be inside the bands' and stopped by a band that failed.
    let walk = walk.section(THE_PROVIDERS, &[], |walk| {
        then_the_provider(then_a_provider_that_needs_no_key(walk))
    });
    then_alo_is_asked(walk)
}

/// The section that adds the providers, which every section alo is asked in
/// needs.
const THE_PROVIDERS: &str = "the providers";

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

/// What `alo-desktop` says when a provider that needs no key was saved — its
/// `SAVED_NEEDING_NO_KEY`, which a test there reads from this line.
pub const A_PROVIDER_THAT_NEEDS_NO_KEY_SAVED: &str =
    "alo-desktop: a provider that needs no key was saved";

/// The most Downs a row of *alo and AI* is looked for with: more than the page
/// has rows, so only a row that is not there fails.
const ROWS_AT_MOST: usize = 24;

/// The most Tabs a field of the provider's form is looked for with: more than
/// the form has fields.
const FIELDS_AT_MOST: usize = 8;

/// Then add a provider that needs no key, the way a person adds a runtime of
/// their own on this computer: the key left empty, the switch after it turned
/// on with Space, and Enter.
///
/// **On every walk**, before the gate that needs a real provider's key,
/// because it needs none. Nothing answers at that address on a walk's
/// machine, and nothing here asks it anything: what is walked is that the
/// form saves it. **It is added before the provider with a key**, so that one
/// is what answers the sections after it.
fn then_a_provider_that_needs_no_key(walk: Walk) -> Walk {
    let look = Settling::UNDER_KVM;
    let (tab, down) = (Chord::key("tab"), Chord::key("down"));
    walk.press(Chord::key("esc"))
        .press(Chord::with("meta_l", "i"))
        .look("settings-for-a-provider-that-needs-no-key", look)
        .press(Chord::key("home"))
        // Into the page, then **down to the row by its name**, never by
        // counting (`alo_acting::said_in_settings`).
        .press(tab.clone())
        .mark()
        .press_until_the_keyboard_is_on(&down, said::ADD_A_PROVIDER, ROWS_AT_MOST)
        .look("on-add-a-provider-that-needs-no-key", look)
        .press(Chord::key("ret"))
        .type_text("On this computer")
        .press_until_the_keyboard_is_on(&tab, said::ADDRESS, FIELDS_AT_MOST)
        .type_text("http://127.0.0.1:11434")
        // Past the key, left empty, to the switch.
        .press_until_the_keyboard_is_on(&tab, said::NEEDS_NO_KEY, FIELDS_AT_MOST)
        .press(Chord::key("spc"))
        .press_until_the_keyboard_is_on(&tab, said::MODEL, FIELDS_AT_MOST)
        .type_text("llama3.2")
        .look("the-keyless-provider-form", look)
        .mark()
        .press(Chord::key("ret"))
        .expect_line(
            "a provider that needs no key is saved",
            A_PROVIDER_THAT_NEEDS_NO_KEY_SAVED,
            Since::Mark,
            Duration::from_secs(60),
        )
        .look("the-keyless-provider-saved", look)
}

/// Then add a provider, typing its key from [`THE_PROVIDER_KEY`] — and
/// nothing at all without it.
fn then_the_provider(walk: Walk) -> Walk {
    let look = Settling::UNDER_KVM;
    let (tab, down) = (Chord::key("tab"), Chord::key("down"));
    walk.only_with(THE_PROVIDER_KEY)
        .press(Chord::key("esc"))
        .press(Chord::with("meta_l", "i"))
        .look("settings-again", look)
        .press(Chord::key("home"))
        // **Down to the row by its name**: five Downs, counted after the
        // provider that needs no key, landed on *alo* (run 44851000278,
        // picture 54), because that provider brings a row and its button.
        .press(tab.clone())
        .mark()
        .press_until_the_keyboard_is_on(&down, said::ADD_A_PROVIDER, ROWS_AT_MOST)
        .look("on-add-a-provider", look)
        .press(Chord::key("ret"))
        .look("the-provider-form", look)
        .type_text("openai")
        .press_until_the_keyboard_is_on(&tab, said::ADDRESS, FIELDS_AT_MOST)
        .type_text("https://api.openai.com")
        .press_until_the_keyboard_is_on(&tab, said::KEY, FIELDS_AT_MOST)
        .type_secret(THE_PROVIDER_KEY)
        // Past the switch that says a service needs no key, left off.
        .press_until_the_keyboard_is_on(&tab, said::MODEL, FIELDS_AT_MOST)
        .type_text("gpt-4o-mini")
        .look("the-provider-form-filled", look)
        .press(Chord::key("ret"))
        .look("the-provider-saved", look)
}

/// Then the six sections alo is asked in, **each needing the providers**.
fn then_alo_is_asked(walk: Walk) -> Walk {
    // **Four independent sections**, each starting from its own Escape and
    // setting its own level and grants, so a failure in one does not hide the
    // others: the walk still fails, and the report gives each its result.
    // Each needs the providers, and the last needs one before it too.
    //
    // With a provider, alo can be asked: Demo 2's second road, at full access.
    let walk = walk.section(
        "VSCodium at full access",
        &[THE_PROVIDERS],
        crate::writing_in_vscodium::at_full_access,
    );
    // And then every other application a fresh machine ships, where asked for.
    let walk = walk.section(
        "every application at full access",
        &[THE_PROVIDERS],
        crate::in_every_shipped_application::at_full_access,
    );
    // And then at *ask each time*, approving each change on its card.
    let walk = walk.section(
        "VSCodium at ask each time",
        &[THE_PROVIDERS],
        crate::writing_in_vscodium_at_each_ask::at_each_ask,
    );
    // And then alo asks for an application nobody gave it, on the sheet,
    // and the grant goes with its task. **It needs ask-each-time VSCodium**:
    // a failure there leaves its turn waiting on a card, and the daemon serves
    // one turn at a time, so this ask would fail on a broken pipe (runs
    // 44069105568 and 44161055698). A Stop shortcut would be a product
    // promise, and is not being built here.
    let walk = walk.section(
        "the Papers sheet at ask each time",
        &[THE_PROVIDERS, "VSCodium at ask each time"],
        crate::asking_for_an_application_at_each_ask::for_this_task,
    );
    // **And an application installed, by the person and by alo** (launch
    // item 6), each setting its own level and needing only the providers;
    // both only with the walk's *installing* box
    // (`crate::installing_an_application`).
    let walk = walk.section(
        "Kate from the market",
        &[THE_PROVIDERS],
        crate::installing_an_application::kate_from_the_market,
    );
    walk.section(
        "Kooha installed by alo",
        &[THE_PROVIDERS],
        crate::installing_an_application::kooha_installed_by_alo,
    )
}

#[cfg(test)]
#[path = "the_first_walk_tests.rs"]
mod tests;
