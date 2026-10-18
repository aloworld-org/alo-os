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
    }
}
