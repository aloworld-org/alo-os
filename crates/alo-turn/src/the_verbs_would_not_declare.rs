//! Why the list of verbs this machine can carry out could not be made.
//!
//! Its own type rather than one of the two families', because the moment there
//! was a second family *which one refused* became a question with an answer. It
//! is a small thing and it is the exact shape this repository has been caught
//! by before: `alo_files::Declaring` and `alo_software::Declaring` have the
//! same two variants over the same two underlying errors, so a software
//! failure can be converted into a file one and compiles perfectly. What comes
//! out the other end is *the file verbs could not be declared*, about a file
//! verb that is fine, and whoever reads it starts in the wrong crate.
//!
//! So the two are kept apart and the family is named. Nothing else is added:
//! each variant carries the family's own error whole, with its own `Display`,
//! because that error already says what is wrong with the declaration.
//!
//! # English, like the two it holds
//!
//! Neither family's `Declaring` is a sentence a person reads, and neither is
//! this: a verb declaration that does not satisfy the contract, or two
//! declarations landing on one name, cannot be caused by anything a person or
//! an agent did. It is read by whoever is fixing the declaration, which is why
//! it keeps its English and its `Display`.

/// The verbs this machine can carry out could not be declared.
/// **No `Clone`**, because neither family's `Declaring` has one and there is
/// nothing here worth adding one for: this is returned once, by a constructor,
/// to a caller that reports it and stops.
#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum TheVerbsWouldNotDeclare {
    /// `alo-files`' six.
    #[error("the file verbs could not be declared: {0}")]
    TheFileVerbs(#[from] alo_files::Declaring),
    /// `alo-software`'s one, or its name colliding with one of the six.
    #[error("the software verb could not be declared: {0}")]
    TheSoftwareVerb(#[from] alo_software::Declaring),
    /// `alo-applications`' three, or a name colliding with one of the seven.
    ///
    /// **Three of its four**, and the fourth's absence is the machine's choice
    /// rather than this type's: `arrange_application` has no road, so
    /// `Machine::carrying_out_file_software_and_application_verbs` declares the
    /// three with
    /// `alo_applications::verbs::declare_the_three_with_a_road_into` and that
    /// function carries the argument.
    #[error("the application verbs could not be declared: {0}")]
    TheApplicationVerbs(#[from] alo_applications::Declaring),
    /// `alo-record`'s one, or its name colliding with one of the ten.
    ///
    /// The record's question, added on 2026-10-18 when the owner decided that
    /// the surface for *find a command by what it did* is a question to alo.
    #[error("the record verb could not be declared: {0}")]
    TheRecordVerb(#[from] alo_record::Declaring),
}
