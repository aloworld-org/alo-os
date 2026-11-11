//! The words on a task's card and square in the right-hand panel.
//!
//! The owner approved the design `07 / A running task in the put-aside panel`
//! (`23:2`) on 2026-11-11 with *"Approve as drawn"*, and these are its words.
//! They are **not** the task window's words, on purpose: the window says
//! *Waiting for your permission* across a whole title row, and a card 216 wide
//! says *Needs your OK*. Both are listed in `crate::words::EVERY_WORD`, so one
//! vocabulary still holds every word this crate can say.
//!
//! Two of them are not in the drawing, and say so in their notes:
//! *Stopped · nothing changed*, because *changes kept* would be false for a
//! task stopped before it changed anything; and *Nothing approved yet*, for the
//! fourth line of a task whose person has not approved anything.

use alo_strings::Word;

/// The card's first line while the task waits on the person.
pub const NEEDS_YOUR_OK: Word = Word::saying("tasks.card.needs-your-ok", "Needs your OK").noting(
    "On a task's card in the right-hand panel, beside alo's mark, while alo waits for the \
     person to approve a change. **A request, not a status** — the card's short form of \
     `tasks.status.waiting`. Drawn in navy, never teal: alo is not acting.",
);

/// The card's first line after a confirmed Stop that kept changes.
pub const STOPPED_CHANGES_KEPT: Word =
    Word::saying("tasks.card.stopped-changes-kept", "Stopped \u{b7} changes kept").noting(
        "On a task's card after Stop was confirmed and the task had already changed \
         something. **Stopping is not undoing**: the second half tells the person the changes \
         are still there, and a translation must keep it.",
    );

/// The card's first line after a confirmed Stop that changed nothing.
pub const STOPPED_NOTHING_CHANGED: Word =
    Word::saying("tasks.card.stopped-nothing-changed", "Stopped \u{b7} nothing changed")
        .noting(
            "On a task's card after Stop was confirmed and the task had changed nothing. Not \
             in the approved drawing, which shows only a task that had changed something; \
             *changes kept* would be false here, so this says so instead.",
        );

/// The card's first line when the task finished on its own.
pub const DONE: Word = Word::saying("tasks.card.done", "Done").noting(
    "On a task's card and under its square in the right-hand panel, when the task finished \
     on its own. Drawn in green. Not *stopped*: nobody stopped it.",
);

/// Line three while a Stop is on its way.
pub const WAITING_FOR_ALO_TO_CONFIRM: Word =
    Word::saying("tasks.card.waiting-to-confirm", "Waiting for alo to confirm").noting(
        "Under the task's name on its card, after the person pressed Stop and before alo has \
         confirmed the work ended. Says the work may still be running.",
    );

/// Line three after a confirmed Stop, counting what it kept.
pub const CHANGES_KEPT_BEFORE_STOP: Word =
    Word::saying("tasks.card.changes-kept", "Changes kept before Stop: {count}").noting(
        "Under the task's name on its card once Stop is confirmed. `{count}` is how many \
         changes the person had approved and alo had made before it stopped, as a number.",
    );

/// Line three after a confirmed Stop that changed nothing.
pub const NOTHING_CHANGED_BEFORE_STOP: Word =
    Word::saying("tasks.card.nothing-changed", "Nothing was changed before Stop").noting(
        "Under the task's name on its card once Stop is confirmed, for a task that had changed \
         nothing. A real answer, said rather than left blank.",
    );

/// Line three when the Stop could not be delivered.
pub const CANNOT_CONFIRM_IT_STOPPED: Word =
    Word::saying("tasks.card.cannot-confirm", "This machine cannot confirm it stopped").noting(
        "Under the task's name on its card when the person pressed Stop and this machine \
         could not deliver it. **It is not saying the work stopped**; it may still be running.",
    );

/// Line three when a task finished with an answer.
pub const ANSWER_READY: Word = Word::saying("tasks.card.answer-ready", "Answer ready").noting(
    "Under the task's name on its card when alo finished and its answer is waiting in the \
     task window. Selecting the card opens it.",
);

/// Line four: the last change the person approved.
pub const LAST_APPROVED: Word =
    Word::saying("tasks.card.last-approved", "Last approved: {what}").noting(
        "The fourth line of a task's card. `{what}` is the sentence of the last change the \
         person approved in this task, exactly as it was put to them.",
    );

/// Line four when the person has approved nothing in this task.
pub const NOTHING_APPROVED_YET: Word =
    Word::saying("tasks.card.nothing-approved", "Nothing approved yet").noting(
        "The fourth line of a task's card when the person has not approved any change in this \
         task. Not in the approved drawing, which shows only tasks that had one.",
    );

/// Line four after a Stop that kept changes.
pub const UNDO_IS_IN_HISTORY: Word =
    Word::saying("tasks.card.undo-in-history", "Undo is in History").noting(
        "The fourth line of a task's card after a confirmed Stop that kept changes. Tells the \
         person where to take them back, because stopping does not undo them. *History* is \
         the name of the place on the Dock.",
    );

/// The card's button while alo works.
pub const STOP: Word = Word::saying("tasks.card.stop", "Stop").noting(
    "The button on a task's card in the right-hand panel while alo works. Stops the work, \
     not the application, and asks nothing first. Keep it one short word.",
);

/// The card's button while alo waits on the person.
pub const REVIEW: Word = Word::saying("tasks.card.review", "Review").noting(
    "The button on a task's card while alo waits for the person's OK. Opens the task window \
     at the change waiting to be approved. Keep it one short word.",
);

/// The card's button once the work is over, or its Stop is unconfirmed.
pub const OPEN: Word = Word::saying("tasks.card.open", "Open").noting(
    "The button on a task's card once it is done or stopped. Opens the task window. Keep it \
     one short word.",
);

/// Under a working task's square, on focus or hover.
pub const RAIL_WORKING: Word = Word::saying("tasks.rail.working", "Working").noting(
    "Under a task's square in the narrow right-hand rail, on focus or hover, while alo works. \
     One word: the square is 48 wide.",
);

/// Under a waiting task's square, on focus or hover.
pub const RAIL_NEEDS_YOU: Word = Word::saying("tasks.rail.needs-you", "Needs you").noting(
    "Under a task's square in the narrow right-hand rail when alo waits on the person. Short: \
     the square is 48 wide. Must stay clearly different from `tasks.rail.working`.",
);

/// Under a stopped task's square, on focus or hover.
pub const RAIL_STOPPED: Word = Word::saying("tasks.rail.stopped", "Stopped").noting(
    "Under a task's square in the narrow right-hand rail after Stop was confirmed. Only after \
     — never while it is still stopping.",
);

/// What a screen reader says for a task's square.
pub const RAIL_NAME: Word = Word::saying("tasks.rail.name", "alo task, {state}: {task}").noting(
    "What a screen reader announces for a task's square in the right-hand rail. `{state}` is \
     the word under the square (*working*, *needs you*…), `{task}` the task in the person's \
     own words. The state is never told by colour alone.",
);
