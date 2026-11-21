//! Which adapters an agent offered a list of verbs can reach an application through.

#![expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use alo_capability::Verbs;

use crate::text_editor::APPLICATION;
use crate::verbs::{adapter_verbs, shipped_adapters};

/// **An adapter nobody is offered is not a road**, so it is not kept: a list
/// holding none of Text Editor's verbs keeps no adapter for it.
#[test]
fn an_adapter_whose_verbs_are_not_offered_is_not_kept() {
    let kept = shipped_adapters()
        .unwrap()
        .whose_verbs_are_on(&Verbs::default());
    assert!(kept.for_application(APPLICATION).is_none());
    assert_eq!(kept.all().count(), 0);
}

/// **And an adapter whose verbs are offered is kept.**
#[test]
fn an_adapter_whose_verbs_are_offered_is_kept() {
    let kept = shipped_adapters()
        .unwrap()
        .whose_verbs_are_on(&adapter_verbs().unwrap());
    assert!(kept.for_application(APPLICATION).is_some());
}
