//! **A person installs an application, and alo works in it; alo installs one
//! when asked, and works in that** — launch item 6, *a person installs an
//! application, and the agent can work with it*, walked. Before this no walk
//! had installed anything (the launch audit of 2026-11-21).
//!
//! Two applications the catalogue offers and a fresh machine does not ship,
//! each small and needing no account, one of each toolkit:
//!
//! - **Kate** ([`KATE`], Qt), installed **by the person**, from the market;
//! - **Kooha** ([`KOOHA`], GTK 4), installed **by alo**, asked in the alo bar
//!   at *ask each time* and approved on its card.
//!
//! # What each section does, and what each check is
//!
//! **Kate from the market**:
//!
//! 1. F6 puts the keyboard on the Dock; Tab until the Dock says it is on
//!    *Applications* (`alo_dock::words::APPLICATIONS_ON_THE_DOCK`, by name);
//!    Enter opens the market, which says so.
//! 2. Down until the market says it is on [`KATE`], by its identifier; Enter.
//!    The market says the person asked, then **an installation** — arrived or
//!    not done, read as one check, so a refusal fails at once saying why
//!    (`alo_acting::said_in_the_market::AN_INSTALLATION`).
//! 3. At full access, the person makes `kate.txt` and asks alo to open it in
//!    Kate, type a line and save; the file on disk is checked to hold it.
//!
//! **Kooha installed by alo**:
//!
//! 1. *Ask me each time*, then *Install Kooha.* The card goes up; *Allow
//!    once*. The turn says `install_application` in [`KOOHA`], done.
//! 2. At full access, alo is asked to open Kooha and say what its window
//!    shows, then to press one of its switches: `open_application`,
//!    `accessible.read_window` and `accessible.activate_control` each done.
//!
//! # What a pass does not prove
//!
//! Kate's file is checked from outside; the rest is the machine's own account
//! (`alo_acting::said_on_the_log`). *Done* for a press includes *done, and the
//! application did not answer*, so which switch moved is read off the
//! pictures, by a person. The level is set by counting rows, as the Papers
//! section does, because the levels say no name on the journal yet.
//!
//! # Only when asked for
//!
//! Both run only with [`INSTALLING`] set — the *Walk* workflow's *installing*
//! box — so that until they pass, a walk after each merge is not red because
//! of them.

use std::time::Duration;

use alo_acting::Opened;
use alo_acting::said_in_the_market as market;
use alo_acting::said_on_the_log::{A_CARD_IS_UP, InAnApplication};

use crate::asking_for_an_application_at_each_ask::{DOWN_TO_RUNNING_CODE, TYPING_NOTHING};
use crate::in_every_shipped_application::{
    READ_WINDOW, TYPE_TEXT, acted, done_in, opened_a_file_in,
};
use crate::keys::Chord;
use crate::settling::Settling;
use crate::step::{Since, Walk};
use crate::writing_in_vscodium_at_each_ask::ASK_EACH_TIME;

/// The variable that turns these sections on.
pub const INSTALLING: &str = "ALO_WALK_INSTALLING";

/// Kate, the KDE text editor: Qt, installed by the person from the market.
pub const KATE: &str = "org.kde.kate";

/// Kooha, a screen recorder: GTK 4, installed by alo when asked.
pub const KOOHA: &str = "io.github.seadve.Kooha";

/// `accessible.activate_control`, held to `alo_adapters::ACTIVATE_CONTROL` by
/// this file's tests.
pub const ACTIVATE_CONTROL: &str = "accessible.activate_control";

/// The row of *At full trust* on *Running code*, from nought, in the order
/// the page lists the levels.
pub const FULL_TRUST: usize = match alo_capability::HowCodeRuns::ALL {
    [_, _, alo_capability::HowCodeRuns::AtFullTrust] => 2,
    _ => 0,
};

/// The file alo writes in Kate, in the person's home.
pub const KATES_FILE: &str = "kate.txt";

/// What alo is asked to type into it.
pub const KATES_TEXT: &str = "Written by alo in Kate";

/// The journal tag the person's check that the file was made goes under.
pub const KATES_FILE_IS_MADE: &str = "alo-walk-kates-file-is-made";

/// The journal tag the person's check of the file goes under.
pub const KATES_NOTE: &str = "alo-walk-kates-note";

/// What the person asks alo to install.
pub const INSTALL_KOOHA: &str = "Install Kooha.";

/// The most Tabs the market's entry is looked for with: more than the bar has
/// slots.
const SLOTS_AT_MOST: usize = 24;

/// The most Downs Kate is looked for with: more than the catalogue holds.
const CARDS_AT_MOST: usize = 128;

/// What the person asks alo to do in Kate.
#[must_use]
pub fn the_ask_in_kate() -> String {
    format!(
        "Open the file {KATES_FILE} in my home folder in Kate, type exactly this text into it: \
         {KATES_TEXT} and then save it."
    )
}

/// What the person asks alo to do in Kooha, after it is open.
pub const THE_ASKS_IN_KOOHA: [&str; 2] = [
    "Open Kooha and tell me what its window shows.",
    "In Kooha, press one of its switches that turns an option on or off, and tell me which.",
];

/// What an installation that arrived says after its start.
#[must_use]
pub fn arrived(identifier: &str) -> String {
    market::an_installation_arrived(identifier)
        .trim_start_matches(market::AN_INSTALLATION)
        .to_owned()
}

/// Set *Running code* to the level `down` rows from its first, in Settings.
fn the_level(walk: Walk, picture: &str, down: usize) -> Walk {
    let look = Settling::UNDER_KVM;
    let (tab, ret, esc) = (Chord::key("tab"), Chord::key("ret"), Chord::key("esc"));
    let down_key = Chord::key("down");
    walk.press(esc.clone())
        .press(Chord::with("meta_l", "i"))
        .look(
            &format!("{picture}-settings"),
            look.at_most(Duration::from_secs(60)),
        )
        .press(Chord::key("home"))
        .press_times(&down_key, DOWN_TO_RUNNING_CODE)
        .press(tab)
        .press_times(&down_key, down)
        .look(&format!("{picture}-on-the-level"), look)
        .press(ret)
        .press(esc)
}

/// Ask alo, in the alo bar, and mark the log as Enter goes.
fn asking(walk: Walk, picture: &str, ask: &str) -> Walk {
    walk.press(Chord::key("esc"))
        .press(Chord::with("meta_l", "a"))
        .type_text(ask)
        .look(picture, Settling::UNDER_KVM)
        .mark()
        .press(Chord::key("ret"))
}

/// The section in which the person installs Kate from the market and alo
/// works in it.
#[must_use]
pub fn kate_from_the_market(walk: Walk) -> Walk {
    walk.section_only_with(INSTALLING, the_person_installs_kate)
}

/// The section in which alo installs Kooha when asked, and works in it.
#[must_use]
pub fn kooha_installed_by_alo(walk: Walk) -> Walk {
    walk.section_only_with(INSTALLING, alo_installs_kooha)
}

/// Kate, from the market, by the person.
fn the_person_installs_kate(walk: Walk) -> Walk {
    let look = Settling::UNDER_KVM;
    let minutes = |it: u64| Duration::from_secs(it * 60);
    let (tab, down, ret, esc) = (
        Chord::key("tab"),
        Chord::key("down"),
        Chord::key("ret"),
        Chord::key("esc"),
    );
    let terminal = Chord::holding(&["ctrl", "alt"], "t");
    let walk = walk
        .press(esc.clone())
        .press(esc.clone())
        .look_as_it_settles("kate-before", look.at_most(minutes(1)))
        // **To the market's entry, by its name.**
        .mark()
        .press(Chord::key("f6"))
        .press_until_it_says(
            &tab,
            market::ON_THE_DOCK_THE_KEYBOARD_IS_ON,
            alo_dock::words::APPLICATIONS_ON_THE_DOCK.says(),
            SLOTS_AT_MOST,
        )
        .look("kate-on-the-markets-entry", look)
        .mark()
        .press(ret.clone())
        .expect_line(
            "the market opens",
            market::THE_MARKET_OPENED,
            Since::Mark,
            minutes(1),
        )
        .look("kate-the-market", look.at_most(minutes(1)))
        // **To Kate's card, by its identifier.**
        .press_until_it_says(
            &down,
            market::IN_THE_MARKET_THE_KEYBOARD_IS_ON,
            KATE,
            CARDS_AT_MOST,
        )
        .look("kate-on-its-card", look)
        .mark()
        .press(ret)
        .expect_line_saying(
            "the person asks the market for Kate",
            market::A_PERSON_ASKED_TO_INSTALL,
            &format!("{KATE:?}"),
            Since::Mark,
            minutes(1),
        )
        .look("kate-being-installed", look.at_most(minutes(1)))
        // **Arrived or not done, whichever is said first**, so a refusal
        // fails here saying why.
        .expect_line_saying(
            "Kate arrives from the market",
            market::AN_INSTALLATION,
            &arrived(KATE),
            Since::Mark,
            minutes(20),
        )
        .look("kate-arrived", look.at_most(minutes(1)))
        .press(esc.clone())
        .press(esc);
    // **alo in Kate**, at full access.
    let walk = the_level(walk, "kate-full-trust", FULL_TRUST)
        .press(terminal.clone())
        .look_as_it_settles("kate-the-terminal", look.at_most(minutes(1)))
        .type_text(&format!(
            " cd ~ && touch {KATES_FILE} && systemd-cat -t {KATES_FILE_IS_MADE} ls {KATES_FILE}"
        ))
        .mark()
        .press(Chord::key("ret"))
        .expect_line_saying(
            "the person made kate.txt",
            KATES_FILE_IS_MADE,
            KATES_FILE,
            Since::Mark,
            minutes(1),
        );
    asking(walk, "kate-the-ask", &the_ask_in_kate())
        .expect_line_saying(
            "alo opens kate.txt in Kate, in the person's window",
            &opened_a_file_in(KATE),
            Opened::InTheirWindow.written(),
            Since::Mark,
            minutes(5),
        )
        .expect_line_saying_or_after_a_retry(
            "alo types the text into Kate and saves it",
            &acted(TYPE_TEXT, KATE),
            &format!("{KATE}, {}", InAnApplication::TypedAndSaved.written()),
            Since::Mark,
            minutes(5),
        )
        .look("kate-with-the-text", look.at_most(minutes(1)))
        .press(Chord::key("esc"))
        .press(terminal)
        .type_text(&format!(" systemd-cat -t {KATES_NOTE} cat ~/{KATES_FILE}"))
        .mark()
        .press(Chord::key("ret"))
        .expect_line_saying(
            "kate.txt holds what alo typed",
            KATES_NOTE,
            KATES_TEXT,
            Since::Mark,
            minutes(1),
        )
}

/// Kooha, installed by alo when asked, approved on its card.
fn alo_installs_kooha(walk: Walk) -> Walk {
    let look = Settling::UNDER_KVM;
    let minutes = |it: u64| Duration::from_secs(it * 60);
    let walk = the_level(walk, "kooha-ask-each-time", ASK_EACH_TIME);
    let walk = asking(walk, "kooha-the-ask", INSTALL_KOOHA)
        .expect_line_saying(
            "the installation is asked on its card",
            A_CARD_IS_UP,
            TYPING_NOTHING,
            Since::Mark,
            minutes(3),
        )
        .look("kooha-the-card", look.at_most(minutes(1)))
        // *Allow once*, as the Papers section chooses it.
        .press(Chord::key("tab"))
        .press(Chord::key("ret"))
        .expect_line_saying(
            "after its approval alo installs Kooha",
            &acted(INSTALL_APPLICATION, KOOHA),
            &done_in(KOOHA),
            Since::Mark,
            minutes(20),
        )
        .look("kooha-installed", look.at_most(minutes(1)));
    let [open, press] = THE_ASKS_IN_KOOHA;
    let walk = the_level(walk, "kooha-full-trust", FULL_TRUST);
    let walk = asking(walk, "kooha-open-ask", open)
        .expect_line_saying(
            "alo opens Kooha",
            &acted("open_application", KOOHA),
            &done_in(KOOHA),
            Since::Mark,
            minutes(5),
        )
        .expect_line_saying_or_after_a_retry(
            "alo reads Kooha's window",
            &acted(READ_WINDOW, KOOHA),
            &done_in(KOOHA),
            Since::Mark,
            minutes(5),
        )
        .look("kooha-open", look.at_most(minutes(1)));
    asking(walk, "kooha-press-ask", press)
        .expect_line_saying_or_after_a_retry(
            "alo presses a switch in Kooha",
            &acted(ACTIVATE_CONTROL, KOOHA),
            &done_in(KOOHA),
            Since::Mark,
            minutes(5),
        )
        .look("kooha-pressed", look.at_most(minutes(1)))
}

/// `install_application`, by the name the turn says it by — held to
/// `alo_software::INSTALL_APPLICATION` by this file's tests.
pub const INSTALL_APPLICATION: &str = "install_application";

#[cfg(test)]
#[path = "installing_an_application_tests.rs"]
mod tests;
