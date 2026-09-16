//! Why an adapter was not loaded.
//!
//! **English, with a `Display`**, like `alo_capability::VerbError`: every one of
//! these refuses a *declaration*, and its reader is whoever is writing the
//! adapter at the moment its tests fail. `docs/contracts/agent-verbs.md` asks an
//! adapter's declaration-time errors to be exactly that, and each says what to
//! change rather than which rule was broken.

use alo_capability::{VerbError, VerbsError};

/// Why an adapter was not loaded.
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum NotLoaded {
    /// A name an agent could not be called by.
    #[error(
        "name the adapter in lower-case words joined by underscores, like text_editor — {name} is not one"
    )]
    NotAName {
        /// The name as declared.
        name: String,
    },
    /// An application identifier no verb could ever name.
    #[error("{application} is not an application identifier this machine could have installed")]
    NotAnApplication {
        /// The identifier as declared.
        application: String,
    },
    /// An adapter for an application that is a person's own (ADR 0043).
    #[error(
        "{application} is a person's own and never an agent's — an adapter for it would be a command verb by another road (ADR 0043)"
    )]
    APersonsOwn {
        /// The identifier as declared.
        application: String,
    },
    /// No release the adapter says it supports.
    #[error(
        "say which releases of the application {adapter} supports — a verb's meaning is only stable against a release"
    )]
    NoReleases {
        /// The adapter.
        adapter: String,
    },
    /// Screenshots and synthetic input.
    #[error(
        "{adapter} drives its application with screenshots and synthetic input, and nobody can say afterwards what that did — no adapter using them is loaded"
    )]
    Synthetic {
        /// The adapter.
        adapter: String,
    },
    /// A mechanism nothing on this machine carries out yet.
    #[error(
        "{adapter} uses the {mechanism} mechanism, which nothing on this machine carries out yet — a verb the machine cannot carry out is never offered"
    )]
    NotCarriedOutHere {
        /// The adapter.
        adapter: String,
        /// The mechanism's name.
        mechanism: &'static str,
    },
    /// An adapter that can do nothing.
    #[error("{adapter} declares no verbs, so there is nothing to load")]
    NoVerbs {
        /// The adapter.
        adapter: String,
    },
    /// A verb that takes a script, a command or free text.
    #[error(
        "{verb} takes {argument} as {kind} — an adapter never takes a script, a command or free text; split the verb into typed ones and build what runs inside the adapter (ADR 0001 §1)"
    )]
    TakesCode {
        /// The verb.
        verb: String,
        /// The argument.
        argument: String,
        /// What it was declared as.
        kind: &'static str,
    },
    /// A verb that hands an argument to something that interprets it.
    #[error(
        "{verb} hands {argument} to {by} to interpret — the model never authors what runs; generate it inside the adapter from typed arguments (ADR 0001 §6)"
    )]
    BecomesCode {
        /// The verb.
        verb: String,
        /// The argument.
        argument: String,
        /// What interprets it.
        by: String,
    },
    /// A method named after running something.
    #[error(
        "{verb} calls {method}, which is named for running something — an adapter reaches typed methods only (ADR 0001 §1)"
    )]
    RunsSomething {
        /// The verb.
        verb: String,
        /// The interface and method.
        method: String,
    },
    /// An argument choosing which of the application's actions runs.
    #[error(
        "{verb} lets an argument choose which action {method} runs — write the action into the adapter, one verb per action, so its meaning is not the model's to choose"
    )]
    ChoosesWhatRuns {
        /// The verb.
        verb: String,
        /// The interface and method.
        method: String,
    },
    /// An object, interface or method that is not a well-formed name.
    #[error("{verb} names {what}, which is not a well-formed object, interface or method name")]
    NotWellFormed {
        /// The verb.
        verb: String,
        /// What it names.
        what: String,
    },
    /// A parameter filled from an argument the verb does not take.
    #[error("{verb} fills a parameter from {argument}, which it does not take")]
    NoSuchArgument {
        /// The verb.
        verb: String,
        /// The argument named.
        argument: String,
    },
    /// A parameter filled from an argument of the wrong kind.
    #[error(
        "{verb} puts {argument} where its kind cannot go — a file address is made from a path, text from a name or a choice, a number from a count"
    )]
    WrongKind {
        /// The verb.
        verb: String,
        /// The argument.
        argument: String,
    },
    /// An argument a person approves that the application never receives.
    #[error(
        "{verb} takes {argument} and never sends it — a person approves the sentence that names it, so it must be what the application is asked"
    )]
    GoesNowhere {
        /// The verb.
        verb: String,
        /// The argument.
        argument: String,
    },
    /// A path argument no grant is required over.
    #[error(
        "{verb} takes the path {argument} without requiring a grant over it — a verb never reaches outside its grant"
    )]
    OutsideItsGrant {
        /// The verb.
        verb: String,
        /// The argument.
        argument: String,
    },
    /// No by-hand road (ADR 0009).
    #[error(
        "say how a person does what {verb} does in the application themselves — a verb that is the only way to do something disappears when the agent does (ADR 0009)"
    )]
    NoByHand {
        /// The verb.
        verb: String,
    },
    /// A word the adapter uses and does not declare.
    #[error(
        "{verb} is declared with {key}, which {adapter} does not declare — it would reach a person as a key"
    )]
    WordNotDeclared {
        /// The adapter.
        adapter: String,
        /// The verb.
        verb: String,
        /// The word's key.
        key: String,
    },
    /// The verb contract refused the verb.
    #[error("{verb}: {why}")]
    Verb {
        /// The verb.
        verb: String,
        /// What the contract said.
        why: VerbError,
    },
    /// A name already on the machine's list.
    #[error(transparent)]
    Taken(#[from] VerbsError),
    /// A second adapter of one name, or for one application.
    #[error(
        "{adapter} is loaded already, or another adapter is for {application} — one application has one adapter, so an agent never has to guess which it asked"
    )]
    Twice {
        /// The adapter.
        adapter: String,
        /// Its application.
        application: String,
    },
}
