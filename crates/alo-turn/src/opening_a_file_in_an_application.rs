//! **`open_file_in_application`**: from *the person approved "open hello.txt in
//! VSCodium"* to *hello.txt is open in VSCodium*.
//!
//! *The owner, 2026-11-14: "Yes, add it."* The order, and why each step is
//! where it is:
//!
//! 1. **The application: granted, and here** — `alo_applications::Reaching`, in
//!    that order, so an ungranted application is refused exactly as one that is
//!    not installed would be.
//! 2. **The file: granted, there, and where it really leads granted too** —
//!    `alo_files::Touching`, the file verbs' own three questions. At full access
//!    the grants permit any file the person's account can open (ADR 0149), and
//!    that is decided where every grant is, not here.
//! 3. **The line**: the application's identifier and the file's **real** path,
//!    the one the grants were asked about — never the path the call arrived
//!    with. A path the line cannot carry is refused by name.
//! 4. **Ask the door**, which answers where it opened — alo's own copy at the
//!    default, the person's own window where they chose that — or why nothing
//!    did, and turn that into the person's own words.
//!
//! **Every refusal is written down, and every one means nothing was opened.**
//! An opening that happened is a change that **ran**, and the record says
//! where it opened.
//!
//! # Where it opens is not decided here
//!
//! The desktop reads the person's own setting at the moment it opens
//! (`alo_acting::opening_a_file`), as it does for working in an application;
//! nothing this crate sends can choose the person's window over alo's copy.
//!
//! **No indicator line and no boundary**, for `crate::acting_on_an_application`'s
//! reasons: nothing here leaves the machine, and what the desktop starts is the
//! person's installed application under their own session. What an
//! application sends once it is open is the indicator's to show on the
//! desktop's side (ADR 0143 Q6), not this executor's.

use std::path::{Path, PathBuf};

use alo_acting::{Opened, ToOpenAFile};
use alo_applications::{NotInstalled, Reaching, words};
use alo_capability::{Authorised, Grants, Refused};
use alo_files::Touching;
use alo_record::Entry;
use alo_strings::{Filling, Said, Strings, Word};

use crate::machine::Machine;
use crate::refusing::NotDone;
use crate::the_door_the_compositor_serves::NotAtTheDoor;

/// A file opened in an application.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AFileOpened {
    /// The application, by its identifier.
    application: String,
    /// The file, as it really is on this machine.
    file: PathBuf,
    /// Where it opened: alo's own copy, or the person's own window.
    opened: Opened,
}

impl AFileOpened {
    /// The application it opened in, by its identifier.
    #[must_use]
    pub fn application(&self) -> &str {
        &self.application
    }

    /// The file, as it really is on this machine.
    #[must_use]
    pub fn file(&self) -> &Path {
        &self.file
    }

    /// Whether it opened in alo's own copy rather than the person's window.
    #[must_use]
    pub const fn in_alos_copy(&self) -> bool {
        matches!(self.opened, Opened::InAlosCopy)
    }
}

/// Carry out one approved `open_file_in_application`.
///
/// Answers the record's entry and what happened, as every executor here does:
/// **a refusal is written down exactly as an execution is**.
pub(crate) fn opening_a_file_in_an_application(
    machine: &mut Machine<'_>,
    authorised: Authorised,
    grants: &Grants,
) -> (Entry, Result<AFileOpened, NotDone>) {
    let strings = machine.strings();
    let at = authorised.at();
    let agent = authorised.under().clone();
    let call = authorised.call().clone();
    let refused = |said: Said| {
        let refused = Refused::worded_elsewhere(call.clone(), said);
        let entry = Entry::refused(&refused, &agent, strings, at);
        (entry, Err(NotDone::Refused(refused)))
    };

    // Step 1. The application: granted, and here.
    let reaching = match Reaching::of(
        authorised,
        grants,
        &machine.the_applications_installed_now(),
        strings,
    ) {
        Ok(reaching) => reaching,
        Err(why) => {
            let entry = Entry::refused(&why, &agent, strings, at);
            return (entry, Err(NotDone::Refused(why)));
        }
    };
    let Some(application) = reaching.application("application").cloned() else {
        // The verb declares the argument and `Reaching` found every application
        // the call named, so this is a declaration and an executor having
        // drifted apart. Said rather than unwrapped.
        return refused(said_about(
            strings,
            words::COULD_NOT_ASK,
            "this application",
            "",
        ));
    };
    let identifier = application.identifier().to_owned();

    // Step 2. The file: granted, there, and really where the grant is.
    let touching = match Touching::of(
        reaching.into_authorised(),
        grants,
        machine.resolving(),
        strings,
    ) {
        Ok(touching) => touching,
        Err(why) => {
            let entry = Entry::refused(&why, &agent, strings, at);
            return (entry, Err(NotDone::Refused(why)));
        }
    };
    let Some(file) = touching
        .real("path")
        .map(|real| real.as_path().to_path_buf())
    else {
        return refused(said_about(strings, words::COULD_NOT_ASK, &identifier, ""));
    };
    let shown = file.display().to_string();
    let authorised = touching.into_authorised();

    // Step 3. The line, with the real path in it.
    let Some(asked) = ToOpenAFile::with(application, &file) else {
        return refused(said_about(
            strings,
            words::NOT_A_FILE_TO_HAND_OVER,
            &identifier,
            &shown,
        ));
    };

    // Step 4. Ask the door, and say what happened in the person's own words.
    let answered = match machine.at_the_compositors_door() {
        Some(door) => door.opening_a_file(&asked),
        None => Err(NotAtTheDoor::NoCompositor),
    };
    let why = match answered {
        Ok(opened @ (Opened::InAlosCopy | Opened::InTheirWindow)) => {
            let where_it_opened = if matches!(opened, Opened::InAlosCopy) {
                words::OPENED_IN_ALOS_COPY
            } else {
                words::OPENED_IN_THEIR_WINDOW
            };
            let entry = Entry::ran(&authorised, strings).telling(&[said_about(
                strings,
                where_it_opened,
                &identifier,
                &shown,
            )]);
            return (
                entry,
                Ok(AFileOpened {
                    application: identifier,
                    file,
                    opened,
                }),
            );
        }
        Ok(Opened::NotInstalled) => {
            return refused(NotInstalled::wanting(&identifier).said(strings));
        }
        Ok(Opened::NoCopyOfItsOwn) => {
            return refused(strings.say(
                &alo_capability::words::NO_COPY_OF_ITS_OWN.key(),
                &Filling::of("application", identifier.clone()),
            ));
        }
        Ok(Opened::TakesNoFile) => words::TAKES_NO_FILE,
        Ok(Opened::CouldNotStart) => words::COULD_NOT_START_WITH_IT,
        // **A door that is not there and one that would not answer read the
        // same to a person**: nothing was opened. Which it was is a log's.
        Err(_) => words::COULD_NOT_ASK,
    };
    refused(said_about(strings, why, &identifier, &shown))
}

/// One of this verb's words, about this application and this file.
fn said_about(strings: &Strings, why: Word, application: &str, file: &str) -> Said {
    strings.say(
        &why.key(),
        &Filling::of("application", application.to_owned()).and("path", file.to_owned()),
    )
}
