//! **What the machine answers when a step is taken**, as a booted machine at
//! full access would — the script a try's later choices are made against.
//!
//! Every answer is built from the protocol's own types and constructors, the
//! ones `alo-agentd` builds its answers from
//! (`what_a_client_is_told_a_verb_did.rs`, `saying_it_was_done_without_asking.rs`),
//! so the model reads a line in the shape it would read on a machine. **The
//! contents are the script's**: the window the editor shows is the one
//! `docs/quirks/vscodium-is-pressed-and-read-only-when-described-completely.md`
//! measured VSCodium showing — a text entry named after the open file — and
//! not one read off a machine during the try.

use alo_acting::{Acting, OPEN_FILE};
use alo_adapters::{Role, Seen, Shown, ShownWindow};
use alo_applications::Application;
use alo_capability::words::DONE_AT_FULL_ACCESS;
use alo_capability::{Call, Value};
use alo_protocol::{Done, ToAnAgent};
use alo_publishing::WhatWasPublished;
use alo_software::{Catalogue, Category, WhatTheCatalogueHolds};
use alo_strings::{Filling, Strings};

use crate::a_road::Answered;

/// The machine's answer to `call`, taken as a step answered `answered`.
#[must_use]
pub fn answering(answered: &Answered, call: &Call, strings: &Strings) -> ToAnAgent {
    let application = the_value_of(call, alo_applications::verbs::APPLICATION);
    match answered {
        Answered::AFileOpened => done_at_full_access(
            Done::of_an_application_act(OPEN_FILE, &application),
            call,
            strings,
        ),
        Answered::AFileOpenedTypingInto { name, window } => done_at_full_access(
            Done::of_a_file_opened(
                OPEN_FILE,
                &application,
                Some((name.clone(), window.clone())),
            ),
            call,
            strings,
        ),
        Answered::AnApplicationOpened => {
            let act = Application::identified(&application)
                .map(|which| Acting::Open(which).on_the_wire())
                .unwrap_or_default();
            done_at_full_access(
                Done::of_an_application_act(act, &application),
                call,
                strings,
            )
        }
        Answered::AWindowShowing { title, field } => {
            let shown = Shown::of(
                application,
                vec![ShownWindow::titled(
                    title.clone(),
                    vec![Seen::of(
                        Role::TextField,
                        field.clone(),
                        Some(String::new()),
                        1,
                    )],
                )],
                false,
            );
            ToAnAgent::did(Done::of_a_window_read(&shown, &[]))
        }
        Answered::TypedAndSaved => {
            let name = the_value_of(call, alo_adapters::fallback_verbs::NAME);
            let typed = the_value_of(call, alo_adapters::fallback_verbs::TEXT);
            done_at_full_access(
                Done::of_a_typing(&application, &name, typed.chars().count(), true, true),
                call,
                strings,
            )
        }
        Answered::Published { account } => {
            let folder = the_value_of(call, alo_publishing::words::FOLDER);
            let address = format!("https://{account}.github.io/");
            let said = alo_publishing::published_without_asking(
                WhatWasPublished {
                    folder: &folder,
                    address: &address,
                    nothing_new: false,
                },
                strings,
            );
            ToAnAgent::done_without_asking(Done::of_a_publication(&folder, &address, false), &said)
        }
        Answered::TheCatalogue(catalogue) => the_catalogue_under(catalogue, call),
        Answered::With(done) if call.waits_for_approval() => {
            done_at_full_access(done.clone(), call, strings)
        }
        Answered::With(done) => ToAnAgent::did(done.clone()),
    }
}

/// What `catalogue` holds under the kind of work `call` chose: matched
/// against the same list the options are built from, as
/// `alo_software::looked_at` matches it, and everything for `everything`.
fn the_catalogue_under(catalogue: &Catalogue, call: &Call) -> ToAnAgent {
    let chosen = the_value_of(call, alo_software::words::KIND_OF_WORK);
    let kind = Category::EVERY
        .into_iter()
        .find(|kind| kind.as_an_option() == chosen);
    ToAnAgent::did(Done::of_a_catalogue_read(&WhatTheCatalogueHolds::under(
        catalogue, kind,
    )))
}

/// What `argument` was given as, or nothing.
fn the_value_of(call: &Call, argument: &str) -> String {
    call.value(argument)
        .map(Value::describe)
        .unwrap_or_default()
}

/// Done at full access, said as `alo-agentd` says it: the call's own sentence
/// inside the full-access words (`saying_it_was_done_without_asking.rs`).
fn done_at_full_access(did: Done, call: &Call, strings: &Strings) -> ToAnAgent {
    let said = strings.say(
        &DONE_AT_FULL_ACCESS.key(),
        &Filling::of("sentence", call.sentence(strings).into_text()),
    );
    ToAnAgent::done_without_asking(did, &said)
}

#[cfg(test)]
#[path = "the_machine_answers_tests.rs"]
mod tests;
