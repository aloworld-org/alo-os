//! **Whether the person's level of running code lets alo publish a folder** —
//! read at the moment, from their own file, as a command's is.
//!
//! The owner's decision of 2026-11-13, with ADR 0149 and ADR 0064:
//!
//! | the level | a publication |
//! |---|---|
//! | full access | carried out without asking (`crate::a_change_asked_for`), and the line afterwards says it went to the internet and Put back can't recall it |
//! | *ask me each time* | put to the person in the sentence that carries that warning, and waits for one OK |
//! | the sealed box, nothing chosen, a file that will not read | refused before anybody is asked, in a sentence that says what it cannot do and where to choose |
//!
//! **Why the sealed box refuses it**: a publication runs `git` and `gh` as the
//! person, and the sealed box cannot run programs on this computer yet — the
//! same reason it refuses a command (`crate::running_a_command`). Putting it to
//! the person would be asking them to approve something the machine would then
//! not do. Asked again at the OK, so a person who moved to the sealed box while
//! the question stood has taken it back.

use alo_capability::HowCodeRuns;
use alo_strings::{Filling, Said, Strings};

use crate::running_a_command::CommandsHere;

/// The sealed box's sentence when `verb` is a publication and the person's
/// level is the sealed box; [`None`] for every other verb and level.
#[must_use]
pub fn not_at_this_level(verb: &str, here: &CommandsHere, strings: &Strings) -> Option<Said> {
    if verb != alo_publishing::PUBLISH_TO_GITHUB_PAGES {
        return None;
    }
    (here.the_level_now() == HowCodeRuns::InASealedBox).then(|| {
        strings.say(
            &alo_publishing::words::THE_SEALED_BOX_CANNOT_PUBLISH.key(),
            &Filling::nothing(),
        )
    })
}
