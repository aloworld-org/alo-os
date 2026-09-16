//! The adapters alo OS ships, and the verbs they put on the machine's list.
//!
//! Beside them, [`declare_into`] puts the accessibility fallback's two verbs
//! ([`mod@crate::fallback_verbs`]) on the list, so they are held to ADR 0009 by the
//! same check.
//!
//! One adapter today: GNOME Text Editor ([`crate::text_editor`]), the reference adapter
//! the plan asks for. `docs/contracts/agent-verbs.md` asks a crate declaring
//! verbs to hand them over through a `pub fn declare_into` in `src/verbs.rs`,
//! because `alo-by-hand` walks the workspace for exactly that — an adapter's
//! verbs are held to ADR 0009 by the same check every other verb is, beside the
//! by-hand road its declaration carries.

use alo_capability::Verbs;

use crate::adapters::Adapters;
use crate::loading::load;
use crate::not_loaded::NotLoaded;
use crate::text_editor::TEXT_EDITOR;

/// Every adapter alo OS ships, loaded.
///
/// # Errors
/// [`NotLoaded`], which the declarations as written cannot cause — a test at
/// the bottom of this file loads them.
pub fn shipped_adapters() -> Result<Adapters, NotLoaded> {
    let mut adapters = Adapters::default();
    adapters.add(load(&TEXT_EDITOR)?)?;
    Ok(adapters)
}

/// The shipped adapters' verbs, as a list of their own.
///
/// # Errors
/// [`NotLoaded`], as [`shipped_adapters`].
pub fn adapter_verbs() -> Result<Verbs, NotLoaded> {
    let mut verbs = Verbs::default();
    shipped_adapters()?.declare_into(&mut verbs)?;
    Ok(verbs)
}

/// Put every verb this crate declares on an existing list: the shipped
/// adapters', and the accessibility fallback's two.
///
/// # Errors
/// [`NotLoaded::Taken`] if the list already has a verb of one of these names.
pub fn declare_into(verbs: &mut Verbs) -> Result<(), NotLoaded> {
    shipped_adapters()?.declare_into(verbs)?;
    crate::fallback_verbs::declare_into(verbs)
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use alo_capability::{Effect, Requires};

    #[test]
    fn the_shipped_adapters_load_and_their_verbs_are_named_under_them() {
        let verbs = adapter_verbs().unwrap();
        let names: Vec<&str> = verbs.all().map(|verb| verb.name()).collect();
        assert_eq!(
            names,
            ["text_editor.open_document", "text_editor.new_window"]
        );
        for verb in verbs.all() {
            assert_eq!(verb.effect(), Effect::Change, "{}", verb.name());
        }
        assert!(matches!(
            verbs.of("text_editor.open_document").unwrap().requires(),
            Requires::Grants(over) if over == &["document".to_owned()]
        ));
    }

    #[test]
    fn a_name_already_taken_is_not_replaced() {
        let mut verbs = adapter_verbs().unwrap();
        assert!(matches!(declare_into(&mut verbs), Err(NotLoaded::Taken(_))));
    }
}
