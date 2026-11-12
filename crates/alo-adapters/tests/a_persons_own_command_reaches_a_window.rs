//! **A person's own command reaches an application's windows by the same walk
//! and the same press alo does**, and is refused for the same reasons
//! (`docs/contracts/terminal-commands.md`, section 2).
//!
//! What is added for a person, each tested here with its refusal: alo OS's own
//! surfaces found by the unique name the shell's tree answers as and never by a
//! name a client chooses, and `--in` narrowing to one window or one surface.

#![expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::cell::RefCell;
use std::collections::BTreeMap;

use alo_adapters::{
    AccessibilityTree, Contents, Facts, NodeAt, NotPressed, NotWalked, Pressable, Running, States,
    TreeFault, Unpressed, Whose, WorkingFully, press_by_hand, read_by_hand,
};
use alo_strings::{Strings, Vocabulary};

/// The application the tests reach.
const MAIL: &str = "org.example.Mail";

/// What is typed into its password field.
const SECRET: &str = "correct-horse-battery-staple";

/// The unique name alo's own shell answers as.
const ALO: &str = ":1.2";

/// `AtspiRole` numbers the tests write.
mod roles {
    /// A frame: a window.
    pub const FRAME: u32 = 23;
    /// A label.
    pub const LABEL: u32 = 29;
    /// A password field.
    pub const PASSWORD_TEXT: u32 = 40;
    /// A button.
    pub const PUSH_BUTTON: u32 = 43;
    /// A switch.
    pub const SWITCH: u32 = 130;
    /// A toggle button.
    pub const TOGGLE_BUTTON: u32 = 62;
    /// A menu item.
    pub const MENU_ITEM: u32 = 35;
    /// A tab.
    pub const PAGE_TAB: u32 = 37;
    /// An application's root.
    pub const APPLICATION: u32 = 75;
}

/// One thing in a window, as a test writes it.
struct Thing {
    /// Its role.
    role: u32,
    /// Its name.
    name: &'static str,
    /// Its states.
    states: States,
    /// Its text, if it has any.
    text: Option<&'static str>,
    /// What can be done to it.
    actions: &'static [&'static str],
    /// What it says about itself besides its name.
    attributes: Vec<(&'static str, &'static str)>,
    /// What is inside it.
    children: Vec<Thing>,
}

/// A thing with a role and a name, showing and usable.
fn thing(role: u32, name: &'static str) -> Thing {
    Thing {
        role,
        name,
        states: States::showing_and_usable(),
        text: None,
        actions: &[],
        attributes: Vec::new(),
        children: Vec::new(),
    }
}

impl Thing {
    /// With this text.
    fn saying(mut self, text: &'static str) -> Self {
        self.text = Some(text);
        self
    }
    /// With these actions.
    fn doing(mut self, actions: &'static [&'static str]) -> Self {
        self.actions = actions;
        self
    }
    /// With these inside.
    fn holding(mut self, children: Vec<Thing>) -> Self {
        self.children = children;
        self
    }
    /// In these states.
    fn in_states(mut self, states: States) -> Self {
        self.states = states;
        self
    }
    /// Saying this about itself.
    fn marked(mut self, key: &'static str, value: &'static str) -> Self {
        self.attributes.push((key, value));
        self
    }
}

/// A stand-in accessibility tree.
#[derive(Default)]
struct ATree {
    /// The applications listed.
    running: Vec<Running>,
    /// Each thing's facts.
    facts: BTreeMap<NodeAt, Facts>,
    /// Each thing's text.
    texts: BTreeMap<NodeAt, String>,
    /// Each thing's actions.
    actions: BTreeMap<NodeAt, Vec<String>>,
    /// Each thing's attributes.
    attributes: BTreeMap<NodeAt, Vec<(String, String)>>,
    /// What pressing answers.
    presses: Option<Result<bool, TreeFault>>,
    /// Every text asked for.
    texts_asked: RefCell<Vec<NodeAt>>,
    /// How many presses were asked.
    pressed: RefCell<usize>,
    /// The next node's number.
    next: usize,
}

impl ATree {
    /// Add an application on `holder`, named by its sandbox as `is`.
    fn with(mut self, holder: &str, is: Option<&str>, windows: Vec<Thing>) -> Self {
        let root = NodeAt {
            holder: holder.to_owned(),
            object: format!("/root/{}", self.running.len()),
        };
        self.running.push(Running {
            at: root.clone(),
            is: is.map(ToOwned::to_owned),
        });
        self.add(holder, root, thing(roles::APPLICATION, "").holding(windows));
        self
    }

    /// Add one thing and everything inside it.
    fn add(&mut self, holder: &str, at: NodeAt, thing: Thing) {
        let mut children = Vec::new();
        for child in thing.children {
            self.next += 1;
            let child_at = NodeAt {
                holder: holder.to_owned(),
                object: format!("/thing/{}", self.next),
            };
            children.push(child_at.clone());
            self.add(holder, child_at, child);
        }
        if let Some(text) = thing.text {
            self.texts.insert(at.clone(), text.to_owned());
        }
        self.actions.insert(
            at.clone(),
            thing.actions.iter().map(|a| (*a).to_owned()).collect(),
        );
        self.attributes.insert(
            at.clone(),
            thing
                .attributes
                .iter()
                .map(|(k, v)| ((*k).to_owned(), (*v).to_owned()))
                .collect(),
        );
        self.facts.insert(
            at,
            Facts {
                role: thing.role,
                name: thing.name.to_owned(),
                states: thing.states.as_the_tree_says().to_vec(),
                children,
                has_text: thing.text.is_some(),
                unlisted_children: false,
            },
        );
    }

    /// Pressing answers this.
    fn pressing_answers(mut self, answer: Result<bool, TreeFault>) -> Self {
        self.presses = Some(answer);
        self
    }
}

impl AccessibilityTree for ATree {
    fn applications(&self) -> Result<Vec<Running>, TreeFault> {
        Ok(self.running.clone())
    }
    fn facts(&self, at: &NodeAt) -> Result<Facts, TreeFault> {
        self.facts.get(at).cloned().ok_or(TreeFault::Vanished)
    }
    fn text(&self, at: &NodeAt) -> Result<String, TreeFault> {
        self.texts_asked.borrow_mut().push(at.clone());
        self.texts.get(at).cloned().ok_or(TreeFault::Vanished)
    }
    fn actions(&self, at: &NodeAt) -> Result<Vec<String>, TreeFault> {
        Ok(self.actions.get(at).cloned().unwrap_or_default())
    }
    fn act(&self, _at: &NodeAt, _action: usize) -> Result<bool, TreeFault> {
        *self.pressed.borrow_mut() += 1;
        self.presses.unwrap_or(Ok(true))
    }
    fn attributes(&self, at: &NodeAt) -> Result<Vec<(String, String)>, TreeFault> {
        Ok(self.attributes.get(at).cloned().unwrap_or_default())
    }
}

/// The words a shell has.
fn in_english() -> Strings {
    let mut vocabulary = Vocabulary::empty();
    alo_adapters::words::declare_into(&mut vocabulary).unwrap();
    Strings::of(vocabulary)
}

/// The mail application with two windows; an impostor naming itself after it;
/// and alo's own shell, beside a client that calls itself alo.
fn the_desktop() -> ATree {
    ATree::default()
        .with(
            ":1.7",
            Some(MAIL),
            vec![
                thing(roles::FRAME, "Inbox").holding(vec![
                    thing(roles::PUSH_BUTTON, "Send").doing(&["click"]),
                    thing(roles::PUSH_BUTTON, "Help").doing(&["click"]),
                    thing(roles::PUSH_BUTTON, "Help").doing(&["click"]),
                    thing(roles::PUSH_BUTTON, "Archive")
                        .doing(&["click"])
                        .in_states(States::showing_and_usable().greyed_out()),
                    thing(roles::PUSH_BUTTON, "Details"),
                    thing(roles::PASSWORD_TEXT, "Password").saying(SECRET),
                    thing(roles::LABEL, "From").saying("anna@example.org"),
                ]),
                thing(roles::FRAME, "Compose")
                    .holding(vec![thing(roles::PUSH_BUTTON, "Send").doing(&["click"])]),
            ],
        )
        .with(
            ":1.9",
            None,
            vec![
                thing(roles::FRAME, "Inbox")
                    .holding(vec![thing(roles::PUSH_BUTTON, "Send").doing(&["click"])]),
            ],
        )
        .with(
            ALO,
            None,
            vec![
                thing(roles::FRAME, "Settings")
                    .marked(alo_access::THE_SURFACE_ATTRIBUTE, "settings")
                    .holding(vec![thing(roles::SWITCH, "Wi-Fi").doing(&["click"])]),
                thing(roles::FRAME, "")
                    .marked(alo_access::THE_SURFACE_ATTRIBUTE, "desktop")
                    .in_states(States::showing_and_usable().hidden()),
            ],
        )
        .with(
            ":1.40",
            None,
            vec![
                thing(roles::FRAME, "settings")
                    .marked(alo_access::THE_SURFACE_ATTRIBUTE, "settings")
                    .holding(vec![thing(roles::SWITCH, "Wi-Fi").doing(&["click"])]),
            ],
        )
}

/// **What a person reads is the application's own windows**, and never the
/// impostor's, and never a password field's contents.
#[test]
fn a_person_reads_the_applications_windows_and_never_a_password() {
    let tree = the_desktop();
    let shown = read_by_hand(&tree, Whose::Application(MAIL), None).unwrap();
    let titles: Vec<&str> = shown.windows().iter().map(|w| w.title()).collect();
    assert_eq!(titles, ["Inbox", "Compose"]);
    assert_eq!(shown.application(), MAIL);
    for seen in shown.everything() {
        if let Contents::Text(text) = seen.contents() {
            assert_ne!(text, SECRET, "a password field's contents were read");
        }
    }
    let asked = tree.texts_asked.borrow();
    assert!(
        asked
            .iter()
            .all(|at| tree.texts.get(at).map(String::as_str) != Some(SECRET)),
        "a password field's text was asked for"
    );
}

/// **`--in` narrows to the one window with that title**, and refuses rather
/// than guesses when none or more than one has it.
#[test]
fn in_narrows_to_one_window_and_refuses_none_or_two() {
    let tree = the_desktop();
    let shown = read_by_hand(&tree, Whose::Application(MAIL), Some("Compose")).unwrap();
    let titles: Vec<&str> = shown.windows().iter().map(|w| w.title()).collect();
    assert_eq!(titles, ["Compose"]);

    // `Send` is in both windows: one press without `--in` is refused, and
    // with it is exactly one press.
    let unnamed = press_by_hand(
        &tree,
        Whose::Application(MAIL),
        None,
        Pressable::Button,
        "Send",
    );
    assert_eq!(unnamed, Err(Unpressed::NotPressed(NotPressed::MoreThanOne)));
    assert_eq!(*tree.pressed.borrow(), 0);
    press_by_hand(
        &tree,
        Whose::Application(MAIL),
        Some("Compose"),
        Pressable::Button,
        "Send",
    )
    .unwrap();
    assert_eq!(*tree.pressed.borrow(), 1);

    assert_eq!(
        read_by_hand(&tree, Whose::Application(MAIL), Some("Outbox")).unwrap_err(),
        NotWalked::NoSuchWindow
    );
    assert_eq!(
        read_by_hand(&tree, Whose::Application(MAIL), Some("compose")).unwrap_err(),
        NotWalked::NoSuchWindow,
        "a title was matched without its case"
    );
    let twice = ATree::default().with(
        ":1.7",
        Some(MAIL),
        vec![thing(roles::FRAME, "Inbox"), thing(roles::FRAME, "Inbox")],
    );
    assert_eq!(
        read_by_hand(&twice, Whose::Application(MAIL), Some("Inbox")).unwrap_err(),
        NotWalked::MoreThanOneWindow
    );

    let strings = in_english();
    assert_eq!(
        NotWalked::NoSuchWindow
            .said(MAIL, Some("Outbox"), &strings)
            .text(),
        "org.example.Mail shows no window called Outbox, so nothing was read or pressed"
    );
    assert!(
        NotWalked::MoreThanOneWindow
            .said(MAIL, Some("Inbox"), &strings)
            .text()
            .starts_with("org.example.Mail shows more than one window called Inbox")
    );
}

/// **Every refusal the agent's road has, a person's command has**, and none
/// of them presses anything.
#[test]
fn every_refusal_is_the_agents_and_nothing_is_pressed() {
    let tree = the_desktop();
    let mail = Whose::Application(MAIL);
    let refused = |kind, name| press_by_hand(&tree, mail, Some("Inbox"), kind, name);
    assert_eq!(
        refused(Pressable::Button, "Reply"),
        Err(Unpressed::NotPressed(NotPressed::NoSuchControl))
    );
    assert_eq!(
        refused(Pressable::Button, "Help"),
        Err(Unpressed::NotPressed(NotPressed::MoreThanOne))
    );
    assert_eq!(
        refused(Pressable::Button, "Archive"),
        Err(Unpressed::NotPressed(NotPressed::CannotBeUsedNow))
    );
    assert_eq!(
        refused(Pressable::Button, "Details"),
        Err(Unpressed::NotPressed(NotPressed::CannotBePressed))
    );
    assert_eq!(
        refused(Pressable::Switch, "Send"),
        Err(Unpressed::NotPressed(NotPressed::NoSuchControl)),
        "a control was found under another kind"
    );
    assert_eq!(
        *tree.pressed.borrow(),
        0,
        "a refused press pressed something"
    );

    let saying_no = the_desktop().pressing_answers(Ok(false));
    assert_eq!(
        press_by_hand(&saying_no, mail, Some("Inbox"), Pressable::Button, "Send"),
        Err(Unpressed::NotPressed(NotPressed::DidNotPress))
    );
    let silent = the_desktop().pressing_answers(Err(TreeFault::DidNotAnswer));
    let sent = press_by_hand(&silent, mail, Some("Inbox"), Pressable::Button, "Send").unwrap();
    assert!(
        sent.unanswered(),
        "an unanswered press was reported as answered"
    );

    assert_eq!(
        press_by_hand(
            &the_desktop(),
            Whose::Application("org.example.NotRunning"),
            None,
            Pressable::Button,
            "Send"
        ),
        Err(Unpressed::NotWalked(NotWalked::NoWindow))
    );

    let strings = in_english();
    assert_eq!(
        Unpressed::NotPressed(NotPressed::NoSuchControl)
            .said(
                mail,
                Some("Inbox"),
                Pressable::Button,
                "Reply",
                WorkingFully::NothingMoreToTurnOn,
                &strings
            )
            .text(),
        NotPressed::NoSuchControl
            .said_of(
                MAIL,
                Pressable::Button,
                "Reply",
                WorkingFully::NothingMoreToTurnOn,
                &strings
            )
            .text()
    );
}

/// **alo's own surfaces are the shell's tree, by the name it answers as**, and
/// `--in` names a surface by its typed name — never a window title, and never
/// a client that calls itself alo.
#[test]
fn alos_own_surfaces_are_found_by_the_shells_name_and_by_surface() {
    let tree = the_desktop();
    let alo = Whose::AlosOwn { answers_as: ALO };
    let settings = read_by_hand(&tree, alo, Some("settings")).unwrap();
    assert_eq!(settings.application(), "alo");
    let titles: Vec<&str> = settings.windows().iter().map(|w| w.title()).collect();
    assert_eq!(titles, ["Settings"]);

    press_by_hand(&tree, alo, Some("settings"), Pressable::Switch, "Wi-Fi").unwrap();
    assert_eq!(
        *tree.pressed.borrow(),
        1,
        "the shell's switch was not pressed exactly once"
    );

    assert_eq!(
        read_by_hand(&tree, alo, Some("desktop")).unwrap_err(),
        NotWalked::NoSuchWindow,
        "a surface not on the screen was read"
    );
    assert_eq!(
        read_by_hand(&tree, alo, Some("Settings")).unwrap_err(),
        NotWalked::NoSuchWindow,
        "a surface was reached by its title rather than its typed name"
    );

    // A client beside the shell marks itself as Settings too: had it been
    // read, `--in settings` above would have found two and refused. Only the
    // name the compositor gave is alo's.
    // And a name that is not a unique name is never taken as alo's.
    let well_known = ATree::default().with(
        "org.alo.Shell",
        None,
        vec![thing(roles::FRAME, "Settings").marked(alo_access::THE_SURFACE_ATTRIBUTE, "settings")],
    );
    assert_eq!(
        read_by_hand(
            &well_known,
            Whose::AlosOwn {
                answers_as: "org.alo.Shell"
            },
            None
        )
        .unwrap_err(),
        NotWalked::NoWindow
    );
    assert_eq!(Whose::AlosOwn { answers_as: ALO }.named(), "alo");
}

/// The editor drawn by Chromium the tests reach.
const CODIUM: &str = "com.vscodium.codium";

/// **VSCodium's window as Chromium describes it**: in its complete mode a toggle
/// button offers `check` or `uncheck` and a menu item `select`; in its basic
/// mode a tab offers only `doDefault`, which answers *done* and does nothing.
/// The actions are the ones measured on 2026-11-12
/// (`docs/quirks/vscodium-is-pressed-and-read-only-when-described-completely.md`).
fn vscodium(toggle: &'static [&'static str]) -> ATree {
    ATree::default().with(
        ":1.30",
        Some(CODIUM),
        vec![thing(roles::FRAME, "README.md - VSCodium").holding(vec![
            thing(roles::TOGGLE_BUTTON, "Toggle Panel (Ctrl+J)").doing(toggle),
            thing(roles::MENU_ITEM, "File").doing(&["select", "showContextMenu"]),
            thing(roles::PAGE_TAB, "Search (Ctrl+Shift+F)")
                .doing(&["doDefault", "showContextMenu"]),
        ])],
    )
}

/// **A toggle button is pressed by whichever of `check` and `uncheck` it
/// offers, and a menu item by `select`** — each once — so a press always turns
/// a toggle over. **`doDefault` is never a press**: a control offering only it
/// is refused, and nothing is sent to it.
#[test]
fn chromiums_complete_mode_is_pressed_and_its_do_default_is_not() {
    let codium = Whose::Application(CODIUM);
    for toggle in [
        &["check", "showContextMenu"][..],
        &["uncheck", "showContextMenu"][..],
    ] {
        let tree = vscodium(toggle);
        let pressed = press_by_hand(
            &tree,
            codium,
            None,
            Pressable::Button,
            "Toggle Panel (Ctrl+J)",
        );
        assert!(pressed.is_ok(), "{toggle:?}: {pressed:?}");
        assert_eq!(*tree.pressed.borrow(), 1, "{toggle:?}");
    }

    let tree = vscodium(&["check", "showContextMenu"]);
    assert!(press_by_hand(&tree, codium, None, Pressable::MenuItem, "File").is_ok());
    assert_eq!(
        *tree.pressed.borrow(),
        1,
        "select did not press the menu item"
    );

    assert_eq!(
        press_by_hand(&tree, codium, None, Pressable::Tab, "Search (Ctrl+Shift+F)"),
        Err(Unpressed::NotPressed(NotPressed::CannotBePressed))
    );
    assert_eq!(
        *tree.pressed.borrow(),
        1,
        "doDefault was sent, and would have reported a press that did not happen"
    );
    assert!(
        !alo_adapters::PRESSING
            .iter()
            .any(|word| word.eq_ignore_ascii_case("doDefault")),
        "doDefault is a press"
    );
}

/// **Where turning on *Let alo work fully inside apps like VSCodium* would give
/// the control a press, the refusal names that switch**; where it would not,
/// the refusal is the plain one, and no other refusal changes either way.
#[test]
fn a_control_that_cannot_be_pressed_names_the_switch_only_where_it_would_help() {
    let strings = in_english();
    let said = |why: NotPressed, fully| {
        why.said_of(CODIUM, Pressable::Tab, "Search", fully, &strings)
            .text()
            .to_owned()
    };
    let naming = said(NotPressed::CannotBePressed, WorkingFully::CouldBeTurnedOn);
    assert!(
        naming.contains("“Let alo work fully inside apps like VSCodium”"),
        "{naming}"
    );
    assert!(
        naming.contains("Settings → Accessibility → Applications"),
        "{naming}"
    );
    assert!(naming.contains("nothing was pressed"), "{naming}");
    assert!(naming.contains(CODIUM), "{naming}");

    let plain = said(
        NotPressed::CannotBePressed,
        WorkingFully::NothingMoreToTurnOn,
    );
    assert!(!plain.contains("Let alo work fully"), "{plain}");
    assert!(plain.contains("nothing was pressed"), "{plain}");

    for other in [
        NotPressed::NoSuchControl,
        NotPressed::MoreThanOne,
        NotPressed::TooMuchToBeSure,
        NotPressed::CannotBeUsedNow,
        NotPressed::DidNotPress,
        NotPressed::NotDrawnYet,
    ] {
        assert_eq!(
            said(other, WorkingFully::CouldBeTurnedOn),
            said(other, WorkingFully::NothingMoreToTurnOn),
            "{other:?} named a switch that would not help"
        );
    }

    assert_eq!(
        WorkingFully::for_one(true, false),
        WorkingFully::CouldBeTurnedOn
    );
    for (reached, turned_on) in [(true, true), (false, false), (false, true)] {
        assert_eq!(
            WorkingFully::for_one(reached, turned_on),
            WorkingFully::NothingMoreToTurnOn,
            "reached = {reached}, turned on = {turned_on}"
        );
    }
}

/// alo's Settings as the shell publishes it from 2026-11-12: a drawn switch
/// marked with the name it had, a setting no page draws listed on the window —
/// and an application whose own switch carries the same mark, which no command
/// may read as alo's.
fn settings_with_old_names() -> ATree {
    ATree::default()
        .with(
            ALO,
            None,
            vec![
                thing(roles::FRAME, "Settings")
                    .marked(alo_access::THE_SURFACE_ATTRIBUTE, "settings")
                    .marked("alo-not-drawn-yet:screen-reader", "read the screen aloud")
                    .holding(vec![
                        thing(roles::SWITCH, "Reduced motion")
                            .marked(
                                alo_access::THE_OLD_NAME_ATTRIBUTE,
                                "stop things sliding and fading",
                            )
                            .doing(&["click"]),
                    ]),
            ],
        )
        .with(
            ":1.7",
            Some(MAIL),
            vec![thing(roles::FRAME, "Inbox").holding(vec![
                thing(roles::SWITCH, "Threads")
                    .marked(alo_access::THE_OLD_NAME_ATTRIBUTE, "Conversations")
                    .doing(&["click"]),
            ])],
        )
}

/// **An old name on alo's Settings reaches the control it named, and says what
/// it is called now**; the drawn name reaches it with nothing said; a setting
/// no page draws is refused by its own sentence; and an application's tree is
/// never read for old names, whatever it marks.
#[test]
fn an_old_name_on_alos_settings_reaches_its_control_and_nowhere_else() {
    let tree = settings_with_old_names();
    let alo = Whose::AlosOwn { answers_as: ALO };

    let by_its_old_name = press_by_hand(
        &tree,
        alo,
        Some("settings"),
        Pressable::Switch,
        "stop things sliding and fading",
    )
    .unwrap();
    assert_eq!(by_its_old_name.now_called(), Some("Reduced motion"));
    let by_its_name = press_by_hand(
        &tree,
        alo,
        Some("settings"),
        Pressable::Switch,
        "Reduced motion",
    )
    .unwrap();
    assert_eq!(by_its_name.now_called(), None);
    assert_eq!(*tree.pressed.borrow(), 2);

    assert_eq!(
        press_by_hand(
            &tree,
            alo,
            Some("settings"),
            Pressable::Switch,
            "read the screen aloud"
        ),
        Err(Unpressed::NotPressed(NotPressed::NotDrawnYet))
    );
    assert_eq!(
        press_by_hand(
            &tree,
            Whose::Application(MAIL),
            None,
            Pressable::Switch,
            "Conversations"
        ),
        Err(Unpressed::NotPressed(NotPressed::NoSuchControl)),
        "an application's own mark was read as an old name"
    );
    assert_eq!(*tree.pressed.borrow(), 2, "a refusal pressed something");

    let strings = in_english();
    let said = NotPressed::NotDrawnYet
        .said_of(
            "alo",
            Pressable::Switch,
            "read the screen aloud",
            WorkingFully::NothingMoreToTurnOn,
            &strings,
        )
        .into_text();
    assert!(said.contains("read the screen aloud"), "{said}");
    assert!(said.contains("nothing was pressed"), "{said}");
}
