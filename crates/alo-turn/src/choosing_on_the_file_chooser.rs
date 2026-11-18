//! **`choose_on_the_file_chooser`**: from *the person approved "choose
//! notes.txt in Documents on the file chooser"* to *the application that asked
//! was given notes.txt*.
//!
//! The coordinator's approval of 2026-11-17, on the owner's decision that alo
//! may answer alo's own file chooser **only inside folders the person
//! granted**. The order, and why each step is where it is:
//!
//! 1. **The folder: granted, there, and where it really leads granted too** —
//!    `alo_files::Touching`, the file verbs' own three questions. At full
//!    access the grants permit any folder the person's account can open (ADR
//!    0149), and that is decided where every grant is, not here.
//! 2. **The file, where it is already there: where it really leads granted
//!    too.** A name in a granted folder can be a link to somewhere nobody
//!    granted, and handing an application *that* would be the file verbs'
//!    refusal walked around. A file not there yet — a name to save under — is
//!    the folder's, and the folder was asked about.
//! 3. **The line**: the folder's **real** path and the name, never the folder
//!    the call arrived with.
//! 4. **Ask the door**, which answers the sheet that is up exactly as the
//!    person's own choice would — or says no sheet was up, or that the sheet
//!    could not have been answered with that file — and turn that into the
//!    person's own words.
//!
//! **Every refusal is written down, and every one means nothing was handed
//! over.** What the portal then records and hands to the application is the
//! portal's (`alo_portals`' `the_person_chooses_a_file`), as for any choice.
//!
//! **No indicator line and no boundary**, for `crate::acting_on_an_application`'s
//! reasons: nothing here leaves the machine, and no work of ours touches the
//! disk — the application opens the file under the person's own session.

use std::path::{Path, PathBuf};
use std::time::SystemTime;

use alo_acting::{AnsweredTheChooser, ToAnswerAFileChooser};
use alo_applications::choosing_on_the_file_chooser::NAME;
use alo_applications::words;
use alo_capability::{Authorised, Grantee, Grants, Refused, Value};
use alo_files::Touching;
use alo_record::Entry;
use alo_strings::{Filling, Said, Strings, Word};

use crate::machine::Machine;
use crate::refusing::NotDone;
use crate::the_door_the_compositor_serves::NotAtTheDoor;

/// A file alo chose on the file chooser.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AFileChosen {
    /// The application the sheet was up for, by its identifier.
    application: String,
    /// The file, as it is on this machine.
    file: PathBuf,
}

impl AFileChosen {
    /// The application the file was chosen for.
    #[must_use]
    pub fn application(&self) -> &str {
        &self.application
    }

    /// The file.
    #[must_use]
    pub fn file(&self) -> &Path {
        &self.file
    }
}

/// Carry out one approved `choose_on_the_file_chooser`.
///
/// Answers the record's entry and what happened, as every executor here does:
/// **a refusal is written down exactly as an execution is**.
pub(crate) fn choosing_on_the_file_chooser(
    machine: &mut Machine<'_>,
    authorised: Authorised,
    grants: &Grants,
) -> (Entry, Result<AFileChosen, NotDone>) {
    let strings = machine.strings();
    let at = authorised.at();
    let agent = authorised.under().clone();
    let call = authorised.call().clone();
    let refused = |said: Said| {
        let refused = Refused::worded_elsewhere(call.clone(), said);
        let entry = Entry::refused(&refused, &agent, strings, at);
        (entry, Err(NotDone::Refused(refused)))
    };
    let Some(Value::Name(name)) = call.value(NAME).cloned() else {
        // The verb declares the argument as a name, so this is a declaration
        // and an executor having drifted apart. Said rather than unwrapped.
        return refused(said_about(
            strings,
            words::NOT_A_FILE_TO_CHOOSE,
            Path::new(""),
        ));
    };

    // Step 1. The folder: granted, there, and really where the grant is.
    let touching = match Touching::of_looking_in(
        authorised,
        grants,
        machine.resolving(),
        strings,
        machine.the_persons_home(),
    ) {
        Ok(touching) => touching,
        Err(why) => {
            let entry = Entry::refused(&why, &agent, strings, at);
            return (entry, Err(NotDone::Refused(why)));
        }
    };
    let Some(folder) = touching
        .real(alo_applications::choosing_on_the_file_chooser::FOLDER)
        .map(|real| real.as_path().to_path_buf())
    else {
        return refused(said_about(
            strings,
            words::NOT_A_FILE_TO_CHOOSE,
            Path::new(&name),
        ));
    };
    let file = folder.join(&name);
    let authorised = touching.into_authorised();

    // Step 2. A file already there is asked about where it really leads.
    if let Some(why) = leads_where_nobody_granted(&file, grants, &agent, at, strings) {
        let refused = Refused::worded_elsewhere(call.clone(), why);
        let entry = Entry::refused(&refused, &agent, strings, at);
        return (entry, Err(NotDone::Refused(refused)));
    }

    // Step 3. The line, with the real folder in it.
    let Ok(asked) = ToAnswerAFileChooser::of(folder, &name) else {
        return refused(said_about(strings, words::NOT_A_FILE_TO_CHOOSE, &file));
    };

    // Step 4. Ask the door, and say what happened in the person's own words.
    let answered = match machine.at_the_compositors_door() {
        Some(door) => door.answering_the_file_chooser(&asked),
        None => Err(NotAtTheDoor::NoCompositor),
    };
    let why = match answered {
        Ok(AnsweredTheChooser::Chosen(application)) => {
            let identifier = application.identifier().to_owned();
            let told = strings.say(
                &words::CHOSEN_ON_THE_FILE_CHOOSER.key(),
                &Filling::of("path", file.display().to_string())
                    .and("application", identifier.clone()),
            );
            let entry = Entry::ran(&authorised, strings).telling(&[told]);
            return (
                entry,
                Ok(AFileChosen {
                    application: identifier,
                    file,
                }),
            );
        }
        Ok(AnsweredTheChooser::NoSheet) => words::NO_FILE_CHOOSER_UP,
        Ok(AnsweredTheChooser::NotBelieved) => words::THE_FILE_CHOOSER_WOULD_NOT_TAKE_IT,
        // **A door that is not there and one that would not answer read the
        // same to a person**: nothing was chosen. Which it was is a log's.
        Err(_) => words::COULD_NOT_ANSWER_THE_FILE_CHOOSER,
    };
    refused(said_about(strings, why, &file))
}

/// Where `file` is already there and really leads somewhere the grants do not
/// reach, the grants' own refusal, worded — [`None`] when it is not there yet
/// or leads somewhere granted.
fn leads_where_nobody_granted(
    file: &Path,
    grants: &Grants,
    agent: &Grantee,
    at: SystemTime,
    strings: &Strings,
) -> Option<Said> {
    let real = std::fs::canonicalize(file).ok()?;
    // **The file verbs' third question, in their own words**
    // (`alo_files::Touching`): against where each granted folder really is.
    if grants
        .permitted_where_it_really_is(agent, &real, at)
        .is_ok()
    {
        return None;
    }
    Some(
        strings.say(
            &alo_files::words::REALLY_LEADS_ELSEWHERE.key(),
            &Filling::of("path", file.display().to_string())
                .and("really", real.display().to_string())
                .and("who", agent.as_str()),
        ),
    )
}

/// One of this verb's words, about this file.
fn said_about(strings: &Strings, why: Word, file: &Path) -> Said {
    strings.say(&why.key(), &Filling::of("path", file.display().to_string()))
}

#[cfg(test)]
#[path = "choosing_on_the_file_chooser_tests.rs"]
mod tests;
