//! What a verb did, turned into what crosses the socket.
//!
//! Two lines of match, in its own file, because of where it has to live.
//! `alo-protocol` holds `alo-turn` as a **dev**-dependency on purpose: the wire
//! format does not know what a turn is, so a client library never has to build
//! the orchestrator to read a message. That leaves `alo_protocol::Done` with
//! one constructor per family of verbs and nobody inside that crate able to
//! choose between them.
//!
//! This daemon can choose: it is the join, and it depends on both. Two of its
//! files answer a person who approved a change — `crate::answering` for one on
//! this machine and `crate::reaching` for one that arrived from another — and
//! without this they would each carry their own copy of the same decision.
//!
//! # When a third family arrives, the compiler says so here
//!
//! The match is total and there is no `_` arm. That is the whole reason
//! `alo_turn::WhatAVerbDid` is a closed enum rather than a trait: a family that
//! answered with something the wire cannot carry would be the same fault as a
//! verb nothing can execute, one level along, and this is where it is caught.

use std::time::SystemTime;

use alo_protocol::Done;
use alo_turn::WhatAVerbDid;

/// What a verb did, as the wire carries it.
pub(crate) fn as_a_client_sees_it(did: &WhatAVerbDid, now: SystemTime) -> Done {
    match did {
        WhatAVerbDid::ToFiles(answer) => Done::of_a_file_verb(answer),
        WhatAVerbDid::AnApplicationInstalled(installed) => Done::of_an_installation(installed),
        // The third family, arriving exactly where this file's header said it
        // would. The two words are handed over rather than the turn's type,
        // because `alo-protocol` must not learn what a turn is.
        WhatAVerbDid::AnApplicationWasReached(reached) => Done::of_an_application_act(
            reached.acting().on_the_wire(),
            reached.acting().application().identifier(),
        ),
        // The fourth family, and **the first that is a read**. Every one before
        // it answered a person who had approved something; this one answers
        // inside the turn, so the two files named in this header are not the
        // only ones that reach it — `crate::doing` does, which is the whole
        // point of a read.
        WhatAVerbDid::WhatCanBeInstalled(holds) => Done::of_a_catalogue_read(holds),
        // The fifth family, and the second read. **The first whose subject is
        // the person themselves**, which is why it takes `now`: what crosses
        // is how long ago each thing happened, not a moment, so what a person
        // reads does not depend on two clocks agreeing.
        //
        // It goes to a client on the machine whose record it is and nowhere
        // else — `alo_turn::Arriving::reading` refuses the verb across a
        // pairing, and `alo_corridor::doorway` answers nothing for this arm.
        WhatAVerbDid::WhatTouchedAFile(found) => Done::of_a_record_read(found, now),
        // The accessibility fallback's two (2026-11-12): a read of what an
        // application's windows show, answered inside the turn, and a press a
        // person approved.
        WhatAVerbDid::WhatTheWindowsShowed(read) => {
            Done::of_a_window_read(read.shown(), read.told())
        }
        WhatAVerbDid::AControlWasPressed(pressed) => Done::of_a_press(
            pressed.application(),
            pressed.kind(),
            pressed.name(),
            pressed.unanswered(),
        ),
        // A typing that saves nothing (2026-11-17) says so on the wire, so a
        // client never reads it as a save that failed.
        WhatAVerbDid::TextWasTyped(typed) => Done::of_either_typing(
            typed.application(),
            typed.name(),
            typed.characters(),
            typed.through_the_tree(),
            typed.asked_to_save(),
            typed.read_back(),
            typed.saved(),
        ),
        // A slider or a number box moved (2026-11-17).
        WhatAVerbDid::AValueWasSet(set) => Done::of_a_setting(
            set.application(),
            set.name(),
            set.value(),
            set.seen(),
            set.unanswered(),
        ),
        // A choice in an application (2026-11-17): an option of a drop-down
        // or an item of a list.
        WhatAVerbDid::SomethingWasChosen(chosen) => Done::of_a_choice(
            chosen.application(),
            chosen.name(),
            chosen.option(),
            chosen.seen(),
            chosen.unanswered(),
        ),
        // An item opened, as a double-click would (2026-11-17).
        WhatAVerbDid::AnItemWasOpened(opened) => {
            Done::of_an_item_opened(opened.application(), opened.name(), opened.unanswered())
        }
        // The seventh family: a folder sent to the person's GitHub Pages site
        // (2026-11-13). The folder and the address cross as text, so the wire
        // need not learn what a publication is.
        WhatAVerbDid::AFolderWasPublished(published) => Done::of_a_publication(
            &published.folder().display().to_string(),
            published.address(),
            published.nothing_new(),
        ),
        // The eighth: a file opened in an application the person named
        // (2026-11-14). It crosses as an application act, `open-file`, which
        // is the compositor door's own word for it; the file and where it
        // opened are in the record.
        WhatAVerbDid::AFileWasOpened(opened) => Done::of_a_file_opened(
            alo_acting::OPEN_FILE,
            opened.application(),
            opened
                .types_into()
                .map(|(name, window)| (name.to_owned(), window.to_owned())),
        ),
        // A file chosen on the file chooser an application had up
        // (2026-11-17). It crosses as an application act, the door's own word
        // for it, naming whom it was chosen for; the file is in the record.
        WhatAVerbDid::AFileWasChosen(chosen) => {
            Done::of_an_application_act(alo_acting::ALO_CHOOSES_FILE, chosen.application())
        }
        // A scroll in an application's window (2026-11-17): what was asked
        // and whether it now reads as on the screen — never what it shows.
        WhatAVerbDid::AWindowWasScrolled(scrolled) => Done::of_a_scroll(
            scrolled.application(),
            scrolled.to(),
            scrolled.a_page(),
            scrolled.now_showing(),
        ),
        // One thing's right-click menu opened (2026-11-17): what it holds is
        // read by `accessible.read_window`, never carried here.
        WhatAVerbDid::AMenuWasOpened(opened) => {
            Done::of_a_menu(opened.application(), opened.name(), opened.unanswered())
        }
        // One key pressed in an application's window (2026-11-17).
        WhatAVerbDid::AKeyWasPressed(pressed) => {
            Done::of_a_key(pressed.application(), pressed.key().named())
        }
        // A picture is never carried: `crate::reading_a_picture` has it read
        // first, and one that reaches here was read by nothing.
        WhatAVerbDid::APictureWasTaken(taken) => {
            Done::of_a_picture_not_read(taken.application(), taken.size())
        }
        WhatAVerbDid::AClickWasMade(made) => {
            Done::of_a_click(made.application(), made.description(), made.changed(), None)
        }
        // **A click still to be found is never told from here**: both roads that
        // approve a change hand it to `crate::clicking_where_it_is` first, and
        // the turn writes one never found down as refused when it ends. Were it
        // reached, it is told as a click whose outcome is not known.
        WhatAVerbDid::AThingIsToBeFound(thing) => {
            Done::of_a_click(thing.application(), thing.description(), false, None)
        }
    }
}
