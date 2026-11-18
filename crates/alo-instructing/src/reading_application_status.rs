//! A verb's purpose describes an operation, not an application's current state.

use alo_capability::{Effect, Verbs};

/// Guide live status questions only where the registry declares a status read.
pub(crate) fn reading_application_status(verbs: &Verbs) -> String {
    let reads: Vec<&str> = verbs
        .all()
        .filter(|verb| verb.effect() == Effect::Read && verb.name().ends_with(".status"))
        .map(|verb| verb.name())
        .collect();
    if reads.is_empty() {
        return String::new();
    }
    format!(
        "\n\nRead the live status before answering a question about an application's current \
         state, using its status read: {}. Remember that a verb's description is not a live \
         report, even when it lists the state the person asks about. Send the matching status \
         read, then answer from its reply.",
        reads.join(", ")
    )
}
