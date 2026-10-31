//! Which installed application fills each of the Dock's default roles, and
//! which one is this machine's terminal.
//!
//! `alo_dock::pinning::Discovery` is the shape of this answer and has been
//! since the roles were written down. **Nothing produced one.** Its own header
//! says why it could not: *whoever reads desktop entries off a Linux machine
//! fills it; this crate could not and should not.* `alo-software` learned to
//! read the machine, and `alo-dock` holds the roles, and **only a binary sees
//! both** — `alo-saying` is the one other crate that does and it is the
//! translation collector. So the join lives here, beside the process it is
//! for, exactly as the launch sender does and for the same reason.
//!
//! # Two of three roles, and the third is named rather than faked
//!
//! `ARole::Settings` opens the compositor's own surface. The owner's ruling of
//! 2026-10-12 forbids inventing an identifier or a bus endpoint for it, and
//! `Discovery::Completed` carries `(ARole, AppId)` — an identifier. So Settings
//! is **deliberately absent from what this produces** and reaches the Dock as a
//! native-action entry, which is separate work. A reader who adds
//! `(ARole::Settings, AppId::named("…"))` here would be inventing the thing the
//! ruling forbids, which is why this paragraph is longer than the code.
//!
//! # A failure is reported as a failure
//!
//! Two things can go wrong and neither yields an empty list.
//! `alo_software::what_this_machine_has` returns no list at all when the tool
//! would not answer, and `Shipped::decided` can fail to parse the built-in
//! list. Both become [`Discovery::Failed`], because
//! `alo_dock::pinning::ThePins::what_to_pin` then answers `NotYet` and **writes
//! nothing down** — which is the owner's fourth clause: *an unfinished scan or
//! discovery failure is not proof that an application is absent.*
//!
//! A scan that succeeded and found no browser is a different fact: that is
//! `Completed` with the browser missing, a Dock with the roles it can fill, and
//! it is written down.
//!
//! # A machine still installing is `SoFar`, not `Completed`
//!
//! This said, until 2026-10-31, that *this runs once, before the desktop stands
//! up, and answers. There is no partial state to report.* **There was one, and
//! it was the commonest machine of all**: a fresh one, whose applications
//! `alo-shipping` installs after its first boot. A booted machine's desktop
//! asked at 669 s, the tool truthfully answered *nothing installed*, and that
//! became `Completed([])` — a Dock of Settings and the market, **written into
//! the person's folder for good** — while Firefox arrived at 1429 s and the
//! rest after it.
//!
//! So a reading taken before `alo_software::the_applications_arrived` marks the
//! walk done is [`Discovery::SoFar`]: drawn, never written down. And the
//! reading is taken **again** whenever the machine's applications change
//! (`crate::watching_what_is_installed`), so what arrives reaches the Dock, the
//! alo Bar and the terminal chord without anybody signing out.
//!
//! # Where it looks, whatever started it
//!
//! The entries and the pictures are read from `XDG_DATA_DIRS` **with both
//! flatpak installations added** (`alo_installed::with_what_flatpak_exports`).
//! This process is a system unit and is given no flatpak directories by its
//! environment; without them, every flatpak on the machine is invisible here.

use std::path::{Path, PathBuf};

use alo_dock::artwork::TheArtwork;
use alo_dock::pinning::{ARole, Discovery};
use alo_dock::window::AppId;
use alo_installed::NotRead;
use alo_shell::AnApplicationHere;
use alo_software::the_applications_arrived::TheApplicationsArrived;
use alo_software::{Role, Shipped, Tool};

use crate::the_artwork_each_application_has::{HowItWent, the_artwork_each_application_has};

/// What one reading of this machine tells the desktop.
///
/// **Two answers from one reading, and that is the point of the type.** Reading
/// the machine means waiting for the rented tool — a subprocess — and this
/// happens before the display is taken, so a second reading would delay the
/// desktop's appearance twice over for the same information.
#[derive(Debug)]
pub struct Offered {
    /// The Dock roles this machine can fill, in [`ARole::EVERY`]'s order.
    pub roles: Discovery,
    /// This machine's terminal, when it has one installed.
    ///
    /// **[`None`] is a real machine and not a fault**, and it is two different
    /// real machines: one where the scan failed, and one where the scan
    /// succeeded and found no terminal. Neither may be told apart from here and
    /// neither should be guessed at — `alo_shell::Server::its_terminal_is` is
    /// simply never called, and the chord says on the error stream that this
    /// machine has none.
    pub terminal: Option<AppId>,
    /// The picture each installed application draws on the Dock.
    ///
    /// **Empty is a real machine too**, and three different ones: nothing
    /// declared an icon, the artwork is all scalable, or none of it decoded.
    /// [`HowItWent`] is carried beside it so a report can say which, instead of
    /// *icons did not appear*.
    pub artwork: TheArtwork,
    /// How the artwork reading went — for saying so, and for nothing else.
    pub artwork_went: HowItWent,
    /// Every application a person could open, as its desktop entry names it —
    /// what the alo Bar searches. **From the entries, not from the rented
    /// tool**, so an application the image itself carries, LibreOffice in
    /// `/usr/share/applications`, is here beside the flatpaks.
    pub installed: Vec<AnApplicationHere>,
    /// The entries that were there and could not be read, for the log.
    pub not_read: Vec<NotRead>,
}

impl Offered {
    /// The same reading, in the shape a running seat is told it.
    #[must_use]
    pub fn as_told(&self) -> alo_shell::WhatTheMachineOffersNow {
        alo_shell::WhatTheMachineOffersNow {
            roles: self.roles.clone(),
            terminal: self.terminal.clone(),
            artwork: self.artwork.clone(),
            installed: self.installed.clone(),
        }
    }
}

/// Where one reading of the machine looks.
///
/// **A value rather than the environment read in place**, so a test can lay a
/// directory tree out like the image and read it through exactly the code a
/// machine runs.
#[derive(Debug, Clone)]
pub struct WhereToLook {
    /// The `applications/` directories, most specific first.
    pub entries: Vec<PathBuf>,
    /// The icon directories, in the same order.
    pub icons: Vec<PathBuf>,
    /// The data directories both lists were made from, flatpak's exports
    /// included — what an application a press starts is handed as its own
    /// `XDG_DATA_DIRS`, so it finds its data where the Dock found its entry.
    pub data_dirs: std::ffi::OsString,
    /// The mark that says the fresh machine's applications were walked.
    pub arrived: TheApplicationsArrived,
    /// The languages the person reads, best first, as BCP 47 tags — what an
    /// entry's localised `GenericName` and `Keywords` are read in, so the alo
    /// Bar finds an application by what it is in their words. Empty reads the
    /// entries' own words only.
    pub languages: Vec<String>,
}

impl WhereToLook {
    /// Where this process looks on the machine it runs on: its own `HOME`,
    /// `XDG_DATA_HOME` and `XDG_DATA_DIRS`, with both flatpak installations
    /// added.
    #[must_use]
    pub fn on_this_machine() -> Self {
        let home = std::env::var_os("HOME").map(PathBuf::from);
        let data_home = std::env::var_os("XDG_DATA_HOME");
        let data_dirs = std::env::var_os("XDG_DATA_DIRS");
        Self::beneath(
            Path::new(alo_installed::THE_MACHINES_INSTALLATION),
            home.as_deref(),
            data_home.as_deref(),
            data_dirs.as_deref(),
            TheApplicationsArrived::on_this_machine(),
        )
    }

    /// Where a reading looks, given the machine's flatpak installation and the
    /// three variables — the one road both a machine and a test take.
    #[must_use]
    pub fn beneath(
        installation: &Path,
        home: Option<&Path>,
        data_home: Option<&std::ffi::OsStr>,
        data_dirs: Option<&std::ffi::OsStr>,
        arrived: TheApplicationsArrived,
    ) -> Self {
        let data_dirs =
            alo_installed::with_what_flatpak_exports(installation, home, data_home, data_dirs);
        Self {
            entries: alo_installed::where_entries_live(home, data_home, Some(&data_dirs)),
            icons: alo_icons::where_icons_live(home, data_home, Some(&data_dirs)),
            data_dirs,
            arrived,
            languages: Vec::new(),
        }
    }

    /// The same places, read in these languages.
    #[must_use]
    pub fn read_in(self, languages: Vec<String>) -> Self {
        Self { languages, ..self }
    }
}

/// The Dock roles this machine can fill, its terminal, its artwork and every
/// application on it, read from the machine this process runs on.
///
/// **In the entries' own words only.** The desktop itself reads through
/// [`what_is_offered`] with places that know the person's languages; this is
/// the tests' way of asking the machine they run on.
#[cfg(test)]
pub fn what_this_machine_offers() -> Offered {
    what_is_offered(
        &WhereToLook::on_this_machine(),
        &alo_software::rented::TheRentedTool::on_this_machine(),
    )
}

/// One reading, from these places and this tool.
///
/// The roles and the terminal come from the tool, matched against the
/// applications this system ships for each; the artwork and the list a person
/// searches come from the desktop entries.
///
/// **The entries are read even where the rented tool failed.** The readings
/// share nothing, and giving up on the icons and the launcher because the
/// tool would not answer would cost a person every application on the machine
/// for a reason that has nothing to do with them.
pub fn what_is_offered(at: &WhereToLook, tool: &(impl Tool + ?Sized)) -> Offered {
    // **The size asked for is `alo_dock::measures::ICON`**, the logical width
    // of the box the Dock draws an application into: the picture is scaled
    // where it is drawn, by the surface that knows which display it is on.
    let (artwork, artwork_went) =
        the_artwork_each_application_has(&at.entries, &at.icons, alo_dock::measures::ICON);
    // **A second walk of the same directories**, because the artwork reader
    // keeps its own. They can disagree only if an entry changes between the two,
    // and the watcher then sees that change and reads again.
    let (found, _icons, not_read) =
        alo_installed::read_with_their_icons_saying_what_did_not(&at.entries);
    // **And a third, for what each one is**, which reads only the entries the
    // walk above listed — so an entry asking not to be listed is not found by
    // its kind either.
    let languages: Vec<&str> = at.languages.iter().map(String::as_str).collect();
    let mut what_each_is = alo_installed::what_they_are(&at.entries, &languages);
    let installed = found
        .all()
        .map(|application| AnApplicationHere {
            is: what_each_is
                .remove(application.identifier())
                .unwrap_or_default(),
            identifier: application.identifier().to_owned(),
            called: match application.name() {
                Some(alo_applications::application::Called::ByWhoeverPackagedIt(called)) => {
                    Some(called.clone())
                }
                // An entry read off a disk is always the packager's words; ours
                // are only ever given in code, never read here.
                _ => None,
            },
        })
        .collect();
    let (roles, terminal) = the_roles_and_the_terminal(tool, at.arrived.already());
    Offered {
        roles,
        terminal,
        artwork,
        artwork_went,
        installed,
        not_read,
    }
}

/// The roles and the terminal, from the tool — and whether the answer is final.
///
/// `walked` is whether `alo-shipping` has finished with this machine. Until it
/// has, what the tool lists is what has arrived **so far**.
fn the_roles_and_the_terminal(
    tool: &(impl Tool + ?Sized),
    walked: bool,
) -> (Discovery, Option<AppId>) {
    let Ok(has) = alo_software::what_this_machine_has::what_this_machine_has(tool) else {
        return (Discovery::Failed, None);
    };
    let Ok(ships) = Shipped::decided() else {
        return (Discovery::Failed, None);
    };
    let filled = filled_by(&has, &ships);
    let roles = if walked {
        Discovery::Completed(filled)
    } else {
        Discovery::SoFar(filled)
    };
    (roles, the_terminal_among(&has, &ships))
}

/// The terminal this machine has installed, from one reading and one shipped
/// list.
///
/// **Not a Dock role, and deliberately not one.** `ARole` is the list of roles
/// a **fresh machine pins**, and `what_a_fresh_machine_pins` answers Files,
/// Browser and Settings — the owner's own three. Adding the terminal to `ARole`
/// would pin it and contradict them, so the terminal is asked for separately
/// and reaches the keyboard rather than the Dock.
///
/// Separated from the reading above so it can be tested at all, exactly as
/// [`filled_by`] is.
fn the_terminal_among(
    has: &alo_software::what_this_machine_has::WhatThisMachineHas,
    ships: &Shipped,
) -> Option<AppId> {
    let identifier = ships.the(Role::Terminal).application().identifier();
    if !has.installed().has(identifier) {
        // **Omitted and not placeholdered**, which is the owner's words and the
        // same rule `filled_by` follows: *omit unavailable applications; never
        // show placeholders or non-working buttons.*
        return None;
    }
    AppId::named(identifier).ok()
}

/// The pairs, from one reading and one shipped list.
///
/// **Separated from the two readings so it can be tested at all.** The function
/// above asks a real machine for a real tool's output; this one is given both
/// and is where every rule lives — the order, the two roles, and the omission
/// of anything not installed.
fn filled_by(
    has: &alo_software::what_this_machine_has::WhatThisMachineHas,
    ships: &Shipped,
) -> Vec<(ARole, AppId)> {
    let mut filled = Vec::new();
    // **In `ARole::EVERY`'s order**, skipping the role no scan can answer.
    // Written as a match over every role rather than a list of the two, so that
    // a role added to `ARole` is a compile error here rather than a Dock slot
    // that silently never fills.
    for role in ARole::EVERY {
        let wanted = match role {
            ARole::Files => Role::FileManager,
            ARole::Browser => Role::WebBrowser,
            // **Neither is an application.** See this module's header: a
            // `(role, AppId)` pair for either would be inventing the identifier
            // the owner's ruling forbids, and `alo_dock::pinning` fills both
            // from the role itself.
            ARole::Settings | ARole::Applications => continue,
        };
        let identifier = ships.the(wanted).application().identifier();
        if !has.installed().has(identifier) {
            // **Omitted and not placeholdered**, which is the owner's words:
            // *omit unavailable applications; never show placeholders or
            // non-working buttons.*
            continue;
        }
        let Ok(app) = AppId::named(identifier) else {
            // An identifier this machine shipped that the Dock will not take is
            // a packaging fault, not a reason to pin something else.
            continue;
        };
        filled.push((role, app));
    }
    filled
}

#[cfg(test)]
#[path = "what_this_machine_offers_tests.rs"]
mod tests;
