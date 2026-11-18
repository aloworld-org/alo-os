//! Prefer an offered application's own operation when its purpose matches.

use alo_capability::{Effect, Verbs};
use std::fmt::Write as _;

/// Derive selection advice from live offered declarations, never a copied list.
pub(crate) fn using_named_application_controls(verbs: &Verbs) -> String {
    let statuses: Vec<_> = verbs
        .all()
        .filter(|verb| verb.effect() == Effect::Read && verb.name().ends_with(".status"))
        .collect();
    if statuses.is_empty() {
        return String::new();
    }
    let mut text = String::from(
        "\n\nNamed application controls:\n\
         Read the live status before choosing a change in one of these applications. \
         Prefer the matching named control over a file action, an accessibility gesture, \
         or another setting. Match the requested operation to the declared purpose below. \
         Use the generic application guidance when no offered named control matches.\n",
    );
    for status in statuses {
        let Some(family) = status.name().strip_suffix(".status") else {
            continue;
        };
        // A String writer cannot fail. Keep every description attached to the
        // same declaration that supplies the real provider/tool schema.
        let _ = writeln!(text, "{}: {}", status.name(), status.purpose_as_written());
        let prefix = format!("{family}.");
        for verb in verbs
            .all()
            .filter(|verb| verb.effect() == Effect::Change && verb.name().starts_with(&prefix))
        {
            let _ = writeln!(text, "{}: {}", verb.name(), verb.purpose_as_written());
        }
    }
    text
}
