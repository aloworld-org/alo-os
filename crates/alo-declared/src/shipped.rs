//! The one registry of crates whose verbs alo OS ships.
//!
//! Each entry names a crate and its declaration function once. The names, the
//! combined registry and the separate registries used by acceptance tests are
//! derived from those entries. Cargo dependencies remain explicit, and
//! [`crate::held`] checks the entries against the workspace.

use std::fmt::Display;

use alo_capability::Verbs;

use crate::not_declared::NotDeclared;

/// The one file a crate that declares verbs is added to.
pub const WHERE_THE_LIST_IS: &str = "crates/alo-declared/src/shipped.rs";

/// Derive every view of the registry from the same entries.
macro_rules! declare_crates {
    ($($name:literal => $declaring:path),+ $(,)?) => {
        /// Every crate whose verbs alo OS ships, derived from the registry.
        pub const WHO_DECLARES_THEM: &[&str] = &[$($name),+];

        /// Every verb alo OS ships, declared into one registry.
        ///
        /// # Errors
        /// [`NotDeclared`] names the crate whose declaration was refused.
        pub fn every_verb_this_machine_ships() -> Result<Verbs, NotDeclared> {
            let mut verbs = Verbs::default();
            $(declare(&mut verbs, $name, $declaring)?;)+
            Ok(verbs)
        }

        /// Each registered crate's verbs, declared into separate registries.
        ///
        /// This lets acceptance compare the combined registry with its parts
        /// without maintaining another list of crates or declaration calls.
        ///
        /// # Errors
        /// [`NotDeclared`] names the crate whose declaration was refused.
        pub fn each_crates_verbs() -> Result<Vec<(&'static str, Verbs)>, NotDeclared> {
            Ok(vec![$({
                let mut verbs = Verbs::default();
                declare(&mut verbs, $name, $declaring)?;
                ($name, verbs)
            }),+])
        }
    };
}

declare_crates! {
    "alo-adapters" => alo_adapters::verbs::declare_into,
    "alo-applications" => alo_applications::declare_into,
    "alo-capturing" => alo_capturing::verbs::declare_into,
    "alo-changing-network" => alo_changing_network::verbs::declare_into,
    "alo-changing-printers" => alo_changing_printers::verbs::declare_into,
    "alo-converting" => alo_converting::verbs::declare_into,
    "alo-files" => alo_files::declare_into,
    "alo-finding" => alo_finding::verbs::declare_into,
    "alo-measuring" => alo_measuring::verbs::declare_into,
    "alo-printing" => alo_printing::verbs::declare_into,
    "alo-software" => alo_software::verbs::declare_into,
}

/// Attach the declaring crate's name to its own refusal.
fn declare<Why: Display>(
    verbs: &mut Verbs,
    list: &'static str,
    declaring: impl FnOnce(&mut Verbs) -> Result<(), Why>,
) -> Result<(), NotDeclared> {
    declaring(verbs).map_err(|why| NotDeclared::of(list, why.to_string()))
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, an unexpected declaration refusal is the failure being reported"
)]
mod tests {
    use super::*;

    /// Every registered name actually declares verbs, in a registry of its own.
    #[test]
    fn every_name_has_a_call_and_every_call_has_a_name() {
        let declared = each_crates_verbs().unwrap();
        let names: Vec<&str> = declared.iter().map(|(name, _)| *name).collect();
        assert_eq!(names, WHO_DECLARES_THEM);
        assert!(!declared.is_empty());
        for (name, verbs) in declared {
            assert!(!verbs.is_empty(), "{name} declared no verbs");
        }
    }

    /// Duplicate entries cannot conceal a missing crate in a count.
    #[test]
    fn no_crate_is_named_twice() {
        let names: std::collections::BTreeSet<_> = WHO_DECLARES_THEM.iter().collect();
        assert_eq!(names.len(), WHO_DECLARES_THEM.len());
    }
}
