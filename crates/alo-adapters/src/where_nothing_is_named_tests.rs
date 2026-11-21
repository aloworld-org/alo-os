use alo_capability::Given;

use super::*;
use crate::fallback_verbs::{KIND, fallback_verbs};

const GAME: &str = "net.supertuxkart.SuperTuxKart";
const BROWSER: &str = "org.mozilla.firefox";

fn in_english() -> Strings {
    Strings::of(crate::words::adapter_words().unwrap())
}

fn a_press_of(application: &str, name: &str) -> Call {
    fallback_verbs()
        .unwrap()
        .call(
            ACTIVATE_CONTROL,
            &[
                (APPLICATION, Given::text(application)),
                (KIND, Given::text("button")),
                (NAME, Given::text(name)),
            ],
        )
        .unwrap()
}

/// **Named where nothing is named: refused, in words that send alo to a click
/// by description** (2026-11-21), naming the application and the name it gave.
#[test]
fn a_control_named_where_nothing_is_named_is_refused_with_the_road_to_a_click() {
    let said = named_where_nothing_is_named(
        &a_press_of(GAME, "Start Race"),
        &[GAME.to_owned()],
        &in_english(),
    )
    .unwrap();
    let expected = in_english()
        .say(
            &words::NAMES_NOTHING.key(),
            &Filling::of("application", GAME.to_owned()).and("name", "Start Race".to_owned()),
        )
        .into_text();
    assert_eq!(said.text(), expected);
    assert!(
        said.text()
            .contains("click the thing by describing what it looks like"),
        "{said}"
    );
}

/// **Every other call is left alone**: a window that names its controls,
/// another application, and a verb that names no control.
#[test]
fn every_other_call_is_left_alone() {
    let strings = in_english();
    assert!(named_where_nothing_is_named(&a_press_of(GAME, "Start Race"), &[], &strings).is_none());
    assert!(
        named_where_nothing_is_named(&a_press_of(BROWSER, "Reload"), &[GAME.to_owned()], &strings)
            .is_none()
    );
    let a_read = fallback_verbs()
        .unwrap()
        .call(
            crate::fallback_verbs::READ_WINDOW,
            &[(APPLICATION, Given::text(GAME))],
        )
        .unwrap();
    assert!(named_where_nothing_is_named(&a_read, &[GAME.to_owned()], &strings).is_none());
}

/// A tree holding one window of the game, with `seen` in it — or, with
/// `on_the_bus` false, no game at all.
struct ATree {
    seen: Option<(u32, &'static str)>,
    on_the_bus: bool,
}

fn at(object: &str) -> crate::accessibility_tree::NodeAt {
    crate::accessibility_tree::NodeAt {
        holder: ":1.9".to_owned(),
        object: object.to_owned(),
    }
}

impl crate::accessibility_tree::AccessibilityTree for ATree {
    fn applications(
        &self,
    ) -> Result<Vec<crate::accessibility_tree::Running>, crate::accessibility_tree::TreeFault> {
        Ok(if self.on_the_bus {
            vec![crate::accessibility_tree::Running {
                at: at("/root"),
                is: Some(GAME.to_owned()),
            }]
        } else {
            Vec::new()
        })
    }

    fn facts(
        &self,
        place: &crate::accessibility_tree::NodeAt,
    ) -> Result<crate::accessibility_tree::Facts, crate::accessibility_tree::TreeFault> {
        let facts = |role, name: &str, children| crate::accessibility_tree::Facts {
            role,
            name: name.to_owned(),
            states: vec![(1 << 25) | (1 << 30), 0],
            children,
            has_text: false,
            unlisted_children: false,
        };
        match place.object.as_str() {
            "/root" => Ok(facts(75, "", vec![at("/window")])),
            "/window" => Ok(facts(
                23,
                "SuperTuxKart",
                self.seen.map(|_| vec![at("/thing")]).unwrap_or_default(),
            )),
            "/thing" => self
                .seen
                .map(|(role, name)| facts(role, name, Vec::new()))
                .ok_or(crate::accessibility_tree::TreeFault::Vanished),
            _ => Err(crate::accessibility_tree::TreeFault::Vanished),
        }
    }

    fn text(
        &self,
        _: &crate::accessibility_tree::NodeAt,
    ) -> Result<String, crate::accessibility_tree::TreeFault> {
        Ok(String::new())
    }

    fn actions(
        &self,
        _: &crate::accessibility_tree::NodeAt,
    ) -> Result<Vec<String>, crate::accessibility_tree::TreeFault> {
        Ok(Vec::new())
    }

    fn act(
        &self,
        _: &crate::accessibility_tree::NodeAt,
        _: usize,
    ) -> Result<bool, crate::accessibility_tree::TreeFault> {
        Ok(false)
    }
}

/// **Checked when the guess arrives**: a canvas, or no window the tree knows,
/// names nothing; a named button names something.
#[test]
fn the_windows_are_asked_whether_they_name_anything() {
    let canvas = ATree {
        seen: Some((6, "")),
        on_the_bus: true,
    };
    assert!(the_windows_name_nothing(&canvas, GAME));
    let not_there = ATree {
        seen: None,
        on_the_bus: false,
    };
    assert!(the_windows_name_nothing(&not_there, GAME));
    let a_button = ATree {
        seen: Some((43, "Start race")),
        on_the_bus: true,
    };
    assert!(!the_windows_name_nothing(&a_button, GAME));
}
