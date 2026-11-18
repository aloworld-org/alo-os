//! Published approval labels shared without depending on the turn executor.
//!
//! The approval surface re-exports and declares these exact published words.
//! Accessibility reads the same definitions; their keys, English and notes stay
//! unchanged. Keeping labels below execution avoids a cycle through adapters.

use alo_strings::Word;

/// The answer that carries the change out: one approval, one execution.
pub const APPROVE: Word = Word::saying("approving.approve", "Approve").noting(
    "The answer that lets one change go ahead, read directly beneath a sentence describing exactly \
     what the agent will do — the agent being the assistant built into alo OS and not a person. It \
     approves that one change and nothing else: it is never a session, a permission or a setting, \
     so a translation that reads as \"always allow\" or \"trust\" would say something alo OS does \
     not offer. Short because the sentence above it carries the whole meaning.",
);

/// The answer that is a whole answer: no, and nobody is asked why.
pub const NO: Word = Word::saying("approving.no", "No").noting(
    "The answer that stops one change, read directly beneath a sentence describing exactly what \
     the agent would do — the agent being the assistant built into alo OS and not a person. It is \
     a complete answer and no reason is asked for or recorded, so a translation should not soften \
     it into \"not now\" or \"cancel\": nothing is postponed and nothing is undone, the change \
     simply does not happen. Short because the sentence above it carries the whole meaning.",
);
