//! **A reader hears each Settings page as it is drawn** — Default apps first,
//! then every page the shared page reaches — with which choice is chosen, its
//! place among its set, and where the keyboard is; and hears nothing of
//! Settings while it is not open.

use alo_access::{Surface, TurnedOn};
use alo_setting_up::a_provider_is_filled_in::AProviderBeingAddded;
use alo_strings::Strings;

use super::*;
use crate::settings_pointed_at::PointedAt;
use crate::settings_testing::{
    Machine, a_browser_and_a_mail_app, a_persons_machine, noon, settings_words,
};

/// Settings, opened on a machine with a browser and a mail application.
fn opened(named: &str) -> (Machine, SettingsWindow) {
    let machine = a_persons_machine(named);
    let mut window = machine.opened();
    window.the_applications_are(a_browser_and_a_mail_app());
    (machine, window)
}

/// The page `category` shown, the keyboard in the side column.
fn on_the_page(window: &mut SettingsWindow, category: SettingsCategory) {
    let _ = window.pointed_at(&PointedAt::ACategory(category), noon());
}

/// The tree a reader is given, with Settings as it is now.
fn read(strings: &Strings, window: &SettingsWindow, takes_the_keys: bool) -> ReadAloudTree {
    ReadAloudTree::of(strings, &[Surface::Desktop], &TurnedOn::nothing()).with_settings(
        strings,
        Some(SettingsShows {
            window,
            filling_in: None,
            takes_the_keys,
            now: noon(),
        }),
    )
}

/// Settings' own node, and what hangs under it, in reading order.
fn settings_of(tree: &ReadAloudTree) -> (&Node, Vec<&Node>) {
    let at = tree.the_surfaces_node(Surface::Settings).unwrap();
    let window = &tree.nodes()[at];
    let mut under = Vec::new();
    let mut looking: Vec<usize> = window.children.iter().rev().copied().collect();
    while let Some(next) = looking.pop() {
        let node = &tree.nodes()[next];
        under.push(node);
        looking.extend(node.children.iter().rev().copied());
    }
    (window, under)
}

/// The node called `name`, of `role`, under Settings.
fn the<'t>(under: &[&'t Node], role: Role, name: &str) -> &'t Node {
    under
        .iter()
        .find(|node| node.role == access_roles::number_of(role) && node.name == name)
        .copied()
        .unwrap_or_else(|| {
            let heard: Vec<&str> = under.iter().map(|node| node.name.as_str()).collect();
            panic!("no {role:?} called {name:?}; a reader hears {heard:?}")
        })
}

/// One attribute's value.
fn attribute<'n>(node: &'n Node, key: &str) -> Option<&'n str> {
    node.attributes
        .iter()
        .find_map(|(k, v)| (k == key).then_some(v.as_str()))
}

/// **Default apps is heard as it is drawn**: its heading, both sections'
/// headings, and every application offered by its own name, a radio button
/// chosen exactly when the page rings it, with its line and its place.
#[test]
fn default_apps_is_heard_as_it_is_drawn() {
    let strings = settings_words();
    let (_machine, mut window) = opened("reader-default-apps");
    on_the_page(&mut window, SettingsCategory::DefaultApps);
    let tree = read(&strings, &window, false);
    let (_, under) = settings_of(&tree);

    let page = crate::settings_page::the_page(
        window.open().unwrap(),
        SettingsCategory::DefaultApps,
        &strings,
        noon(),
    );
    let heading = the(&under, Role::Heading, &page.heading);
    assert_eq!(attribute(heading, "level"), Some("1"));
    let mut rows_heard = 0;
    for block in &page.blocks {
        let ABlock::Section(section) = block else {
            continue;
        };
        if let Some(heading) = &section.heading {
            assert_eq!(
                attribute(the(&under, Role::Heading, heading), "level"),
                Some("2")
            );
        }
        let choices = section.rows.len();
        for (place, row) in section.rows.iter().enumerate() {
            let TheKind::AChoice { chosen } = row.kind else {
                panic!("a Default apps row is not a choice: {row:?}");
            };
            let node = the(&under, Role::RadioButton, &row.title);
            assert_eq!(
                access_roles::is_checked(node.states),
                chosen,
                "{}",
                row.title
            );
            assert_eq!(node.description, row.lines.join("\n"));
            assert_eq!(
                attribute(node, "posinset"),
                Some((place + 1).to_string().as_str())
            );
            assert_eq!(
                attribute(node, "setsize"),
                Some(choices.to_string().as_str())
            );
            assert_eq!(
                node.press,
                Some(Pressed::InSettings(InSettings::ARow(
                    row.row.clone().unwrap()
                )))
            );
            rows_heard += 1;
        }
    }
    assert_eq!(
        rows_heard, 2,
        "a browser and a mail application were offered"
    );
    for heading in [w::WEB_LINKS, w::EMAIL_ADDRESSES] {
        let _ = the(&under, Role::Heading, &said(&strings, heading));
    }
}

/// **Every page the shared page reaches is heard**: on each, every row a
/// person can act on is a node a reader can press, by the name drawn on it.
#[test]
fn every_page_is_heard_and_every_row_on_it_can_be_pressed() {
    let strings = settings_words();
    let (_machine, mut window) = opened("reader-every-page");
    for category in SettingsCategory::ALL {
        on_the_page(&mut window, category);
        let tree = read(&strings, &window, false);
        let (_, under) = settings_of(&tree);
        let page =
            crate::settings_page::the_page(window.open().unwrap(), category, &strings, noon());
        let _ = the(&under, Role::Heading, &page.heading);
        for row in page.rows() {
            let pressed = Pressed::InSettings(InSettings::ARow(row.clone()));
            assert!(
                under
                    .iter()
                    .any(|node| node.press.as_ref() == Some(&pressed)),
                "{category:?}: {row:?} is drawn and a reader cannot press it"
            );
        }
    }
}

/// **The side column is every page, the one shown selected, each with its
/// place** — and a press on one shows it.
#[test]
fn the_side_column_is_every_page_with_the_one_shown_selected() {
    let strings = settings_words();
    let (_machine, mut window) = opened("reader-side-column");
    on_the_page(&mut window, SettingsCategory::DefaultApps);
    let tree = read(&strings, &window, false);
    let (_, under) = settings_of(&tree);
    let tabs: Vec<&&Node> = under
        .iter()
        .filter(|node| node.role == access_roles::number_of(Role::Tab))
        .collect();
    assert_eq!(tabs.len(), SettingsCategory::ALL.len());
    for (place, category) in SettingsCategory::ALL.into_iter().enumerate() {
        let tab = tabs[place];
        assert_eq!(tab.name, category.said(&strings));
        assert_eq!(
            tab.states[0] & (1 << access_roles::SELECTED) != 0,
            category == SettingsCategory::DefaultApps,
            "{category:?}"
        );
        assert_eq!(
            attribute(tab, "posinset"),
            Some((place + 1).to_string().as_str())
        );
        assert_eq!(
            tab.press,
            Some(Pressed::InSettings(InSettings::APage(category)))
        );
    }
}

/// **The Reduced motion switch says which way it is set, and keeps its old
/// name**; the text size is a slider with its percentage.
#[test]
fn the_accessibility_page_says_which_way_its_switch_is_set() {
    let strings = settings_words();
    let (_machine, mut window) = opened("reader-accessibility");
    on_the_page(&mut window, SettingsCategory::Accessibility);
    let tree = read(&strings, &window, false);
    let (_, under) = settings_of(&tree);
    let switch = the(&under, Role::Switch, &said(&strings, w::REDUCED_MOTION));
    assert_eq!(
        window.motion_is_reduced(),
        Some(false),
        "as this computer comes"
    );
    assert!(!access_roles::is_checked(switch.states));
    assert_eq!(
        attribute(switch, alo_access::THE_OLD_NAME_ATTRIBUTE),
        Some(said(&strings, alo_access::words::REDUCED_MOTION).as_str())
    );
    let slider = the(&under, Role::Slider, &said(&strings, w::TEXT_SIZE));
    assert!(
        slider.description.contains('%'),
        "the text size is heard without its size: {:?}",
        slider.description
    );

    // **Turned on by the person's own press, it is told as on** — the value
    // read from the file the page draws it from, not one handed in.
    let _ = window.pointed_at(&PointedAt::ARow(SettingsRow::ReducedMotion), noon());
    let door = crate::settings_testing::CountingDoor::revoking();
    let _ = window.pressed(
        crate::SettingsPress {
            key: crate::SettingsKey::Choose,
            chord: None,
            filling: crate::a_key_fills_in_a_provider::FillingInKey::Nothing,
        },
        crate::settings_testing::doors(&strings, &door),
    );
    assert_eq!(
        window.motion_is_reduced(),
        Some(true),
        "the press did not turn it on"
    );
    let tree = read(&strings, &window, true);
    let (_, under) = settings_of(&tree);
    let switch = the(&under, Role::Switch, &said(&strings, w::REDUCED_MOTION));
    assert!(
        access_roles::is_checked(switch.states),
        "an on switch is heard as off"
    );
    assert!(access_roles::has_the_keyboard(switch.states));
}

/// **Settings is on the screen while it is open, and off it while it is
/// not** — and while it is closed a reader hears no row of it.
#[test]
fn settings_is_shown_while_it_is_open_and_not_otherwise() {
    let strings = settings_words();
    let (_machine, window) = opened("reader-shown");
    let open = read(&strings, &window, false);
    let (node, under) = settings_of(&open);
    assert!(access_roles::is_showing(node.states));
    assert_eq!(node.name, said(&strings, w::SETTINGS));
    assert!(under.len() > SettingsCategory::ALL.len());

    let closed = ReadAloudTree::of(&strings, &[Surface::Desktop], &TurnedOn::nothing())
        .with_settings(&strings, None);
    let (node, under) = settings_of(&closed);
    assert!(!access_roles::is_showing(node.states));
    assert!(
        under.iter().all(|node| node.press.is_none()),
        "a closed Settings can be pressed"
    );
}

/// **The keyboard is told where it is only while Settings takes the keys**:
/// the page shown in the side column, then the row it moves to.
#[test]
fn the_keyboard_is_where_settings_has_it_and_nowhere_while_it_has_none() {
    let strings = settings_words();
    let (_machine, mut window) = opened("reader-keyboard");
    on_the_page(&mut window, SettingsCategory::DefaultApps);
    let in_the_column = read(&strings, &window, true);
    let sheet = in_the_column.in_front().copied().unwrap();
    assert_eq!(sheet.which, TheSheet::Settings);
    let keyboard = &in_the_column.nodes()[sheet.keyboard];
    assert_eq!(keyboard.name, SettingsCategory::DefaultApps.said(&strings));
    assert!(access_roles::has_the_keyboard(keyboard.states));

    let first = window
        .rows_on_the_page(noon())
        .into_iter()
        .next()
        .expect("Default apps offers a row");
    let _ = window.pointed_at(&PointedAt::ARow(first.clone()), noon());
    let on_a_row = read(&strings, &window, true);
    let sheet = on_a_row.in_front().copied().unwrap();
    assert_eq!(
        on_a_row.nodes()[sheet.keyboard].press,
        Some(Pressed::InSettings(InSettings::ARow(first)))
    );
    let (told, _) = crate::access_events::what_happened(None, &on_a_row);
    assert!(!told.is_empty(), "Settings coming in front is not told");

    let behind = read(&strings, &window, false);
    assert!(behind.in_front().is_none());
    assert!(
        behind
            .nodes()
            .iter()
            .all(|node| !access_roles::has_the_keyboard(node.states)),
        "Settings behind another window still claims the keyboard"
    );
}

/// **The list a row opened is heard with the keyboard on its entry**, each
/// entry a choice that says whether it is what the machine is set to.
#[test]
fn an_open_list_is_heard_with_the_keyboard_on_its_entry() {
    let strings = settings_words();
    let (_machine, mut window) = opened("reader-picker");
    on_the_page(&mut window, SettingsCategory::RegionAndTime);
    window
        .open_mut()
        .unwrap()
        .the_list_opens(crate::SettingsPicker::Region, &strings);
    let tree = read(&strings, &window, true);
    let (_, under) = settings_of(&tree);
    let _ = the(&under, Role::Entry, &said(&strings, w::SEARCH_THE_LIST));
    let open = window.open().unwrap();
    let picker = open.picker.as_ref().unwrap();
    let entries = open.every_entry(picker.which, &strings);
    for entry in &entries {
        let node = the(&under, Role::RadioButton, &entry.label);
        assert_eq!(access_roles::is_checked(node.states), entry.chosen);
    }
    let sheet = tree.in_front().copied().unwrap();
    let focused = &tree.nodes()[sheet.keyboard];
    assert_eq!(
        focused.press,
        Some(Pressed::InSettings(InSettings::AnEntry(
            picker.shown[picker.focus].clone()
        )))
    );
}

/// **The form that adds a provider is heard in place of the page**, with the
/// keyboard on the field being filled in — and **nothing typed crosses**: the
/// key is a password field, and no field's contents are in any name or line.
#[test]
fn the_form_is_heard_and_what_is_typed_never_crosses() {
    let strings = settings_words();
    let (_machine, window) = opened("reader-form");
    let mut form = AProviderBeingAddded::empty();
    for letter in "harbour-name".chars() {
        let _ = form.typed(letter);
    }
    form.the_keyboard_is_in(alo_setting_up::a_provider_is_filled_in::WhichField::Key);
    for letter in "secret-sauce".chars() {
        let _ = form.typed(letter);
    }
    let tree = ReadAloudTree::of(&strings, &[Surface::Desktop], &TurnedOn::nothing())
        .with_settings(
            &strings,
            Some(SettingsShows {
                window: &window,
                filling_in: Some((&form, None)),
                takes_the_keys: true,
                now: noon(),
            }),
        );
    let (_, under) = settings_of(&tree);
    for field in alo_setting_up::a_provider_is_filled_in::WhichField::EVERY {
        let role = if field.is_a_secret() {
            Role::PasswordEntry
        } else {
            Role::Entry
        };
        let _ = the(
            &under,
            role,
            &crate::a_provider_being_added_is_drawn::label(field, &strings),
        );
    }
    for node in &under {
        for typed in ["secret-sauce", "harbour-name"] {
            assert!(
                !node.name.contains(typed) && !node.description.contains(typed),
                "what was typed crossed to a reader: {node:?}"
            );
        }
    }
    let sheet = tree.in_front().copied().unwrap();
    let keyboard = &tree.nodes()[sheet.keyboard];
    assert_eq!(keyboard.role, access_roles::number_of(Role::PasswordEntry));
    assert_eq!(
        keyboard.name,
        crate::a_provider_being_added_is_drawn::label(form.which_field(), &strings)
    );
}
