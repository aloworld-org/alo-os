//! **What alo asks the agent about a command, and how it reads the reply.**
//!
//! `docs/features.md`: *Explain a command before it runs. Plain words for what
//! it does, with anything destructive, anything needing elevated rights and
//! anything leaving the machine marked.*
//!
//! The owner decided on 2026-10-18 that the explanation comes **through the
//! agent** — one AI road, which is their ruling that `alo-ask` is the centre
//! carried into the terminal. `crate::what_a_person_asked_goes_to_the_agent` is
//! that road; this file is the only thing that knows what travels on it for an
//! explanation.
//!
//! # Why a shape at all
//!
//! The door carries a line each way. An explanation is **four things** — the
//! plain words and three marks — so a reply that was only prose would lose the
//! marks, and the marks are half the promise. So the request names a shape and
//! the reply is read against it.
//!
//! # And why the reply degrades rather than refuses
//!
//! A model answers in words, and a model that ignored the shape would leave a
//! person with nothing where they asked for something. **So a reply with no
//! marks line is still an explanation** — it simply carries no marks.
//!
//! **That is a real cost and it is named here rather than discovered**: a
//! destructive command whose explanation came back unmarked reads exactly like
//! a safe one. What stands between a person and that is
//! [`alo_terminal::what_a_command_will_do::WhatACommandWillDo::and_not_a_guarantee`],
//! which no shape of that type can omit and which every explanation carries.
//! *Marks are what a model claimed, never what this machine found, and the
//! absence of a mark has never meant the absence of the thing.*

use alo_terminal::what_a_command_will_do::{WhatACommandWillDo, WorthSeeing};

/// The line alo sends the agent to have a command explained.
///
/// **The command is last.** Everything before it is alo's instruction, so a
/// command containing words like *ignore the above* lands where instructions
/// have already been read rather than among them. That is not a guarantee
/// against a command written to mislead a model — nothing here is — and it is
/// the cheap half of not inviting it.
///
/// Newlines in the command are replaced, because the door is one line each way
/// and a command carrying one would end the request early.
#[must_use]
pub fn the_request_for(command: &str) -> String {
    let one_line: String = command
        .chars()
        .map(|it| if it == '\n' || it == '\r' { ' ' } else { it })
        .collect();
    format!(
        "Explain what this shell command would do, for somebody about to run it. \
         Answer in two parts. First line exactly `{THE_MARKS_LINE}` followed by a comma-separated \
         list of any that apply out of `{DESTRUCTIVE}`, `{ELEVATED}`, `{LEAVES}`, or `{NONE}` if \
         none do. Every line after that is the explanation, in plain words, no more than two \
         sentences. The command is: {one_line}"
    )
}

/// **Everything alo should ask the agent, given what a shell just said.**
///
/// A function rather than a `match` inside the frame loop, because the frame
/// loop is the one place in this crate no test can reach: it needs a
/// compositor, a display and a shell. **The decision about which marks become a
/// question is the part worth holding**, so it lives here where a test can cut
/// it and watch.
///
/// Empty on nearly every frame. A shell with no integration snippet installed
/// never sends a mark at all.
#[must_use]
pub fn the_requests_in(
    said: &[Result<
        alo_terminal::what_the_shell_says_about_the_prompt::AShellSaid,
        alo_terminal::what_the_shell_says_about_the_prompt::NotRead,
    >],
) -> Vec<String> {
    use alo_terminal::what_the_shell_says_about_the_prompt::AShellSaid;
    said.iter()
        .filter_map(|one| match one {
            // **A typed line is a person asking what a command will do**, and
            // the only thing that produces one is their own key press.
            Ok(AShellSaid::TheyTyped(command)) => Some(the_request_for(command)),
            // **The cycle marks say where a prompt and a command are**, which
            // the kernel already answers better for *why is it waiting* — so
            // they are read and kept for what needs them later rather than
            // given a use they do not have.
            Ok(_) => None,
            // **A refusal asks nothing.** The only one a person can cause is a
            // command too long to read, and the sentence for it is not written;
            // drawing the error's own English would be hardcoded English in
            // front of a person, which `CLAUDE.md` calls a bug.
            Err(_) => None,
        })
        .collect()
}

/// What the first line of a reply starts with.
const THE_MARKS_LINE: &str = "MARKS:";
/// What a model writes for *this destroys something*.
const DESTRUCTIVE: &str = "destructive";
/// What a model writes for *this needs rights beyond the person's own*.
const ELEVATED: &str = "elevated";
/// What a model writes for *something leaves this machine*.
const LEAVES: &str = "leaves-the-machine";
/// What a model writes when none of the three apply.
const NONE: &str = "none";

/// The explanation in a reply the agent sent back.
///
/// **Never fails.** A reply that ignored the shape entirely is read as an
/// explanation with no marks, which is this module's header at length: a person
/// who asked for words gets words.
///
/// An empty reply is the one case that answers [`None`], because an explanation
/// of nothing is not an explanation.
#[must_use]
pub fn what_came_back(reply: &str) -> Option<WhatACommandWillDo> {
    let mut marked = Vec::new();
    let mut words: Vec<&str> = Vec::new();

    for line in reply.lines() {
        let trimmed = line.trim();
        // **Only the first line is read for marks**, and only when it says so.
        // A model that mentions the word *destructive* in its explanation has
        // described a command, not marked one.
        if words.is_empty()
            && marked.is_empty()
            && let Some(listed) = the_marks_on(trimmed)
        {
            marked = listed;
            continue;
        }
        if !trimmed.is_empty() {
            words.push(trimmed);
        }
    }

    let said = words.join(" ");
    (!said.is_empty()).then(|| WhatACommandWillDo::of(&said, &marked))
}

/// The marks a `MARKS:` line lists, or [`None`] if this is not that line.
///
/// A name this does not know is skipped rather than refusing the line: a model
/// inventing a fourth mark should not cost a person the three that were right.
fn the_marks_on(line: &str) -> Option<Vec<WorthSeeing>> {
    let listed = line
        .strip_prefix(THE_MARKS_LINE)
        .or_else(|| line.strip_prefix(&THE_MARKS_LINE.to_lowercase()))?;
    let mut marks = Vec::new();
    for name in listed.split(',') {
        let name = name.trim().trim_matches('`').to_lowercase();
        match name.as_str() {
            DESTRUCTIVE => marks.push(WorthSeeing::Destructive),
            ELEVATED => marks.push(WorthSeeing::NeedsElevatedRights),
            LEAVES => marks.push(WorthSeeing::LeavesTheMachine),
            _ => {}
        }
    }
    Some(marks)
}

#[cfg(test)]
#[path = "what_alo_asks_about_a_command_tests.rs"]
mod tests;
