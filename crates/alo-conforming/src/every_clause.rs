//! **Every clause of the standard that applies to alo OS's shell**, and where
//! each one stands.
//!
//! Three parts of the standard reach a shell that is not a web page and not a
//! document:
//!
//! - **Clause 5, generic requirements** — what any piece of information and
//!   communications technology must do, whatever it is;
//! - **Clause 11, software** — the one this shell is, and the one that carries
//!   the accessibility criteria a reader knows from WCAG, renumbered for
//!   software as `11.1` to `11.4`;
//! - and **clause 11.5**, which is the part a desktop shell lives or dies by:
//!   what it must hand an assistive technology through the platform's own
//!   accessibility services.
//!
//! **Clause 9 is the web**, and the plan's own wording — *the parts of 9 that
//! apply to a native shell's text* — is worth correcting here rather than
//! quietly following: the criteria it means reach a native shell through
//! `11.1`–`11.4`, which is where they are listed below. A shell that drew web
//! pages would be answering clause 9 as well; this one does not.
//!
//! # What is here and what is left out
//!
//! Clause 12 (documentation and support services) and clause 13 (relay and
//! emergency services) are outside this task's acceptance and outside v0.5:
//! nobody has written the documentation a person is owed yet, and this machine
//! makes no calls. They are **not** listed as not applicable, because they are
//! not: they are *not yet looked at*, which is a different thing and belongs in
//! the plan that takes them.
//!
//! Sub-clauses of closed functionality (`5.1`) are held as one row rather than
//! listed one by one. This shell has a screen, a keyboard and an accessibility
//! tree; none of it is closed functionality, and listing a dozen numbers to
//! dismiss them all with the same sentence would be more precision than this
//! lane can honestly claim about numbers it has not read.

use crate::clause::Clause;
use crate::standard::THE_PLAN;
use crate::standing::{Checked::NotAgainstTheText, Evidence, Standing, Waiting};

/// The desktop plan, where the shell's own surfaces are drawn.
const THE_DESKTOP_PLAN: &str = "docs/autonomy/v0-5-hands-on-the-desktop-plan.md";

/// The documents plan, where converting a document lives.
const THE_DOCUMENTS_PLAN: &str = "docs/autonomy/v0-5-documents-and-paper-plan.md";

/// A clause met by a test in this workspace.
const fn met(crate_named: &'static str, file: &'static str, test: &'static str) -> Standing {
    Standing::Met {
        by: Evidence {
            crate_named,
            file,
            test,
        },
    }
}

/// A clause waiting on a task somebody has written down.
const fn not_yet(task: u32, plan: &'static str) -> Standing {
    Standing::NotYet {
        because: Waiting { task, plan },
    }
}

/// A clause this machine is not the kind of machine for.
const fn not_applicable(because: &'static str) -> Standing {
    Standing::NotApplicable { because }
}

/// Every clause that applies, in the standard's own order.
pub const THE_CLAUSES: [Clause; 46] = [
    Clause {
        number: "5.1",
        requirement: "Technology that does not let an assistive technology be attached to it must \
                      do the whole job by itself.",
        standing: not_applicable(
            "this shell is not closed functionality: it has a screen, a keyboard, and an \
             accessibility tree anything may attach to",
        ),
        checked: NotAgainstTheText,
    },
    Clause {
        number: "5.2",
        requirement: "A person must be able to turn the accessibility features on without needing \
                      the features they are turning on.",
        standing: met(
            "alo-access",
            "tests/each_setting_changes_one_thing.rs",
            "every_setting_can_be_turned_on_where_there_is_no_account_yet",
        ),
        checked: NotAgainstTheText,
    },
    Clause {
        number: "5.3",
        requirement: "Where biometrics are the only way in, there must be another way in.",
        standing: not_applicable("nothing on this machine identifies anybody by a biometric"),
        checked: NotAgainstTheText,
    },
    Clause {
        number: "5.4",
        requirement: "Converting a document must carry the accessibility information in it across \
                      to the copy.",
        standing: not_yet(2, THE_DOCUMENTS_PLAN),
        checked: NotAgainstTheText,
    },
    Clause {
        number: "5.5.1",
        requirement: "Anything a person has to grip, pinch or twist must be usable another way.",
        standing: not_applicable(
            "this is software on a machine somebody else makes; the parts a person grips are that \
             machine's, and the certified laptop answers this clause rather than this repository",
        ),
        checked: NotAgainstTheText,
    },
    Clause {
        number: "5.5.2",
        requirement: "A part a person operates must be findable without seeing it.",
        standing: not_applicable("as 5.5.1: the machine's own parts, not this shell's"),
        checked: NotAgainstTheText,
    },
    Clause {
        number: "5.6.1",
        requirement: "A control that locks or toggles must say which way it is set without being \
                      looked at.",
        standing: met(
            "alo-access",
            "src/tree.rs",
            "every_surface_says_what_it_is_and_what_is_in_it",
        ),
        checked: NotAgainstTheText,
    },
    Clause {
        number: "5.6.2",
        requirement: "A control that locks or toggles must say which way it is set where it can be \
                      seen.",
        standing: not_yet(6, THE_DESKTOP_PLAN),
        checked: NotAgainstTheText,
    },
    Clause {
        number: "5.7",
        requirement: "A key held down must be able to stop repeating, and the delay before it \
                      repeats must be adjustable.",
        standing: not_yet(6, THE_DESKTOP_PLAN),
        checked: NotAgainstTheText,
    },
    Clause {
        number: "5.8",
        requirement: "A key struck twice quickly must be able to count as one.",
        standing: not_yet(6, THE_DESKTOP_PLAN),
        checked: NotAgainstTheText,
    },
    Clause {
        number: "5.9",
        requirement: "Nothing must need two keys or two fingers at once.",
        standing: not_yet(6, THE_DESKTOP_PLAN),
        checked: NotAgainstTheText,
    },
    Clause {
        number: "11.1.1.1",
        requirement: "Anything that is not text must have text that says what it is.",
        standing: met(
            "alo-access",
            "tests/every_surface_the_shell_draws_is_read_aloud.rs",
            "every_frame_the_shell_exports_is_a_surface_something_can_be_said_about",
        ),
        checked: NotAgainstTheText,
    },
    Clause {
        number: "11.1.3.1",
        requirement: "What a thing is, and what it belongs to, must be readable by a program and \
                      not only visible.",
        standing: met(
            "alo-access",
            "src/tree.rs",
            "every_surface_says_what_it_is_and_what_is_in_it",
        ),
        checked: NotAgainstTheText,
    },
    Clause {
        number: "11.1.3.2",
        requirement: "The order things are read in must be an order that makes sense.",
        standing: met(
            "alo-access",
            "src/tree.rs",
            "the_approval_is_read_as_the_sentence_then_its_two_answers",
        ),
        checked: NotAgainstTheText,
    },
    Clause {
        number: "11.1.3.3",
        requirement: "Nothing must be explained only by where it is, what shape it is or what \
                      colour it is.",
        standing: not_yet(6, THE_DESKTOP_PLAN),
        checked: NotAgainstTheText,
    },
    Clause {
        number: "11.1.3.4",
        requirement: "Nothing must work in only one screen orientation.",
        standing: not_yet(6, THE_DESKTOP_PLAN),
        checked: NotAgainstTheText,
    },
    Clause {
        number: "11.1.3.5",
        requirement: "A field asking for something about the person must say which thing it is \
                      asking for.",
        standing: not_yet(6, THE_DESKTOP_PLAN),
        checked: NotAgainstTheText,
    },
    Clause {
        number: "11.1.4.1",
        requirement: "Colour must never be the only thing saying something.",
        standing: met(
            "alo-appearance",
            "src/contrast.rs",
            "terracotta_on_cream_cannot_be_the_only_thing_saying_something",
        ),
        checked: NotAgainstTheText,
    },
    Clause {
        number: "11.1.4.2",
        requirement: "Sound that starts by itself must be stoppable.",
        standing: not_yet(2, "docs/autonomy/v0-5-devices-and-media-plan.md"),
        checked: NotAgainstTheText,
    },
    Clause {
        number: "11.1.4.3",
        requirement: "Text must stand out from what is behind it by a measured amount.",
        standing: met(
            "alo-appearance",
            "src/contrast.rs",
            "navy_on_cream_reads_and_the_number_is_the_published_one",
        ),
        checked: NotAgainstTheText,
    },
    Clause {
        number: "11.1.4.4",
        requirement: "Text must be able to be made bigger without anything being lost.",
        standing: not_yet(6, THE_DESKTOP_PLAN),
        checked: NotAgainstTheText,
    },
    Clause {
        number: "11.1.4.5",
        requirement: "Words must be text rather than a picture of text.",
        standing: not_yet(6, THE_DESKTOP_PLAN),
        checked: NotAgainstTheText,
    },
    Clause {
        number: "11.1.4.10",
        requirement: "Everything must still be usable when the screen is small or the text is \
                      large, without scrolling in two directions.",
        standing: not_yet(6, THE_DESKTOP_PLAN),
        checked: NotAgainstTheText,
    },
    Clause {
        number: "11.1.4.11",
        requirement: "The edges of a control must stand out from what is behind them by a measured \
                      amount.",
        standing: not_yet(6, THE_DESKTOP_PLAN),
        checked: NotAgainstTheText,
    },
    Clause {
        number: "11.1.4.12",
        requirement: "Text must survive being given more space between its letters, words and \
                      lines.",
        standing: not_yet(6, THE_DESKTOP_PLAN),
        checked: NotAgainstTheText,
    },
    Clause {
        number: "11.1.4.13",
        requirement: "Anything that appears when a pointer or the focus is on something must be \
                      dismissable and must stay long enough to read.",
        standing: not_yet(6, THE_DESKTOP_PLAN),
        checked: NotAgainstTheText,
    },
    Clause {
        number: "11.2.1.1",
        requirement: "Everything must be usable from a keyboard alone.",
        standing: not_yet(3, THE_PLAN),
        checked: NotAgainstTheText,
    },
    Clause {
        number: "11.2.1.2",
        requirement: "The keyboard must never be trapped somewhere it cannot leave.",
        standing: not_yet(3, THE_PLAN),
        checked: NotAgainstTheText,
    },
    Clause {
        number: "11.2.1.4",
        requirement: "A shortcut that is a single letter must be able to be turned off or changed.",
        standing: not_yet(3, THE_PLAN),
        checked: NotAgainstTheText,
    },
    Clause {
        number: "11.2.2.1",
        requirement: "A time limit must be able to be turned off, adjusted or extended.",
        standing: not_yet(6, THE_DESKTOP_PLAN),
        checked: NotAgainstTheText,
    },
    Clause {
        number: "11.2.2.2",
        requirement: "Anything that moves, blinks or scrolls by itself must be able to be paused.",
        standing: not_yet(6, THE_DESKTOP_PLAN),
        checked: NotAgainstTheText,
    },
    Clause {
        number: "11.2.3.1",
        requirement: "Nothing must flash in the way that causes seizures.",
        standing: not_yet(6, THE_DESKTOP_PLAN),
        checked: NotAgainstTheText,
    },
    Clause {
        number: "11.2.4.3",
        requirement: "The focus must move in an order that keeps meaning.",
        standing: not_yet(3, THE_PLAN),
        checked: NotAgainstTheText,
    },
    Clause {
        number: "11.2.4.6",
        requirement: "Headings and labels must say what they are about.",
        standing: met(
            "alo-access",
            "tests/every_surface_the_shell_draws_is_read_aloud.rs",
            "every_control_is_named_and_the_approval_reads_as_the_sentence_it_asks",
        ),
        checked: NotAgainstTheText,
    },
    Clause {
        number: "11.2.4.7",
        requirement: "Wherever the focus is, it must be visible.",
        standing: not_yet(3, THE_PLAN),
        checked: NotAgainstTheText,
    },
    Clause {
        number: "11.2.5.1",
        requirement: "Anything done with a path or several fingers must have a simpler way.",
        standing: not_yet(6, THE_DESKTOP_PLAN),
        checked: NotAgainstTheText,
    },
    Clause {
        number: "11.2.5.2",
        requirement: "Pressing something must be undoable before it is let go.",
        standing: not_yet(6, THE_DESKTOP_PLAN),
        checked: NotAgainstTheText,
    },
    Clause {
        number: "11.2.5.3",
        requirement: "A control's name for a program must contain the words a person sees on it.",
        standing: met(
            "alo-access",
            "tests/every_surface_the_shell_draws_is_read_aloud.rs",
            "every_control_is_named_and_the_approval_reads_as_the_sentence_it_asks",
        ),
        checked: NotAgainstTheText,
    },
    Clause {
        number: "11.2.5.4",
        requirement: "Anything done by moving the machine must also be doable without moving it.",
        standing: not_applicable(
            "nothing in this shell is done by moving, shaking or tilting the machine",
        ),
        checked: NotAgainstTheText,
    },
    Clause {
        number: "11.3.1.1",
        requirement: "The language the software is in must be readable by a program.",
        standing: not_yet(6, THE_DESKTOP_PLAN),
        checked: NotAgainstTheText,
    },
    Clause {
        number: "11.3.2.1",
        requirement: "Nothing must change under a person because the focus arrived on it.",
        standing: not_yet(6, THE_DESKTOP_PLAN),
        checked: NotAgainstTheText,
    },
    Clause {
        number: "11.3.2.2",
        requirement: "Nothing must change under a person because they typed into it.",
        standing: not_yet(6, THE_DESKTOP_PLAN),
        checked: NotAgainstTheText,
    },
    Clause {
        number: "11.3.3.1",
        requirement: "A mistake must be described in words, not only shown in a colour.",
        standing: not_yet(6, THE_DESKTOP_PLAN),
        checked: NotAgainstTheText,
    },
    Clause {
        number: "11.3.3.2",
        requirement: "Anything a person has to fill in must say what it wants.",
        standing: met(
            "alo-access",
            "tests/every_surface_the_shell_draws_is_read_aloud.rs",
            "every_control_is_named_and_the_approval_reads_as_the_sentence_it_asks",
        ),
        checked: NotAgainstTheText,
    },
    Clause {
        number: "11.4.1.2",
        requirement: "Everything on the screen must tell a program what it is, what it is called \
                      and what state it is in.",
        standing: met(
            "alo-access",
            "src/tree.rs",
            "every_surface_says_what_it_is_and_what_is_in_it",
        ),
        checked: NotAgainstTheText,
    },
    Clause {
        number: "11.5.2.15",
        requirement: "A change on the screen must be announced to an assistive technology rather \
                      than waited for.",
        standing: met(
            "alo-access",
            "src/tree.rs",
            "what_is_leaving_and_what_the_agent_is_doing_are_announced_rather_than_found",
        ),
        checked: NotAgainstTheText,
    },
];
