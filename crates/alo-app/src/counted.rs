//! **What the application sends while one act runs**, counted by the desktop
//! and written down beside the act
//! ([ADR 0148](../../../docs/decisions/0148-the-kernel-counts-what-an-application-sends-during-a-persons-command.md)).
//!
//! `alo-app` itself sends nothing across the network; what leaves is sent by
//! the application, from its own process. So around every act a count of that
//! application is held open on the desktop's door, and the indicator shows
//! *… is sending, during your command* while it is. After the act the count is
//! ended and its number goes into the record. **The command never reports an
//! egress of its own** (ADR 0125): it writes down the number the service that
//! showed it answered.
//!
//! # When it cannot be counted, the command still runs
//!
//! A machine with no counter, a desktop that is not answering, a count the
//! desktop has no room for: in each of these the act goes ahead, the person is
//! told on the error output, in one plain sentence, that alo cannot show what
//! that application sends during this command, and the record keeps that same
//! sentence. **This is a choice put to the owner rather than settled**: the
//! alternative is to refuse the command, which law 5 makes the person's
//! choice and not this file's.

use alo_acting::{Counted, ToCount};
use alo_record::{Line, SentDuring};
use alo_strings::{Filling, Strings};

use crate::machine::Machine;
use crate::subject::Subject;
use crate::words;

/// Carry out `act` with a count held around it, and say what was sent.
pub fn counting<T>(
    subject: &Subject,
    strings: &Strings,
    machine: &mut dyn Machine,
    act: impl FnOnce(&mut dyn Machine) -> T,
) -> (T, SentDuring) {
    let Some(application) = subject.application() else {
        // alo OS's own surfaces: what alo OS sends is on the indicator already.
        let why = strings.say(&words::ALOS_OWN_NOT_COUNTED.key(), &Filling::nothing());
        return (
            act(machine),
            SentDuring::NotCounted {
                why: Line::of(why.text()),
            },
        );
    };
    let held = machine
        .door()
        .begin_counting(&ToCount::what(application.clone()));
    let filling = Filling::of(words::APPLICATION, subject.called.clone());
    let Some(held) = held else {
        let why = strings.say(&words::CANNOT_COUNT.key(), &filling);
        machine.warned(why.text());
        return (
            act(machine),
            SentDuring::NotCounted {
                why: Line::of(why.text()),
            },
        );
    };
    let done = act(machine);
    let sent = match held.done() {
        Some(Counted::Sent(bytes)) => SentDuring::Counted { bytes },
        Some(Counted::Stopped(bytes)) => SentDuring::AtLeast { bytes },
        None => SentDuring::NotCounted {
            why: Line::of(strings.say(&words::COUNT_UNENDED.key(), &filling).text()),
        },
    };
    (done, sent)
}
