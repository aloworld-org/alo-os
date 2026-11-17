//! From *this may run* to *this is what happened*, and the entry that says so.
//!
//! A read and an approved change reach this point by different roads — one
//! answers inside the turn, the other waited for a person — and from here they
//! are the same four questions, asked in the same order:
//!
//! 1. **Where does every path this call names really lead, and do the grants
//!    cover that?** [`Touching`] asks it, at the authorisation's own moment.
//!    Containment is decided lexically and touches no disk, so a link out of a
//!    granted folder is caught here and nowhere else.
//! 2. **Where does this execution have to be able to reach?** [`Reaching`]
//!    answers it: the resolved paths, plus the folder above anything the call
//!    would create.
//! 3. **May anything this call would create be created?** [`Did`] asks it, and
//!    it is the one question asked *inside* the boundary. A rename invents a
//!    name and a move invents one inside a folder, and under a grant over a
//!    single file that name is one nobody granted.
//! 4. **And then the machine does it**, or says why it could not.
//!
//! # The boundary is around the verb and not around the entry
//!
//! Steps 3 and 4 happen inside a boundary the kernel imposes ([`Bounding`]) and
//! nothing else does. That is not tidiness in either direction:
//!
//! **Resolving is outside** because a reach is made of resolved paths, so a
//! thread inside a boundary would have to look up what it is not yet allowed to
//! open. **Writing the entry is outside** because the record is a file the
//! grants say nothing about, and a thread bounded across both would be refused
//! the evidence of what it had just done.
//!
//! # This file decides nothing and writes nothing down
//!
//! It answers with the entry and with what the caller gets, and
//! [`crate::Turning`] writes the first before handing over the second. That is
//! the crate's whole promise in one place rather than in three: **every road
//! out of here carries an entry**, so there is no branch a caller could take
//! that leaves nothing written.
//!
//! **That includes the road where there was no boundary**, and until
//! 2026-09-12 it did not. A turn that could not be bounded did not run — no
//! cgroup, no syscall, nothing asked of the disk and no grant consulted — and
//! this file argued that there was nothing true to write. There was: the
//! machine refused to run the agent's turn, which is a refusal, and a record
//! that kept every refusal but the machine's own showed a boundary that had
//! gone as a record that simply stopped. So the road out of a boundary that
//! could not be applied carries [`Entry::not_bounded`], with the sentence the
//! person was shown and the machine's own account of which pin was gone, and
//! [`crate::NotDone::NotBounded`] beside it — the same shape as
//! [`crate::NotDone::MachineCouldNot`], which is the other thing a machine
//! rather than a grant can say no to.
//!
//! The moment is taken off the [`Authorised`] before it is consumed, because it
//! is the moment the grants were asked and every entry about this call belongs
//! at it. A second reading of a clock would be a record saying the machine got
//! round to writing at a different time from the one it decided at.

use alo_capability::{Authorised, Grants};
use alo_files::{Reaching, Touching};
use alo_nearby::Origin;
use alo_record::Entry;

use crate::acting_on_an_application::acting_on_an_application;
use crate::arriving::worded_here;
use crate::asking_the_record::asking_the_record;
use crate::bounding::Doing;
use crate::choosing_in_an_application::choosing_in_an_application;
use crate::choosing_on_the_file_chooser::choosing_on_the_file_chooser;
use crate::installing_an_application::installing_an_application;
use crate::looking_at_the_catalogue::looking_at_the_catalogue;
use crate::machine::Machine;
use crate::opening_a_file_in_an_application::opening_a_file_in_an_application;
use crate::opening_a_menu_in_an_application::opening_a_menu_in_an_application;
use crate::opening_an_item_in_an_application::opening_an_item_in_an_application;
use crate::pressing_a_control::pressing_a_control;
use crate::publishing_a_folder::publishing_a_folder;
use crate::reading_an_applications_windows::reading_an_applications_windows;
use crate::refusing::NotDone;
use crate::said_in_an_application;
use crate::said_of_a_publication;
use crate::scrolling_in_an_application::scrolling_in_an_application;
use crate::setting_in_an_application::setting_in_an_application;
use crate::typing_into_an_application::typing_into_an_application;
use crate::what_a_verb_did::WhatAVerbDid;

/// The verbs `crate::acting_on_an_application` carries out.
///
/// Three of `alo-applications`' four. `arrange_application` is absent because
/// nothing can carry it out — `Machine::carrying_out_file_software_and_application_verbs`
/// does not declare it either, and the two absences have to agree.
const THE_APPLICATION_ACTS: [&str; 3] =
    ["open_application", "focus_application", "close_application"];

/// Carry an authorised call out inside a boundary, and say what happened.
///
/// The entry comes back on every road, including all three refusals: a call
/// the grants stopped at the last moment is a thing that happened, a call the
/// machine could not manage is one that was attempted and is recorded as one
/// (`alo-files`' rule, kept here), and a call the machine would not run
/// because it could not bound it is the machine's own refusal, written down
/// as such.
///
/// `origin` is the machine the call came from when it came from another one
/// (ADR 0003), and it changes one thing here: the two refusals the grants can
/// make about a resolved path are worded for the machine that was asked, as
/// they are at every other door. The boundary, the reach and the work are
/// exactly what they are for a local call — ADR 0013 applies on the receiving
/// machine exactly as it does here.
///
/// **An act in an application is also said on the journal**, done or not
/// (`crate::said_in_an_application`), so a walk can check it ran — and so
/// is a publication, published or not (`crate::said_of_a_publication`).
pub(crate) fn carrying_out(
    machine: &mut Machine<'_>,
    authorised: Authorised,
    grants: &Grants,
    origin: Option<&Origin>,
    now: std::time::SystemTime,
) -> (Entry, Result<WhatAVerbDid, NotDone>) {
    let in_an_application = said_in_an_application::about(&authorised);
    let a_publication = said_of_a_publication::is_one(authorised.verb());
    let (entry, did) = carrying_it_out(machine, authorised, grants, origin, now);
    if let Some((verb, application)) = in_an_application {
        eprintln!(
            "alo-agentd: {}",
            said_in_an_application::said(&verb, &application, &did)
        );
    }
    if a_publication {
        eprintln!("alo-agentd: {}", said_of_a_publication::said(&did));
    }
    (entry, did)
}

/// [`carrying_out`], without the journal's line.
fn carrying_it_out(
    machine: &mut Machine<'_>,
    authorised: Authorised,
    grants: &Grants,
    origin: Option<&Origin>,
    now: std::time::SystemTime,
) -> (Entry, Result<WhatAVerbDid, NotDone>) {
    // **Which executor, decided by the verb's own name and nothing else.**
    // The alternative is asking the arguments what shape they are, which is a
    // second opinion about what a call is: `alo_capability` already decided
    // that when the call was validated against the declaration, and a verb
    // routed by anything but its name could be routed one way here and another
    // way in the record.
    if authorised.verb() == alo_software::INSTALL_APPLICATION {
        let (entry, installed) = installing_an_application(machine, authorised, now);
        return (entry, installed.map(WhatAVerbDid::from));
    }
    if authorised.verb() == alo_publishing::PUBLISH_TO_GITHUB_PAGES {
        let (entry, published) = publishing_a_folder(machine, authorised, now);
        return (entry, published.map(WhatAVerbDid::from));
    }
    // **The read, routed by name like the rest.** It is asked before the
    // boundary is worked out rather than after, because there is nothing for a
    // boundary to be made of: ADR 0013's is built from resolved paths and this
    // verb reaches none. Taking `&*machine` rather than `&mut` says in the
    // signature what the file says in prose — nothing on the machine moves.
    if authorised.verb() == alo_software::WHAT_CAN_BE_INSTALLED {
        let (entry, holds) = looking_at_the_catalogue(machine, authorised);
        return (entry, holds.map(WhatAVerbDid::from));
    }
    // **The record's question, routed by name like the rest.** It is asked
    // before the boundary is worked out, for `WHAT_CAN_BE_INSTALLED`'s reason
    // one arm up: ADR 0013's boundary is built from resolved paths and this
    // verb opens none — the path it carries is matched against what was
    // written down, as text. Taking `&*machine` rather than `&mut` says in the
    // signature what the file says in prose: nothing on the machine moves.
    if authorised.verb() == alo_record::what_touched_a_file::THE_VERB {
        let (entry, found) = asking_the_record(machine, authorised);
        return (entry, found.map(WhatAVerbDid::from));
    }
    // **The three application acts, routed by name for the same reason.** The
    // fourth, `arrange_application`, is not declared by the constructor that
    // offers these, so a call for it is never authorised and never arrives —
    // and `acting_on_an_application` refuses it by name anyway, because a list
    // and an executor are two edits and somebody will one day make only one.
    // **The accessibility fallback's two, routed by name like the rest**, and
    // before the boundary for the reason the record's question is: ADR 0013's
    // boundary is built from resolved paths, and neither verb opens a path —
    // what they reach is the person's accessibility tree, opened for the one
    // act and dropped with it.
    if authorised.verb() == alo_adapters::READ_WINDOW {
        let (entry, read) = reading_an_applications_windows(machine, authorised, grants);
        return (entry, read.map(WhatAVerbDid::from));
    }
    if authorised.verb() == alo_adapters::ACTIVATE_CONTROL {
        let (entry, pressed) = pressing_a_control(machine, authorised, grants);
        return (entry, pressed.map(WhatAVerbDid::from));
    }
    if authorised.verb() == alo_adapters::TYPE_TEXT {
        let (entry, typed) = typing_into_an_application(machine, authorised, grants);
        return (entry, typed.map(WhatAVerbDid::from));
    }
    if authorised.verb() == alo_adapters::SCROLL_TO
        || authorised.verb() == alo_adapters::SCROLL_PAGE
    {
        let (entry, scrolled) = scrolling_in_an_application(machine, authorised, grants);
        return (entry, scrolled.map(WhatAVerbDid::from));
    }
    // **Moving a slider or a number box** (2026-11-17), before the boundary
    // for the same reason.
    if authorised.verb() == alo_adapters::SET_VALUE {
        let (entry, set) = setting_in_an_application(machine, authorised, grants);
        return (entry, set.map(WhatAVerbDid::from));
    }
    // **Choosing in an application** (2026-11-17): an option of a drop-down or
    // an item of a list, before the boundary for the same reason.
    if authorised.verb() == alo_adapters::CHOOSE_OPTION
        || authorised.verb() == alo_adapters::SELECT_ITEM
    {
        let (entry, chosen) = choosing_in_an_application(machine, authorised, grants);
        return (entry, chosen.map(WhatAVerbDid::from));
    }
    // **Opening an item** (2026-11-17), before the boundary for the same
    // reason.
    if authorised.verb() == alo_adapters::OPEN_ITEM {
        let (entry, opened) = opening_an_item_in_an_application(machine, authorised, grants);
        return (entry, opened.map(WhatAVerbDid::from));
    }
    // **A thing's own menu** (2026-11-17), before the boundary for the same reason.
    if authorised.verb() == alo_adapters::OPEN_MENU {
        let (entry, opened) = opening_a_menu_in_an_application(machine, authorised, grants);
        return (entry, opened.map(WhatAVerbDid::from));
    }
    // **Opening a file in an application, routed by name like the rest**
    // (2026-11-14). Before the boundary, for the accessibility verbs' reason:
    // the path it carries is resolved and asked about by `alo_files::Touching`
    // inside the executor, and what opens it is the person's application under
    // their own session — no work of ours touches the disk.
    // **Answering the file chooser, routed by name like the rest**
    // (2026-11-17), before the boundary for the same reason: its folder is
    // resolved and asked about by `alo_files::Touching` inside the executor,
    // and no work of ours touches the disk.
    if authorised.verb() == alo_applications::CHOOSE_ON_THE_FILE_CHOOSER {
        let (entry, chosen) = choosing_on_the_file_chooser(machine, authorised, grants);
        return (entry, chosen.map(WhatAVerbDid::from));
    }
    if authorised.verb() == alo_applications::OPEN_FILE_IN_APPLICATION {
        let (entry, opened) = opening_a_file_in_an_application(machine, authorised, grants);
        return (entry, opened.map(WhatAVerbDid::from));
    }
    if THE_APPLICATION_ACTS.contains(&authorised.verb()) {
        let (entry, reached) = acting_on_an_application(machine, authorised, grants);
        return (entry, reached.map(WhatAVerbDid::from));
    }
    let at = authorised.at();
    let agent = authorised.under().clone();
    let strings = machine.strings();

    let touching = match Touching::of_looking_in(
        authorised,
        grants,
        machine.resolving(),
        strings,
        machine.the_persons_home(),
    ) {
        Ok(touching) => touching,
        Err(refused) => {
            let refused = worded_here(refused, origin, strings);
            let entry = Entry::refused(&refused, &agent, strings, at);
            return (entry, Err(NotDone::Refused(refused)));
        }
    };

    // Everywhere the work will have to open, worked out before the work can
    // open anything. A call of the six always reaches at least one place, so
    // this refuses only a verb something else declared — and it is the same
    // answer, and the same entry, as the machine failing at it a moment later.
    let reaching = match Reaching::of(&touching) {
        Ok(reaching) => reaching,
        Err(failed) => {
            let authorised = touching.into_authorised();
            return (
                Entry::ran(&authorised, strings),
                Err(NotDone::MachineCouldNot(failed)),
            );
        }
    };

    let done = match machine
        .bounding()
        .carrying_out(&reaching, Doing::of(touching, grants, strings))
    {
        Ok(done) => done,
        Err(no_boundary) => {
            let said = no_boundary.said(strings);
            let entry = Entry::not_bounded(&agent, said.text(), no_boundary.why(), at);
            return (entry, Err(NotDone::NotBounded(no_boundary)));
        }
    };
    let did = match done {
        Ok(did) => did,
        Err(refused) => {
            let refused = worded_here(refused, origin, strings);
            let entry = Entry::refused(&refused, &agent, strings, at);
            return (entry, Err(NotDone::Refused(refused)));
        }
    };

    let (authorised, outcome) = did.into_parts();
    // **What the machine said, when it said no**, kept with the attempt
    // (`Entry::telling`). Since 2026-11-08 a change at full access runs with
    // nobody shown it first (ADR 0149), so the record is the one place a person
    // can read afterwards that it was tried and *not* done: an attempt with
    // nothing told beside it read exactly like one that worked.
    let entry = match &outcome {
        Ok(_) => Entry::ran(&authorised, strings),
        Err(failed) => Entry::ran(&authorised, strings).telling(&[failed.said(strings)]),
    };
    (
        entry,
        outcome
            .map(WhatAVerbDid::from)
            .map_err(NotDone::MachineCouldNot),
    )
}
