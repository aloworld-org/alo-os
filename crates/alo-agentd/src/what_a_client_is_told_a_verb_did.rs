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

use alo_protocol::Done;
use alo_turn::WhatAVerbDid;

/// What a verb did, as the wire carries it.
pub(crate) fn as_a_client_sees_it(did: &WhatAVerbDid) -> Done {
    match did {
        WhatAVerbDid::ToFiles(answer) => Done::of_a_file_verb(answer),
        WhatAVerbDid::AnApplicationInstalled(installed) => Done::of_an_installation(installed),
    }
}
