//! **Every name `alo-app alo … --in settings` accepted before the live page
//! still resolves**, through the very walk and press a person's command takes
//! (`alo_adapters::press_by_hand`), on the tree a reader is given.
//!
//! The owner's decision of 2026-11-12: the old name of a row that is drawn
//! reaches it, with a notice that the name will change; the old name of a
//! setting no page draws yet is refused by a sentence saying so. Both states
//! are held, and so is the move from one to the other — which is keyed on the
//! [`Setting`] and needs nothing here to change.

use std::cell::RefCell;

use alo_access::{Surface, TurnedOn};
use alo_adapters::accessibility_tree::{AccessibilityTree, Facts, NodeAt, Running, TreeFault};
use alo_adapters::{NotPressed, Pressable, Unpressed, Whose, press_by_hand, read_by_hand};

use super::*;
use crate::SettingsWindow;
use crate::access_action::{InSettings, Pressed};
use crate::access_nodes::ReadAloudTree;
use crate::settings_as_read::SettingsShows;
use crate::settings_categories::SettingsCategory;
use crate::settings_pointed_at::PointedAt;
use crate::settings_testing::{a_browser_and_a_mail_app, a_persons_machine, noon, settings_words};

/// The name this tree answers as, as the bus would give it.
const ANSWERS_AS: &str = ":1.42";

/// **The tree a reader is given, read the way a command reads a bus** — every
/// question a walk asks, answered from the nodes and their attributes exactly
/// as `crate::access_bus` serves them, with every press kept.
struct OnTheTree {
    /// The tree.
    tree: ReadAloudTree,
    /// What was pressed, in order.
    pressed: RefCell<Vec<Pressed>>,
}

impl OnTheTree {
    /// The node at `at`.
    fn node(&self, at: &NodeAt) -> Result<&crate::access_nodes::Node, TreeFault> {
        self.tree
            .nodes()
            .iter()
            .find(|node| node.path == at.object)
            .ok_or(TreeFault::Vanished)
    }

    /// Where a node is.
    fn at(path: &str) -> NodeAt {
        NodeAt {
            holder: ANSWERS_AS.to_owned(),
            object: path.to_owned(),
        }
    }
}

impl AccessibilityTree for OnTheTree {
    fn applications(&self) -> Result<Vec<Running>, TreeFault> {
        Ok(vec![Running {
            at: Self::at(crate::access_nodes::ROOT),
            is: None,
        }])
    }

    fn facts(&self, at: &NodeAt) -> Result<Facts, TreeFault> {
        let node = self.node(at)?;
        Ok(Facts {
            role: node.role,
            name: node.name.clone(),
            states: node.states.to_vec(),
            children: node
                .children
                .iter()
                .filter_map(|child| self.tree.nodes().get(*child))
                .map(|child| Self::at(&child.path))
                .collect(),
            has_text: false,
            unlisted_children: false,
        })
    }

    fn text(&self, _at: &NodeAt) -> Result<String, TreeFault> {
        Ok(String::new())
    }

    fn actions(&self, at: &NodeAt) -> Result<Vec<String>, TreeFault> {
        Ok(self
            .node(at)?
            .press
            .iter()
            .map(|_| "click".to_owned())
            .collect())
    }

    fn act(&self, at: &NodeAt, action: usize) -> Result<bool, TreeFault> {
        let press = self.node(at)?.press.clone();
        match (action, press) {
            (0, Some(press)) => {
                self.pressed.borrow_mut().push(press);
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    fn attributes(&self, at: &NodeAt) -> Result<Vec<(String, String)>, TreeFault> {
        Ok(self
            .tree
            .attributes_of(self.node(at)?)
            .into_iter()
            .collect())
    }
}

/// Settings as it is now, as a reader is given it.
fn on(window: &SettingsWindow) -> OnTheTree {
    let strings = settings_words();
    OnTheTree {
        tree: ReadAloudTree::of(&strings, &[Surface::Desktop], &TurnedOn::nothing()).with_settings(
            &strings,
            Some(SettingsShows {
                window,
                filling_in: None,
                takes_the_keys: true,
                now: noon(),
            }),
        ),
        pressed: RefCell::new(Vec::new()),
    }
}

/// alo's own surfaces, as a command names them.
const ALO: Whose<'static> = Whose::AlosOwn {
    answers_as: ANSWERS_AS,
};

/// A setting's old name, in the person's language.
fn old_name(setting: Setting) -> String {
    crate::settings_page::said(&settings_words(), setting.word())
}

/// Settings open on a machine whose access file can be kept, on Accessibility.
fn on_accessibility(named: &str) -> (crate::settings_testing::Machine, SettingsWindow) {
    let machine = a_persons_machine(named);
    let mut window = machine.opened();
    window.the_applications_are(a_browser_and_a_mail_app());
    let _ = window.pointed_at(
        &PointedAt::ACategory(SettingsCategory::Accessibility),
        noon(),
    );
    (machine, window)
}

/// **A drawn row keeps its old name**: *stop things sliding and fading* still
/// presses Reduced motion, and the command is told what it is called now. Its
/// drawn name presses it too, with no notice.
#[test]
fn the_old_name_of_a_drawn_row_reaches_it_and_says_its_new_name() {
    let (_machine, window) = on_accessibility("old-name-drawn");
    let reading = on(&window);
    let strings = settings_words();
    let now_called =
        crate::settings_page::said(&strings, crate::the_settings_words::REDUCED_MOTION);

    let pressed = press_by_hand(
        &reading,
        ALO,
        Some("settings"),
        Pressable::Switch,
        &old_name(Setting::ReducedMotion),
    )
    .expect("the old name presses the switch");
    assert_eq!(pressed.now_called(), Some(now_called.as_str()));

    let pressed = press_by_hand(
        &reading,
        ALO,
        Some("settings"),
        Pressable::Switch,
        &now_called,
    )
    .expect("the drawn name presses the switch");
    assert_eq!(
        pressed.now_called(),
        None,
        "the drawn name was told it is old"
    );

    assert_eq!(
        *reading.pressed.borrow(),
        vec![Pressed::InSettings(InSettings::ARow(SettingsRow::ReducedMotion)); 2],
        "one press each, and only on Reduced motion"
    );
}

/// **The screen reader's switch keeps its old name too**: *read the screen
/// aloud* presses the Screen reader switch the 2026-11-12 row draws, told what
/// it is called now — the row reached its old name by saying which setting it
/// is, and nothing else changed.
#[test]
fn the_old_name_of_the_screen_reader_reaches_its_switch() {
    let (_machine, window) = on_accessibility("old-name-screen-reader");
    let reading = on(&window);
    let now_called =
        crate::settings_page::said(&settings_words(), crate::the_settings_words::SCREEN_READER);
    let pressed = press_by_hand(
        &reading,
        ALO,
        Some("settings"),
        Pressable::Switch,
        &old_name(Setting::ScreenReader),
    )
    .expect("the old name presses the switch");
    assert_eq!(pressed.now_called(), Some(now_called.as_str()));
    assert_eq!(
        *reading.pressed.borrow(),
        vec![Pressed::InSettings(InSettings::ARow(
            SettingsRow::ScreenReader
        ))],
        "one press, and only on Screen reader"
    );
}
/// **A setting no page draws yet is refused by its own sentence**, and nothing
/// is pressed — for each of the seven, by the name it had.
#[test]
fn a_setting_no_page_draws_yet_is_refused_and_nothing_is_pressed() {
    let (_machine, window) = on_accessibility("old-name-not-drawn");
    let reading = on(&window);
    for setting in Setting::ALL {
        if matches!(setting, Setting::ReducedMotion | Setting::ScreenReader) {
            continue;
        }
        assert_eq!(
            press_by_hand(
                &reading,
                ALO,
                Some("settings"),
                Pressable::Switch,
                &old_name(setting)
            )
            .unwrap_err(),
            Unpressed::NotPressed(NotPressed::NotDrawnYet),
            "{setting:?}"
        );
    }
    assert!(
        reading.pressed.borrow().is_empty(),
        "a refusal pressed something"
    );
    assert_eq!(
        press_by_hand(
            &reading,
            ALO,
            Some("settings"),
            Pressable::Switch,
            "no such setting"
        )
        .unwrap_err(),
        Unpressed::NotPressed(NotPressed::NoSuchControl),
        "a name that was never one is not told it is an old setting"
    );
}

/// **Both states, and the move between them**: where the person's access file
/// cannot be kept, Reduced motion is drawn on no page, so its old name is
/// refused as not drawn yet — and on a machine where it is drawn, the same
/// name reaches it. The tree carries each setting exactly once: on its row, or
/// on Settings as not drawn yet.
#[test]
fn a_setting_is_on_its_row_or_listed_as_not_drawn_and_never_both() {
    let strings = settings_words();
    let machine = a_persons_machine("old-name-both-states");
    let mut drawn = machine.opened();
    let mut not_drawn = SettingsWindow::closed();
    let _ = not_drawn.opened_by_hand(&machine.with_no_folder(), noon());
    for window in [&mut drawn, &mut not_drawn] {
        let _ = window.pointed_at(
            &PointedAt::ACategory(SettingsCategory::Accessibility),
            noon(),
        );
    }
    for (window, reduced_motion_is_drawn) in [(&drawn, true), (&not_drawn, false)] {
        let open = window.open().unwrap();
        let rows = open.rows(noon());
        assert_eq!(
            rows.contains(&SettingsRow::ReducedMotion),
            reduced_motion_is_drawn,
            "the fixture is not the state this test means"
        );
        let listed = the_settings_not_drawn_yet(open, &strings, noon());
        for setting in Setting::ALL {
            let on_a_row = rows.iter().any(|row| row.the_setting() == Some(setting));
            let on_settings = listed
                .iter()
                .any(|(key, _)| alo_access::not_drawn_yet(key) == Some(setting));
            assert_ne!(
                on_a_row, on_settings,
                "{setting:?} is on both, or on neither"
            );
        }
        let reading = on(window);
        let answered = press_by_hand(
            &reading,
            ALO,
            Some("settings"),
            Pressable::Switch,
            &old_name(Setting::ReducedMotion),
        );
        if reduced_motion_is_drawn {
            assert!(answered.is_ok(), "{answered:?}");
        } else {
            assert_eq!(
                answered.unwrap_err(),
                Unpressed::NotPressed(NotPressed::NotDrawnYet)
            );
        }
    }
}

/// **The window and its list keep their names**: `--in settings` reaches
/// Settings, which a command reads under the window's drawn name with the old
/// name *what can be changed* still on its list of pages.
#[test]
fn the_window_and_its_list_of_pages_keep_their_names() {
    let strings = settings_words();
    let (_machine, window) = on_accessibility("old-name-window");
    let reading = on(&window);
    let shown = read_by_hand(&reading, ALO, Some("settings")).expect("Settings is reached");
    assert_eq!(shown.windows().len(), 1);
    let settings = reading
        .tree
        .the_surfaces_node(Surface::Settings)
        .and_then(|at| reading.tree.nodes().get(at))
        .unwrap();
    assert_eq!(
        reading
            .tree
            .attributes_of(settings)
            .get(alo_access::THE_OLD_NAME_ATTRIBUTE)
            .map(String::as_str),
        Some(crate::settings_page::said(&strings, alo_access::words::SETTINGS).as_str())
    );
    let list = crate::settings_page::said(&strings, alo_access::words::WHAT_CAN_BE_CHANGED);
    assert!(
        reading.tree.nodes().iter().any(|node| node.name == list),
        "nothing on Settings is called {list:?} any more"
    );
}

/// **A choice row is pressed by its drawn name**, as a radio button — which is
/// what makes Default apps reachable from the terminal at all.
#[test]
fn a_default_apps_row_is_pressed_as_a_radio_button() {
    let (_machine, mut window) = on_accessibility("old-name-radio");
    let _ = window.pointed_at(&PointedAt::ACategory(SettingsCategory::DefaultApps), noon());
    let reading = on(&window);
    let pressed = press_by_hand(
        &reading,
        ALO,
        Some("settings"),
        Pressable::RadioButton,
        "Firefox",
    )
    .expect("Firefox is a choice on Default apps");
    assert_eq!(pressed.now_called(), None);
    assert!(matches!(
        reading.pressed.borrow().as_slice(),
        [Pressed::InSettings(InSettings::ARow(SettingsRow::DefaultFor(_, identifier)))]
            if identifier == "org.mozilla.firefox"
    ));
}
