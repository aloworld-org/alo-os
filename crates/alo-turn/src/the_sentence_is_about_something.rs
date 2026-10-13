//! **A person is never asked about something that is not there.**
//!
//! `CLAUDE.md`: *what a person approves is that sentence.* So the sentence has
//! to be true when they read it, and until 2026-10-13 it did not have to be.
//!
//! # What went wrong, measured
//!
//! `alo_capability::Proposal::checked` asks three things: that this is a change
//! and not a read, that the grants permit it, and that the question has time to
//! stand. **Not that anything named in it exists** — existence is
//! `alo_files::Resolving::real`'s answer, and it is asked inside
//! `alo_files::Touching` at the moment the change runs.
//!
//! That ordering is right and stays: a file can be deleted between a person
//! being asked and them saying yes, so the check at execution is the last word
//! and nothing here replaces it.
//!
//! What it left open was found by the agent's first real run. Asked *find
//! whichever invoice is for March and add `-final` to its name*, over a folder
//! holding `march.pdf` and `april.pdf`, **both `gpt-4o-mini` and `gpt-4o`
//! invented a filename** — `march_invoice.pdf` and `invoice_march.pdf` — and
//! neither listed the folder. The proposal was **accepted**, because the grants
//! covered the folder and it was a change. A sentence was composed saying
//! *rename `march_invoice.pdf` to `march_invoice-final.pdf`*, and the refusal
//! arrived only when the person approved it.
//!
//! Two things come out of that, and the second is the larger.
//!
//! **A person is shown a sentence that is false.** They approve something
//! impossible and are then told it could not be done. Nothing is damaged —
//! the capability model did its job — but *what a person approves is that
//! sentence* stops being true of the product.
//!
//! **And the agent cannot correct itself.** `crate::Turning` ends a turn when a
//! change is waiting, because waiting for a person is not something an agent
//! does. So the one refusal that would have taught the model to look first is
//! the one it never sees. Asking here instead closes that loop: the agent is
//! told *there is nothing at …* while it still has rounds left, and
//! `alo_agent::driving` feeds a refusal back as a step.
//!
//! # What is asked, and what deliberately is not
//!
//! Every [`Ask::Path`] the call reaches is resolved, through the same
//! [`Resolving`] the execution uses, so **the refusal is the same sentence
//! word for word** — `alo-files` wrote it and nothing here rewords it.
//!
//! Only what the call **reaches**, never what it would create. Each of the six
//! file verbs names what it creates with a `Takes::name` and not a path —
//! `rename_file`'s new name, `archive_folder`'s archive — so those are not in
//! [`Call::asks`] and are not looked for. A check over every path *value*
//! rather than over `asks` would refuse every archive ever proposed.
//!
//! **Containment is not asked here**, though resolving makes it nearly free: a
//! link that leads out of a granted folder is `Touching`'s refusal at the
//! moment of execution, where the grants are asked again anyway. What this file
//! is for is the sentence being *true*, and existence is what makes it false.
//! Adding a second opinion about containment would be two places deciding one
//! thing.

use alo_capability::{Ask, Call, Refused};
use alo_files::Resolving;
use alo_strings::Strings;

/// Refuse a change that names something which is not there.
///
/// # Errors
/// [`Refused`], worded by `alo-files` — the same value and the same sentence
/// the execution would have answered with.
pub(crate) fn the_sentence_is_about_something(
    call: &Call,
    resolving: &dyn Resolving,
    strings: &Strings,
) -> Result<(), Refused> {
    for ask in call.asks() {
        let Ask::Path(given) = ask else {
            continue;
        };
        if let Err(why) = resolving.real(given) {
            return Err(Refused::worded_elsewhere(call.clone(), why.said(strings)));
        }
    }
    Ok(())
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::the_sentence_is_about_something;
    use crate::testing::{a_folder_of_our_own, in_english};
    use alo_capability::Given;
    use alo_files::{OnThisMachine, file_verbs};

    /// **A change naming a file that is not there is refused, in `alo-files`'
    /// own sentence.**
    ///
    /// The words are asserted rather than only the refusal, because the whole
    /// point of resolving through the same [`Resolving`] the execution uses is
    /// that a person and an agent read the *same* sentence whichever moment
    /// refused it. A reworded one here would be a second thing to keep true.
    #[test]
    fn a_change_naming_nothing_is_refused_in_the_words_the_execution_would_use() {
        let strings = in_english();
        let folder = a_folder_of_our_own("nothing-there");
        let missing = folder.join("march_invoice.pdf");
        let call = file_verbs()
            .unwrap()
            .call(
                "rename_file",
                &[
                    ("file", Given::text(missing.display().to_string())),
                    ("name", Given::text("march-final.pdf")),
                ],
            )
            .unwrap();

        // `unwrap_err` rather than `expect_err`: this module exempts
        // `unwrap_used` and not `expect_used`, counted rather than copied.
        let refused = the_sentence_is_about_something(&call, &OnThisMachine, &strings).unwrap_err();
        let said = refused.said(&strings);
        assert!(!said.is_a_bug(), "{said}");
        assert!(
            said.text().contains("there is nothing at"),
            "not the sentence `alo-files` wrote: {said}"
        );
        assert!(
            said.text().contains("march_invoice.pdf"),
            "the refusal does not name what was asked for: {said}"
        );
    }

    /// And one naming a file that is there passes, so this refuses only what it
    /// is for.
    #[test]
    fn a_change_naming_a_real_file_is_let_through() {
        let strings = in_english();
        let folder = a_folder_of_our_own("really-there");
        let invoice = folder.join("march.pdf");
        std::fs::write(&invoice, "March").unwrap();
        let call = file_verbs()
            .unwrap()
            .call(
                "rename_file",
                &[
                    ("file", Given::text(invoice.display().to_string())),
                    ("name", Given::text("march-final.pdf")),
                ],
            )
            .unwrap();
        assert!(the_sentence_is_about_something(&call, &OnThisMachine, &strings).is_ok());
    }

    /// **What a verb creates is not looked for**, which is the one way this
    /// check could have broken every archive ever proposed.
    ///
    /// `archive_folder` reaches `folder` and `into` — both of which must be
    /// there — and names the archive itself with a `Takes::name`, so the thing
    /// that does not exist yet is not an [`Ask::Path`] and is never resolved.
    #[test]
    fn what_a_verb_would_create_is_not_looked_for() {
        let strings = in_english();
        let folder = a_folder_of_our_own("to-archive");
        std::fs::write(folder.join("march.pdf"), "March").unwrap();
        let into = a_folder_of_our_own("archives");
        let call = file_verbs()
            .unwrap()
            .call(
                "archive_folder",
                &[
                    ("folder", Given::text(folder.display().to_string())),
                    ("into", Given::text(into.display().to_string())),
                    // The archive does not exist and is not supposed to.
                    ("name", Given::text("invoices-2026.zip")),
                ],
            )
            .unwrap();
        assert!(
            the_sentence_is_about_something(&call, &OnThisMachine, &strings).is_ok(),
            "an archive was refused because the archive it would make is not there yet"
        );
    }

    /// A verb that reaches nothing — `install_application`, whose
    /// `Requires::nothing_because` says the application is not on the machine
    /// yet — has no paths to look for and is not refused for having none.
    #[test]
    fn a_verb_that_reaches_no_path_is_not_refused_for_it() {
        let strings = in_english();
        let mut verbs = file_verbs().unwrap();
        alo_software::verbs::declare_into(&mut verbs).unwrap();
        let call = verbs
            .call(
                alo_software::INSTALL_APPLICATION,
                &[
                    ("application", Given::text("org.gnome.TextEditor")),
                    ("source", Given::text("flathub")),
                ],
            )
            .unwrap();
        assert!(call.asks().is_empty(), "an installation reaches a path");
        assert!(the_sentence_is_about_something(&call, &OnThisMachine, &strings).is_ok());
    }
}
