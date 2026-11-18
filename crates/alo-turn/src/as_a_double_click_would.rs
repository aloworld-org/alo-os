//! **A file opened *in* a file manager opens as a double-click there would**
//! — the owner's decision of 2026-11-17: *when a person asks alo to open a file
//! "in" a file manager, that means what a double-click there means; the file
//! opens in the application that file type opens with.*
//!
//! So an `open_file_in_application` naming an application this machine opens
//! folders with (`crate::SaysWhatAFileOpensWith::opens_folders`), for a path
//! that is a **file**, is made into the call a double-click would be: the same
//! verb and the same file, **in the application the file opens with**, saying
//! *open report.pdf in org.gnome.Papers, as org.kde.dolphin would*. A folder
//! still opens in the file manager, which is what a double-click on a folder
//! does there.
//!
//! **Made here, before anything else is asked of the call**, so everything
//! after it is about the application that will open the file: the grants are
//! asked about it, the person approves a sentence naming it, the record says
//! it ran there, and at full access the same. Nothing about this road is a
//! second executor — what runs is an ordinary `open_file_in_application`.
//!
//! - **Nothing opens a file of that kind**: refused, recorded, in
//!   `alo_applications`' own words, and nothing is opened.
//! - **The application it opens with is not allowed for alo**: noted as wanted
//!   (`Turning::wanting`), so the grants' refusal pauses the turn and the
//!   person's shell asks whether alo may use that application — the sheet, not
//!   a dead end (the owner's decision of 2026-11-16).
//!
//! **What is read to decide is read only once the file manager itself is
//! allowed**: a call naming an application nobody granted is left as it came,
//! for the grants to refuse, so no file's kind is ever read for a call that
//! could not have reached it.

use std::time::SystemTime;

use alo_applications::opening_a_file::PATH;
use alo_applications::verbs::APPLICATION;
use alo_applications::{OPEN_FILE_IN_APPLICATION, words};
use alo_capability::{Ask, Call, Given, Grants, Refused, Value};
use alo_record::Entry;

use crate::refusing::NotDone;
use crate::turning::Turning;

impl Turning<'_, '_> {
    /// `call`, or the call a double-click in the file manager it names would
    /// be (this file's header).
    ///
    /// # Errors
    /// [`NotDone::Refused`] where nothing opens a file of that kind, recorded.
    pub(crate) fn as_a_double_click_would(
        &mut self,
        call: Call,
        grants: &Grants,
        now: SystemTime,
    ) -> Result<Call, NotDone> {
        if call.verb() != OPEN_FILE_IN_APPLICATION {
            return Ok(call);
        }
        let (Some(Value::Application(manager)), Some(Value::Path(file))) =
            (call.value(APPLICATION), call.value(PATH))
        else {
            return Ok(call);
        };
        let Some(opening) = self.machine().what_files_open_with() else {
            return Ok(call);
        };
        let grantee = self.grantee().clone();
        if !opening.opens_folders(manager)
            || grants
                .permitted(&grantee, &Ask::Application(manager.clone()), now)
                .is_err()
        {
            return Ok(call);
        }
        // **Only a file**: a folder opens in the file manager, and a path where
        // nothing is is refused later, in the words every verb uses for it.
        let Ok(real) = self.machine().resolving().real(file) else {
            return Ok(call);
        };
        if opening.is_a_folder(real.as_path()) {
            return Ok(call);
        }
        let opener = match opening.opens_with(real.as_path()) {
            Ok(opener) => opener,
            Err(nothing) => {
                let refused = Refused::worded_elsewhere(call, nothing.said().clone());
                return self.stopped_under(refused, grants, now);
            }
        };
        let manager = manager.clone();
        let file = file.display().to_string();
        let instead = match self.machine().verbs().call(
            OPEN_FILE_IN_APPLICATION,
            &[
                (APPLICATION, Given::text(opener.clone())),
                (PATH, Given::text(file)),
            ],
        ) {
            Ok(instead) => instead,
            // An identifier the machine itself answered that the verb will not
            // take is a reading and a declaration drifted apart: said, never
            // unwrapped, and recorded like any other turning away.
            Err(why) => {
                let said = why.said(self.machine().strings()).into_text();
                let entry = Entry::turned_away(OPEN_FILE_IN_APPLICATION, &said, &grantee, now);
                self.writing_down(entry)?;
                return Err(NotDone::TurnedAway(why));
            }
        };
        // **The application it opens with, wanted where it is not allowed**, so
        // the grants' refusal that follows puts the sheet up for it.
        if grants
            .permitted(&grantee, &Ask::Application(opener.clone()), now)
            .is_err()
        {
            self.wanting(vec![opener]);
        }
        Ok(instead.saying_instead(
            words::OPEN_FILE_AS_A_FILE_MANAGER_WOULD.key(),
            &[("file_manager", manager)],
        ))
    }
}

// Tested through a whole turn, with the door and the grants, in
// `opening_a_file_in_an_application_tests.rs`: *as a file manager would*.
