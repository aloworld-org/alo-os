//! The accessibility fallback, held to each clause of the plan's acceptance,
//! with every refusal beside what it refuses.
//!
//! `docs/autonomy/software-and-the-web-plan.md`, task 6:
//!
//! - **an application with no adapter is readable and operable through its
//!   accessibility tree, through two typed verbs** —
//!   [`the_two_verbs_are_a_read_and_a_change_and_take_nothing_else`],
//!   [`a_window_is_read_within_its_grant_and_recorded`],
//!   [`one_approval_presses_the_control_it_names_once_and_is_recorded`];
//! - **the activation is proposed as a sentence naming the control and the
//!   application, and approved like any change** — the same two;
//! - **the tree is read at invocation for that turn only, never watched** —
//!   [`with_no_invocation_the_tree_is_never_asked`],
//!   [`the_control_is_found_again_at_the_moment_it_is_pressed`];
//! - **a password field's contents are never read** —
//!   [`a_password_fields_contents_are_never_asked_for`], and against a real
//!   application in `the_accessibility_fallback_on_a_real_application.rs`;
//! - **what the fallback cannot do is said in words, never guessed at with
//!   coordinates** — [`what_cannot_be_read_is_said_in_words`] and
//!   [`nothing_is_guessed_when_the_control_is_not_the_only_one`].
//!
//! The tree here is a stand-in that keeps every question it is asked
//! ([`ATree`]), so what was asked is what it heard rather than what this crate
//! claims to have asked.

#![expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::cell::RefCell;
use std::collections::BTreeMap;
use std::time::{Duration, SystemTime};

use alo_adapters::{
    ACTIVATE_CONTROL, AccessibilityTree, Activating, Adapter, Adapters, Contents, Facts, Limit,
    NodeAt, NotLoaded, READ_WINDOW, ReadingWindows, Role, Running, States, TreeFault, WorkingFully,
    fallback_verbs, load, shipped_adapters,
};
use alo_applications::{Application, Installed};
use alo_capability::{
    Approvals, Authorised, Call, Effect, Given, Grant, GrantError, Grantee, Grants, Proposal,
    ProposalError, Reach, Refused,
};
use alo_record::{Entry, Record};
use alo_strings::{Strings, Vocabulary};

// ---------------------------------------------------------------------------
// The machine these tests look at.
// ---------------------------------------------------------------------------

/// The application with no adapter.
const MAIL: &str = "org.example.Mail";

/// Another installed application, never granted here.
const PAYROLL: &str = "org.example.Payroll";

/// What is typed into the mail application's password field.
const SECRET: &str = "correct-horse-battery-staple";

/// A fixed moment, so expiry is arithmetic rather than a wait.
fn noon() -> SystemTime {
    SystemTime::UNIX_EPOCH + Duration::from_secs(1_760_000_000)
}

/// How long grants and questions last here.
fn hour() -> Duration {
    Duration::from_secs(60 * 60)
}

/// The agent.
fn agent() -> Grantee {
    Grantee::named("@alo")
}

/// The words a shell has.
fn in_english() -> Strings {
    let mut vocabulary = Vocabulary::empty();
    alo_capability::declare_into(&mut vocabulary).unwrap();
    alo_applications::words::declare_into(&mut vocabulary).unwrap();
    alo_adapters::words::declare_into(&mut vocabulary).unwrap();
    Strings::of(vocabulary)
}

/// A grant to the agent over each of these applications, at noon for an hour.
fn granting(applications: &[&str]) -> Grants {
    let mut grants = Grants::default();
    for application in applications {
        grants.grant(
            Grant::checked(
                "@alo",
                Reach::Application((*application).to_owned()),
                noon(),
                hour(),
            )
            .unwrap(),
        );
    }
    grants
}

/// A machine with the mail application, the payroll application and the text
/// editor installed.
fn installed() -> Installed {
    Installed::holding([
        Application::called(MAIL, "Mail").unwrap(),
        Application::called(PAYROLL, "Payroll").unwrap(),
        Application::called("org.gnome.TextEditor", "Text Editor").unwrap(),
    ])
}

/// A call of one of the fallback's verbs.
fn calling(verb: &str, given: &[(&str, Given)]) -> Call {
    fallback_verbs().unwrap().call(verb, given).unwrap()
}

/// Read this application's windows.
fn reading(application: &str) -> Call {
    calling(READ_WINDOW, &[("application", Given::text(application))])
}

/// Press this kind of control, of this name, in this application.
fn pressing(application: &str, kind: &str, name: &str) -> Call {
    calling(
        ACTIVATE_CONTROL,
        &[
            ("application", Given::text(application)),
            ("kind", Given::text(kind)),
            ("name", Given::text(name)),
        ],
    )
}

/// Propose, approve once and redeem.
fn approved(call: &Call, grants: &Grants) -> Authorised {
    let mut approvals = Approvals::default();
    let id = approvals.propose(Proposal::checked(call, &agent(), grants, noon(), hour()).unwrap());
    approvals
        .approve(id, noon())
        .unwrap()
        .redeem(grants, noon())
        .unwrap()
}

// ---------------------------------------------------------------------------
// A tree that keeps every question it is asked.
// ---------------------------------------------------------------------------

/// One question the tree was asked.
#[derive(Debug, Clone, PartialEq, Eq)]
enum Asked {
    Applications,
    Facts(NodeAt),
    Text(NodeAt),
    Actions(NodeAt),
    Act(NodeAt, usize),
}

/// One thing in a window, as a test writes it.
struct Thing {
    role: u32,
    name: &'static str,
    states: States,
    text: Option<&'static str>,
    /// The number it holds, for a slider or a meter.
    value: Option<f64>,
    actions: &'static [&'static str],
    children: Vec<Thing>,
    /// Held on another connection: another program's.
    elsewhere: bool,
}

/// A thing with a role and a name, showing and usable.
fn thing(role: u32, name: &'static str) -> Thing {
    Thing {
        role,
        name,
        states: States::showing_and_usable(),
        text: None,
        value: None,
        actions: &[],
        children: Vec::new(),
        elsewhere: false,
    }
}

impl Thing {
    fn saying(mut self, text: &'static str) -> Self {
        self.text = Some(text);
        self
    }
    fn holding_the_value(mut self, value: f64) -> Self {
        self.value = Some(value);
        self
    }
    fn doing(mut self, actions: &'static [&'static str]) -> Self {
        self.actions = actions;
        self
    }
    fn holding(mut self, children: Vec<Thing>) -> Self {
        self.children = children;
        self
    }
    fn in_states(mut self, states: States) -> Self {
        self.states = states;
        self
    }
    fn elsewhere(mut self) -> Self {
        self.elsewhere = true;
        self
    }
}

/// `AtspiRole` numbers the tests write.
mod roles {
    pub const CHECK_BOX: u32 = 7;
    pub const COMBO_BOX: u32 = 11;
    pub const DRAWING_AREA: u32 = 18;
    pub const FILLER: u32 = 20;
    pub const FRAME: u32 = 23;
    pub const LABEL: u32 = 29;
    pub const LIST: u32 = 31;
    pub const LIST_ITEM: u32 = 32;
    pub const MENU: u32 = 33;
    pub const MENU_BAR: u32 = 34;
    pub const MENU_ITEM: u32 = 35;
    pub const PAGE_TAB: u32 = 37;
    pub const PROGRESS_BAR: u32 = 42;
    pub const SLIDER: u32 = 51;
    pub const TABLE_CELL: u32 = 56;
    pub const PASSWORD_TEXT: u32 = 40;
    pub const PUSH_BUTTON: u32 = 43;
    pub const TEXT: u32 = 61;
    pub const APPLICATION: u32 = 75;
}

/// A stand-in accessibility tree.
#[derive(Default)]
struct ATree {
    running: Vec<Running>,
    facts: BTreeMap<NodeAt, Facts>,
    texts: BTreeMap<NodeAt, String>,
    values: BTreeMap<NodeAt, f64>,
    actions: BTreeMap<NodeAt, Vec<String>>,
    names: BTreeMap<String, NodeAt>,
    /// What pressing answers.
    presses: Option<Result<bool, TreeFault>>,
    /// What every question answers instead, when set.
    broken: Option<TreeFault>,
    asked: RefCell<Vec<Asked>>,
    next: usize,
}

impl ATree {
    /// Add an application held on `holder`, named by its sandbox as `is`, with
    /// these windows.
    fn with(mut self, holder: &str, is: Option<&str>, windows: Vec<Thing>) -> Self {
        let root = NodeAt {
            holder: holder.to_owned(),
            object: "/root".to_owned(),
        };
        self.running.push(Running {
            at: root.clone(),
            is: is.map(ToOwned::to_owned),
        });
        let root_thing = thing(roles::APPLICATION, "").holding(windows);
        self.add(holder, root, root_thing);
        self
    }

    fn add(&mut self, holder: &str, at: NodeAt, thing: Thing) {
        let mut children = Vec::new();
        for child in thing.children {
            self.next += 1;
            let child_holder = if child.elsewhere { ":9.9" } else { holder };
            let child_at = NodeAt {
                holder: child_holder.to_owned(),
                object: format!("/thing/{}", self.next),
            };
            children.push(child_at.clone());
            self.add(child_holder, child_at, child);
        }
        if let Some(text) = thing.text {
            self.texts.insert(at.clone(), text.to_owned());
        }
        if let Some(value) = thing.value {
            self.values.insert(at.clone(), value);
        }
        if !thing.actions.is_empty() {
            self.actions.insert(
                at.clone(),
                thing.actions.iter().map(|a| (*a).to_owned()).collect(),
            );
        }
        if !thing.name.is_empty() {
            self.names
                .insert(format!("{holder}{}", thing.name), at.clone());
        }
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

    fn pressing_answers(mut self, answer: Result<bool, TreeFault>) -> Self {
        self.presses = Some(answer);
        self
    }

    fn broken(mut self, fault: TreeFault) -> Self {
        self.broken = Some(fault);
        self
    }

    /// Where the thing of this name, held on `holder`, is.
    fn named(&self, holder: &str, name: &str) -> NodeAt {
        self.names.get(&format!("{holder}{name}")).unwrap().clone()
    }

    fn asked(&self) -> Vec<Asked> {
        self.asked.borrow().clone()
    }

    fn presses_asked(&self) -> usize {
        self.asked()
            .iter()
            .filter(|asked| matches!(asked, Asked::Act(..)))
            .count()
    }

    fn hear(&self, asked: Asked) -> Result<(), TreeFault> {
        self.asked.borrow_mut().push(asked);
        self.broken.map_or(Ok(()), Err)
    }
}

impl AccessibilityTree for ATree {
    fn applications(&self) -> Result<Vec<Running>, TreeFault> {
        self.hear(Asked::Applications)?;
        Ok(self.running.clone())
    }

    fn facts(&self, at: &NodeAt) -> Result<Facts, TreeFault> {
        self.hear(Asked::Facts(at.clone()))?;
        self.facts.get(at).cloned().ok_or(TreeFault::Vanished)
    }

    fn text(&self, at: &NodeAt) -> Result<String, TreeFault> {
        self.hear(Asked::Text(at.clone()))?;
        self.texts.get(at).cloned().ok_or(TreeFault::Vanished)
    }

    fn value(&self, at: &NodeAt) -> Result<Option<f64>, TreeFault> {
        self.hear(Asked::Facts(at.clone()))?;
        Ok(self.values.get(at).copied())
    }

    /// Every thing holding a number goes from 0 to 400 here.
    fn range(&self, at: &NodeAt) -> Result<Option<alo_adapters::Bounds>, TreeFault> {
        self.hear(Asked::Facts(at.clone()))?;
        Ok(self.values.get(at).map(|_| alo_adapters::Bounds {
            least: 0.0,
            most: 400.0,
            step: 1.0,
        }))
    }

    fn actions(&self, at: &NodeAt) -> Result<Vec<String>, TreeFault> {
        self.hear(Asked::Actions(at.clone()))?;
        Ok(self.actions.get(at).cloned().unwrap_or_default())
    }

    fn act(&self, at: &NodeAt, action: usize) -> Result<bool, TreeFault> {
        self.hear(Asked::Act(at.clone(), action))?;
        self.presses.unwrap_or(Ok(true))
    }
}

/// The mail application's sign-in window, and around it an impostor naming
/// itself after it and the payroll application.
fn the_desktop() -> ATree {
    ATree::default()
        .with(
            ":1.7",
            Some(MAIL),
            vec![
                thing(roles::FRAME, "Sign in to Mail").holding(vec![
                    thing(roles::FILLER, "").holding(vec![
                        thing(roles::LABEL, "Account").saying("Account"),
                        thing(roles::TEXT, "").saying("anna"),
                        thing(roles::PASSWORD_TEXT, "Password").saying(SECRET),
                        thing(roles::CHECK_BOX, "Remember me").doing(&["click"]),
                        thing(roles::PUSH_BUTTON, "Sign in").doing(&["Click"]),
                        thing(roles::PUSH_BUTTON, "").doing(&["click"]),
                        thing(roles::DRAWING_AREA, ""),
                        thing(roles::PUSH_BUTTON, "Delete account")
                            .doing(&["click"])
                            .in_states(States::showing_and_usable().greyed_out()),
                        thing(roles::PUSH_BUTTON, "Help").doing(&["click"]),
                        thing(roles::PUSH_BUTTON, "Help").doing(&["click"]),
                        thing(roles::PUSH_BUTTON, "Details"),
                        thing(roles::LABEL, "Not on the screen")
                            .saying("Not on the screen")
                            .in_states(States::showing_and_usable().hidden()),
                        thing(roles::LABEL, "An advertisement").elsewhere(),
                    ]),
                ]),
                // A window that is not on the screen.
                thing(roles::FRAME, "Drafts").in_states(States::showing_and_usable().hidden()),
            ],
        )
        .with(
            ":1.9",
            None,
            vec![
                thing(roles::FRAME, MAIL)
                    .holding(vec![thing(roles::LABEL, "Salaries").saying("Anna 4,200")]),
            ],
        )
        .with(
            ":1.11",
            Some(PAYROLL),
            vec![
                thing(roles::FRAME, "Payroll")
                    .holding(vec![thing(roles::LABEL, "Salaries").saying("Anna 4,200")]),
            ],
        )
}

// ---------------------------------------------------------------------------
// The verbs.
// ---------------------------------------------------------------------------

/// **Two typed verbs**: a read over a granted application, and a change that
/// names a kind of control, the name it shows and the application — in the
/// sentence a person approves. Nothing else can be sent: no position, no text
/// to type, no kind of control that is not on the list.
#[test]
fn the_two_verbs_are_a_read_and_a_change_and_take_nothing_else() {
    let strings = in_english();
    let verbs = fallback_verbs().unwrap();
    assert_eq!(verbs.of(READ_WINDOW).unwrap().effect(), Effect::Read);
    assert_eq!(verbs.of(ACTIVATE_CONTROL).unwrap().effect(), Effect::Change);

    let grants = granting(&[MAIL]);
    let proposal = Proposal::checked(
        &pressing(MAIL, "button", "Sign in"),
        &agent(),
        &grants,
        noon(),
        hour(),
    )
    .unwrap();
    assert_eq!(
        proposal.sentence(&strings).text(),
        "press the button named “Sign in” in org.example.Mail"
    );
    assert_eq!(
        Proposal::checked(
            &pressing(MAIL, "check_box", "Remember me"),
            &agent(),
            &grants,
            noon(),
            hour()
        )
        .unwrap()
        .sentence(&strings)
        .text(),
        "press the check box named “Remember me” in org.example.Mail"
    );

    for refused in [
        // A kind of control that is not on the list.
        vec![
            ("application", Given::text(MAIL)),
            ("kind", Given::text("text_field")),
            ("name", Given::text("Password")),
        ],
        // A name that is a journey, not a name.
        vec![
            ("application", Given::text(MAIL)),
            ("kind", Given::text("button")),
            ("name", Given::text("../Sign in")),
        ],
        // No name at all.
        vec![
            ("application", Given::text(MAIL)),
            ("kind", Given::text("button")),
            ("name", Given::text("  ")),
        ],
        // A position.
        vec![
            ("application", Given::text(MAIL)),
            ("kind", Given::text("button")),
            ("name", Given::text("Sign in")),
            ("x", Given::number(120)),
        ],
        // Text to type.
        vec![
            ("application", Given::text(MAIL)),
            ("kind", Given::text("button")),
            ("name", Given::text("Sign in")),
            ("text", Given::text(SECRET)),
        ],
    ] {
        assert!(
            verbs.call(ACTIVATE_CONTROL, &refused).is_err(),
            "{refused:?} was accepted"
        );
    }
    // A read waits for nothing, and so cannot be put to a person.
    assert!(matches!(
        Proposal::checked(&reading(MAIL), &agent(), &grants, noon(), hour()).unwrap_err(),
        ProposalError::ReadDoesNotWait { .. }
    ));
}

/// **An adapter cannot be called what the fallback's verbs are named under.**
#[test]
fn an_adapter_cannot_take_the_fallbacks_name() {
    let named_accessible = Box::leak(Box::new(Adapter {
        name: "accessible",
        ..alo_adapters::TEXT_EDITOR
    }));
    assert_eq!(
        load(named_accessible).unwrap_err(),
        NotLoaded::TheFallbacksName
    );
}

// ---------------------------------------------------------------------------
// Reading.
// ---------------------------------------------------------------------------

/// **What a granted application's windows show is read inside the turn, within
/// its grant, and recorded**: the words, the field's contents, the controls and
/// whether each is on or usable — and nothing that is not on the screen, and
/// nothing of any other program's, including one that names itself after it.
#[test]
fn a_window_is_read_within_its_grant_and_recorded() {
    let strings = in_english();
    let tree = the_desktop();
    let grants = granting(&[MAIL]);
    let authorised = Authorised::read(&reading(MAIL), &agent(), &grants, noon()).unwrap();
    let read = ReadingWindows::of(
        authorised,
        &shipped_adapters().unwrap(),
        &grants,
        &installed(),
        &strings,
    )
    .unwrap()
    .read(&tree, &strings)
    .unwrap();

    let shown = read.shown();
    assert_eq!(shown.application(), MAIL);
    let titles: Vec<&str> = shown.windows().iter().map(|w| w.title()).collect();
    assert_eq!(
        titles,
        ["Sign in to Mail"],
        "a window off the screen was read"
    );
    let seen: Vec<(Role, &str)> = shown.everything().map(|s| (s.role(), s.name())).collect();
    assert_eq!(
        seen,
        [
            (Role::Label, "Account"),
            (Role::TextField, ""),
            (Role::PasswordField, "Password"),
            (Role::CheckBox, "Remember me"),
            (Role::Button, "Sign in"),
            (Role::Button, ""),
            (Role::Canvas, ""),
            (Role::Button, "Delete account"),
            (Role::Button, "Help"),
            (Role::Button, "Help"),
            (Role::Button, "Details"),
            (Role::Part, ""),
        ]
    );
    let field = shown.everything().nth(1).unwrap();
    assert_eq!(field.contents(), &Contents::Text("anna".to_owned()));
    let remember = shown.everything().nth(3).unwrap();
    assert_eq!(remember.is_on(), Some(false));
    let delete = shown.everything().nth(7).unwrap();
    assert!(!delete.is_usable());

    // Nothing of the impostor's, nor the payroll application's, was asked.
    for asked in tree.asked() {
        if let Asked::Facts(at) | Asked::Text(at) = &asked {
            assert_eq!(at.holder, ":1.7", "{asked:?} is another program's");
        }
    }
    assert!(!format!("{shown:?}").contains("4,200"));
    assert!(!format!("{shown:?}").contains("Not on the screen"));

    let (authorised, _) = read.into_parts();
    let mut record = Record::default();
    record.keep(Entry::ran(&authorised, &strings));
    let entry = record.everything().next().unwrap();
    assert!(entry.happened().ran());
    assert_eq!(entry.happened().from_approval(), None);
    assert_eq!(entry.happened().against().len(), 1);
    let what = entry.happened().what().unwrap();
    assert!(what.verb().is(READ_WINDOW));
    assert!(
        what.sentence()
            .is("read what org.example.Mail shows in its windows")
    );
}

/// **A password field's contents are never asked for** — the tree here would
/// answer with them, and is never given the chance — and never appear in what
/// is handed back.
#[test]
fn a_password_fields_contents_are_never_asked_for() {
    let strings = in_english();
    let tree = the_desktop();
    let grants = granting(&[MAIL]);
    let authorised = Authorised::read(&reading(MAIL), &agent(), &grants, noon()).unwrap();
    let read = ReadingWindows::of(
        authorised,
        &Adapters::default(),
        &grants,
        &installed(),
        &strings,
    )
    .unwrap()
    .read(&tree, &strings)
    .unwrap();

    let password = tree.named(":1.7", "Password");
    assert!(
        !tree.asked().contains(&Asked::Text(password.clone())),
        "the password field was asked for its text"
    );
    assert!(tree.asked().contains(&Asked::Facts(password)));
    let field = read
        .shown()
        .everything()
        .find(|seen| seen.role() == Role::PasswordField)
        .unwrap();
    assert_eq!(field.contents(), &Contents::Withheld);
    assert!(!format!("{:?}", read.shown()).contains(SECRET));

    // And pressing something in the same window asks it nothing either.
    let tree = the_desktop();
    let call = pressing(MAIL, "button", "Sign in");
    Activating::of(
        approved(&call, &grants),
        &Adapters::default(),
        &grants,
        &installed(),
        &strings,
    )
    .unwrap()
    .press(&tree, WorkingFully::NothingMoreToTurnOn, &strings)
    .unwrap();
    let password = tree.named(":1.7", "Password");
    assert!(!tree.asked().contains(&Asked::Text(password)));
}

/// **What the fallback cannot read is said in words** — a control with no
/// name, an area the application draws itself, a part another program shows,
/// and a window larger than is read at once — and none of them is something
/// the agent can ask to press.
#[test]
fn what_cannot_be_read_is_said_in_words() {
    let strings = in_english();
    let tree = the_desktop();
    let grants = granting(&[MAIL]);
    let read = ReadingWindows::of(
        Authorised::read(&reading(MAIL), &agent(), &grants, noon()).unwrap(),
        &Adapters::default(),
        &grants,
        &installed(),
        &strings,
    )
    .unwrap()
    .read(&tree, &strings)
    .unwrap();
    let limits: Vec<(Role, Limit)> = read
        .shown()
        .everything()
        .filter_map(|seen| seen.limit().map(|limit| (seen.role(), limit)))
        .collect();
    assert_eq!(
        limits,
        [
            (Role::Button, Limit::NoName),
            (Role::Canvas, Limit::NotDescribed),
            (Role::Part, Limit::ShownByAnotherProgram),
        ]
    );
    assert_eq!(
        Limit::NoName.said(&strings).text(),
        "a control with no name, which the agent can neither describe nor press"
    );
    assert_eq!(
        Limit::NotDescribed.said(&strings).text(),
        "an area the application draws itself and does not describe, which the agent cannot read"
    );
    assert_eq!(
        Limit::ShownByAnotherProgram.said(&strings).text(),
        "a part of the window shown by another program, which was not read"
    );
    assert!(!read.shown().not_all_read());
    assert!(read.shown().not_all_read_said(&strings).is_none());

    // A drawing area is not a kind of control, and a nameless control has no
    // name to be asked for by.
    let verbs = fallback_verbs().unwrap();
    for kind in ["canvas", "drawing_area", "image"] {
        assert!(
            verbs
                .call(
                    ACTIVATE_CONTROL,
                    &[
                        ("application", Given::text(MAIL)),
                        ("kind", Given::text(kind)),
                        ("name", Given::text("Sign in")),
                    ],
                )
                .is_err()
        );
    }

    // A window with more in it than is read at once says so.
    let mut big = the_desktop();
    let window = big.named(":1.7", "Sign in to Mail");
    big.facts.get_mut(&window).unwrap().unlisted_children = true;
    let read = ReadingWindows::of(
        Authorised::read(&reading(MAIL), &agent(), &grants, noon()).unwrap(),
        &Adapters::default(),
        &grants,
        &installed(),
        &strings,
    )
    .unwrap()
    .read(&big, &strings)
    .unwrap();
    assert!(read.shown().not_all_read());
    assert_eq!(
        read.shown().not_all_read_said(&strings).unwrap().text(),
        "org.example.Mail shows more than can be read at once, so not all of it was read"
    );
}

/// **A text longer than is read at once says it was cut short** — on the thing
/// whose text it is, so an agent told what a document holds is never handed
/// its beginning as though it were the whole. A text that fits is not marked,
/// and the mark does not stop a press: what is cut is one text, not the window.
#[test]
fn a_text_longer_than_is_read_at_once_says_it_was_cut_short() {
    let strings = in_english();
    let longest = alo_adapters::walking::LONGEST_TEXT;
    let long: &'static str = Box::leak("a".repeat(longest + 1).into_boxed_str());
    let whole: &'static str = Box::leak("b".repeat(longest).into_boxed_str());
    let tree = ATree::default().with(
        ":1.7",
        Some(MAIL),
        vec![thing(roles::FRAME, "A long letter").holding(vec![
            thing(roles::TEXT, "Letter").saying(long),
            thing(roles::TEXT, "Reply").saying(whole),
            thing(roles::PUSH_BUTTON, "Send").doing(&["click"]),
        ])],
    );
    let grants = granting(&[MAIL]);
    let read = ReadingWindows::of(
        Authorised::read(&reading(MAIL), &agent(), &grants, noon()).unwrap(),
        &Adapters::default(),
        &grants,
        &installed(),
        &strings,
    )
    .unwrap()
    .read(&tree, &strings)
    .unwrap();
    let cut: Vec<(&str, bool, usize)> = read
        .shown()
        .everything()
        .filter(|seen| seen.role() == Role::TextField)
        .map(|seen| {
            (
                seen.name(),
                seen.is_cut_short(),
                seen.text_to_read().map_or(0, |text| text.chars().count()),
            )
        })
        .collect();
    assert_eq!(cut, [("Letter", true, longest), ("Reply", false, longest)]);
    assert!(
        !read.shown().not_all_read(),
        "one cut text is not the window"
    );
    let said: Vec<String> = read
        .shown()
        .cut_short_said(&strings)
        .into_iter()
        .map(alo_strings::Said::into_text)
        .collect();
    assert_eq!(
        said,
        [
            "org.example.Mail shows a text longer than is read at once, so only its first 4000 \
             characters were read"
        ]
    );

    let pressed = Activating::of(
        approved(&pressing(MAIL, "button", "Send"), &grants),
        &Adapters::default(),
        &grants,
        &installed(),
        &strings,
    )
    .unwrap()
    .press(&tree, WorkingFully::NothingMoreToTurnOn, &strings);
    assert!(pressed.is_ok(), "a cut text stopped a press");
}

/// One thing read: its role, name, text, whether selected, and its number.
type OneThingRead<'a> = (
    Role,
    &'a str,
    Option<&'a str>,
    Option<bool>,
    Option<&'a str>,
);

/// **Drop-downs, list items and sliders are read for what they are**, with
/// what a person sees of each: the choice a drop-down shows, which items and
/// tabs are selected, and the number a slider or a meter holds. Until
/// 2026-11-17 all three were walked past as layout — a font box, a folder's
/// files, a zoom slider — so alo could not say what was chosen, selected or
/// set. A thing that cannot be selected says nothing about selection, and a
/// thing with no number says no number.
#[test]
fn drop_downs_list_items_and_sliders_are_read_for_what_they_are() {
    let strings = in_english();
    let selectable = States::showing_and_usable().selectable();
    let tree = ATree::default().with(
        ":1.7",
        Some(MAIL),
        vec![thing(roles::FRAME, "Letter").holding(vec![
            thing(roles::PAGE_TAB, "General")
                .doing(&["click"])
                .in_states(selectable.and_selected()),
            thing(roles::PAGE_TAB, "Advanced")
                .doing(&["click"])
                .in_states(selectable),
            thing(roles::COMBO_BOX, "Font").saying("Arial"),
            thing(roles::LIST, "Attachments").holding(vec![
                thing(roles::LIST_ITEM, "Report.pdf").in_states(selectable.and_selected()),
                thing(roles::LIST_ITEM, "Notes.txt").in_states(selectable),
                thing(roles::TABLE_CELL, "12 KB"),
            ]),
            thing(roles::SLIDER, "Zoom").holding_the_value(150.0),
            thing(roles::PROGRESS_BAR, "Sending").holding_the_value(40.5),
        ])],
    );
    let grants = granting(&[MAIL]);
    let read = ReadingWindows::of(
        Authorised::read(&reading(MAIL), &agent(), &grants, noon()).unwrap(),
        &Adapters::default(),
        &grants,
        &installed(),
        &strings,
    )
    .unwrap()
    .read(&tree, &strings)
    .unwrap();
    let seen: Vec<OneThingRead<'_>> = read
        .shown()
        .everything()
        .map(|seen| {
            (
                seen.role(),
                seen.name(),
                seen.text_to_read(),
                seen.is_selected(),
                seen.value(),
            )
        })
        .collect();
    assert_eq!(
        seen,
        [
            (Role::Tab, "General", None, Some(true), None),
            (Role::Tab, "Advanced", None, Some(false), None),
            (Role::DropDown, "Font", Some("Arial"), None, None),
            (Role::Part, "Attachments", None, None, None),
            (
                Role::Item,
                "Report.pdf",
                Some("Report.pdf"),
                Some(true),
                None
            ),
            (
                Role::Item,
                "Notes.txt",
                Some("Notes.txt"),
                Some(false),
                None
            ),
            (Role::Item, "12 KB", Some("12 KB"), None, None),
            (Role::Range, "Zoom", None, None, Some("150")),
            (Role::Range, "Sending", None, None, Some("40.5")),
        ]
    );
    // **And whether each can be moved is said**, so a model told a slider
    // holds 150 is not left to guess whether it may set it: a slider can, a
    // progress bar cannot (2026-11-20 — told only `slider` and a number,
    // `gpt-4o-mini` answered "it cannot be set to 200" in 11 of 20 tries).
    let moved: Vec<(&str, Option<bool>)> = read
        .shown()
        .everything()
        .filter(|seen| seen.role() == Role::Range)
        .map(|seen| (seen.name(), seen.can_be_moved()))
        .collect();
    assert_eq!(moved, [("Zoom", Some(true)), ("Sending", Some(false))]);
    // **And how far a slider goes**, so a model is not left to guess that a
    // slider at 100 can go no further: told a number and nothing else, it
    // answered "cannot be set to 200" in 2 of 10 tries at full access
    // (run 44803684908). A progress bar is never asked its range.
    let ranges: Vec<(&str, Option<(&str, &str)>)> = read
        .shown()
        .everything()
        .filter(|seen| seen.role() == Role::Range)
        .map(|seen| (seen.name(), seen.goes_from_to()))
        .collect();
    assert_eq!(ranges, [("Zoom", Some(("0", "400"))), ("Sending", None)]);
}

/// One thing read: its role, name, limit, text and number.
type OneThingLimited<'a> = (
    Role,
    &'a str,
    Option<Limit>,
    Option<&'a str>,
    Option<&'a str>,
);

/// **What a drop-down or a slider with no name says decides whether it is
/// shown**: a drop-down with neither a name nor a choice is a control alo can
/// say nothing useful about, said as one with no name; one showing a choice is
/// shown by it; a slider with no name is shown for its number, and one with
/// neither is passed over as layout.
#[test]
fn a_nameless_drop_down_or_slider_is_shown_by_what_it_holds() {
    let strings = in_english();
    let tree = ATree::default().with(
        ":1.7",
        Some(MAIL),
        vec![thing(roles::FRAME, "Letter").holding(vec![
            thing(roles::COMBO_BOX, ""),
            thing(roles::COMBO_BOX, "").saying("Arial"),
            thing(roles::SLIDER, "").holding_the_value(3.0),
            thing(roles::SLIDER, ""),
            thing(roles::SLIDER, "Volume"),
        ])],
    );
    let grants = granting(&[MAIL]);
    let read = ReadingWindows::of(
        Authorised::read(&reading(MAIL), &agent(), &grants, noon()).unwrap(),
        &Adapters::default(),
        &grants,
        &installed(),
        &strings,
    )
    .unwrap()
    .read(&tree, &strings)
    .unwrap();
    let seen: Vec<OneThingLimited<'_>> = read
        .shown()
        .everything()
        .map(|seen| {
            (
                seen.role(),
                seen.name(),
                seen.limit(),
                seen.text_to_read(),
                seen.value(),
            )
        })
        .collect();
    assert_eq!(
        seen,
        [
            (Role::DropDown, "", Some(Limit::NoName), None, None),
            (Role::DropDown, "", None, Some("Arial"), None),
            (Role::Range, "", None, None, Some("3")),
            (Role::Range, "Volume", None, None, None),
        ]
    );
}

// ---------------------------------------------------------------------------
// Pressing.
// ---------------------------------------------------------------------------

/// **What the approval card says a press changes and leaves alone is what a
/// press does** (the owner's decision of 2026-11-01): *presses that one control
/// once, by its own action* — one act, on the named control, by an action the
/// control offers — and *nothing is typed, no other control is pressed, and
/// nothing is found by where it is on the screen* — no text was asked of any
/// field, no other control was acted on, and the tree has no question about
/// position to ask.
#[test]
fn a_press_does_what_its_card_says_and_nothing_it_says_it_leaves() {
    let strings = in_english();
    let verbs = fallback_verbs().unwrap();
    let press = verbs.of(ACTIVATE_CONTROL).unwrap();
    assert_eq!(
        press.what_it_changes().map(|word| word.says()),
        Some("Presses that one control once, by its own action, as a person pressing it would.")
    );
    assert_eq!(
        press.what_it_leaves().map(|word| word.says()),
        Some(
            "Nothing is typed, no other control is pressed, and nothing is found by where it is \
             on the screen."
        )
    );

    let tree = the_desktop();
    let grants = granting(&[MAIL]);
    let authorised = approved(&pressing(MAIL, "button", "Sign in"), &grants);
    Activating::of(
        authorised,
        &shipped_adapters().unwrap(),
        &grants,
        &installed(),
        &strings,
    )
    .unwrap()
    .press(&tree, WorkingFully::NothingMoreToTurnOn, &strings)
    .unwrap();

    let sign_in = tree.named(":1.7", "Sign in");
    let acts: Vec<Asked> = tree
        .asked()
        .into_iter()
        .filter(|asked| matches!(asked, Asked::Act(..)))
        .collect();
    // One act, on the named control and no other. **Nothing typed and nothing
    // by position** need no assertion of their own: `AccessibilityTree` has no
    // question that types or asks where a thing is, so no press can.
    assert_eq!(acts, [Asked::Act(sign_in, 0)], "one control, once");
}

/// **One approval presses the control it names, once, and is recorded** with
/// its approval, its grant and the sentence the person approved.
#[test]
fn one_approval_presses_the_control_it_names_once_and_is_recorded() {
    let strings = in_english();
    let tree = the_desktop();
    let grants = granting(&[MAIL]);
    let call = pressing(MAIL, "button", "Sign in");

    let mut approvals = Approvals::default();
    let id =
        approvals.propose(Proposal::checked(&call, &agent(), &grants, noon(), hour()).unwrap());
    let authorised = approvals
        .approve(id, noon())
        .unwrap()
        .redeem(&grants, noon())
        .unwrap();
    assert!(approvals.approve(id, noon()).is_err(), "answered twice");

    let pressed = Activating::of(
        authorised,
        &shipped_adapters().unwrap(),
        &grants,
        &installed(),
        &strings,
    )
    .unwrap()
    .press(&tree, WorkingFully::NothingMoreToTurnOn, &strings)
    .unwrap();
    assert!(!pressed.unanswered());
    assert!(pressed.said(&strings).is_none());

    let sign_in = tree.named(":1.7", "Sign in");
    let presses: Vec<Asked> = tree
        .asked()
        .into_iter()
        .filter(|asked| matches!(asked, Asked::Act(..)))
        .collect();
    // The action named "Click" — matched without regard to case — at its place.
    assert_eq!(presses, [Asked::Act(sign_in, 0)], "one approval, one press");

    let mut record = Record::default();
    record.keep(Entry::ran(&pressed.into_authorised(), &strings));
    let entry = record.everything().next().unwrap();
    assert!(entry.happened().ran());
    assert_eq!(entry.happened().from_approval(), Some(id.as_u64()));
    assert_eq!(entry.happened().against().len(), 1);
    let what = entry.happened().what().unwrap();
    assert!(what.verb().is(ACTIVATE_CONTROL));
    assert!(
        what.sentence()
            .is("press the button named “Sign in” in org.example.Mail")
    );
}

/// **A menu's heading can be opened.** In a menu bar, *File* and *Edit* are
/// not menu items to the tree but menus (`ROLE_MENU`) — GTK 3's, LibreOffice's,
/// Firefox's — and so were walked past as layout, which left every menu bar on
/// the machine shut to alo. A named menu is now a menu item a press opens, by
/// the same kind and the same sentence; a menu with no name — the list a
/// heading opens, in GTK 3 — is still layout, and is not said as a control
/// alo cannot name.
#[test]
fn a_menus_heading_is_a_menu_item_a_press_opens() {
    let strings = in_english();
    let grants = granting(&[MAIL]);
    let tree = ATree::default().with(
        ":1.7",
        Some(MAIL),
        vec![
            thing(roles::FRAME, "Letter").holding(vec![thing(roles::MENU_BAR, "").holding(vec![
                thing(roles::MENU, "File").doing(&["click"]).holding(vec![
                    thing(roles::MENU, "").holding(vec![
                        thing(roles::MENU_ITEM, "Save").doing(&["click"]),
                    ]),
                ]),
            ])]),
        ],
    );
    let read = ReadingWindows::of(
        Authorised::read(&reading(MAIL), &agent(), &grants, noon()).unwrap(),
        &Adapters::default(),
        &grants,
        &installed(),
        &strings,
    )
    .unwrap()
    .read(&tree, &strings)
    .unwrap();
    let seen: Vec<(Role, &str, Option<Limit>)> = read
        .shown()
        .everything()
        .map(|seen| (seen.role(), seen.name(), seen.limit()))
        .collect();
    assert_eq!(
        seen,
        [
            (Role::MenuItem, "File", None),
            (Role::MenuItem, "Save", None),
        ]
    );

    let pressed = Activating::of(
        approved(&pressing(MAIL, "menu_item", "File"), &grants),
        &Adapters::default(),
        &grants,
        &installed(),
        &strings,
    )
    .unwrap()
    .press(&tree, WorkingFully::NothingMoreToTurnOn, &strings);
    assert!(pressed.is_ok(), "the menu's heading was not pressed");
    assert_eq!(
        tree.asked().last(),
        Some(&Asked::Act(tree.named(":1.7", "File"), 0))
    );
}

/// **The control is looked for again when it is pressed**, in the window as it
/// is then — a control renamed since it was read is not pressed.
#[test]
fn the_control_is_found_again_at_the_moment_it_is_pressed() {
    let strings = in_english();
    let grants = granting(&[MAIL]);
    let before = the_desktop();
    ReadingWindows::of(
        Authorised::read(&reading(MAIL), &agent(), &grants, noon()).unwrap(),
        &Adapters::default(),
        &grants,
        &installed(),
        &strings,
    )
    .unwrap()
    .read(&before, &strings)
    .unwrap();

    let mut after = the_desktop();
    let sign_in = after.named(":1.7", "Sign in");
    "Sign out".clone_into(&mut after.facts.get_mut(&sign_in).unwrap().name);
    let call = pressing(MAIL, "button", "Sign in");
    let refused = Activating::of(
        approved(&call, &grants),
        &Adapters::default(),
        &grants,
        &installed(),
        &strings,
    )
    .unwrap()
    .press(&after, WorkingFully::NothingMoreToTurnOn, &strings)
    .unwrap_err();
    assert_eq!(
        refused.said(&strings).text(),
        "org.example.Mail shows no button named “Sign in” now, so nothing was pressed"
    );
    assert_eq!(after.presses_asked(), 0);
    assert!(
        after.asked().contains(&Asked::Applications),
        "not looked for again"
    );
}

/// **Nothing is guessed**: two controls of the same kind and name are refused
/// rather than one being chosen, and a window too large to read whole is
/// refused because *the only one* cannot then be known.
#[test]
fn nothing_is_guessed_when_the_control_is_not_the_only_one() {
    let strings = in_english();
    let grants = granting(&[MAIL]);
    let press = |tree: &ATree, name: &str| {
        Activating::of(
            approved(&pressing(MAIL, "button", name), &grants),
            &Adapters::default(),
            &grants,
            &installed(),
            &strings,
        )
        .unwrap()
        .press(tree, WorkingFully::NothingMoreToTurnOn, &strings)
        .unwrap_err()
    };
    let tree = the_desktop();
    assert_eq!(
        press(&tree, "Help").said(&strings).text(),
        "org.example.Mail shows more than one button named “Help”, so nothing was pressed rather \
         than guessing which"
    );
    let mut big = the_desktop();
    let window = big.named(":1.7", "Sign in to Mail");
    big.facts.get_mut(&window).unwrap().unlisted_children = true;
    assert_eq!(
        press(&big, "Sign in").said(&strings).text(),
        "org.example.Mail shows more than can be read at once, so whether its button named “Sign \
         in” is the only one is not known, and nothing was pressed"
    );
    assert_eq!(tree.presses_asked() + big.presses_asked(), 0);
}

/// **A long list off the screen no longer stops a press.** A list view lists
/// every row it holds, on the screen or not — Qt's does — so an archive of a
/// few hundred files held more things than one walk reads, and then nothing in
/// its window could be pressed: *too much to be sure*. What is counted against
/// [`alo_adapters::walking::MOST_THINGS`] is what is **on the screen**, which is
/// all a person could press; what is off it is still bounded, by
/// [`alo_adapters::walking::MOST_ASKED`], and past that the window still says
/// not all of it was read and a press is still refused.
#[test]
fn a_long_list_off_the_screen_does_not_stop_a_press() {
    let strings = in_english();
    let grants = granting(&[MAIL]);
    let hidden = |count: usize| -> Vec<Thing> {
        (0..count)
            .map(|_| thing(roles::FILLER, "").in_states(States::showing_and_usable().hidden()))
            .collect()
    };
    let window = |rows: Vec<Thing>| {
        ATree::default().with(
            ":1.7",
            Some(MAIL),
            vec![thing(roles::FRAME, "Archive").holding(vec![
                thing(roles::FILLER, "").holding(rows),
                thing(roles::PUSH_BUTTON, "Extract").doing(&["click"]),
            ])],
        )
    };
    let press = |tree: &ATree| {
        Activating::of(
            approved(&pressing(MAIL, "button", "Extract"), &grants),
            &Adapters::default(),
            &grants,
            &installed(),
            &strings,
        )
        .unwrap()
        .press(tree, WorkingFully::NothingMoreToTurnOn, &strings)
    };
    let read = |tree: &ATree| {
        ReadingWindows::of(
            Authorised::read(&reading(MAIL), &agent(), &grants, noon()).unwrap(),
            &Adapters::default(),
            &grants,
            &installed(),
            &strings,
        )
        .unwrap()
        .read(tree, &strings)
        .unwrap()
    };

    let long = window(hidden(alo_adapters::walking::MOST_THINGS + 500));
    assert!(!read(&long).shown().not_all_read());
    assert!(press(&long).is_ok(), "rows off the screen stopped a press");
    assert_eq!(long.presses_asked(), 1);

    let endless = window(hidden(alo_adapters::walking::MOST_ASKED + 1));
    assert!(read(&endless).shown().not_all_read());
    assert!(
        press(&endless).is_err(),
        "a list past every bound was pressed in"
    );
    assert_eq!(endless.presses_asked(), 0);
}

/// **Exactly as many things on the screen as one walk reads are all read**,
/// and one more is said as not all read: the list itself and the labels in it
/// are what count, one each.
#[test]
fn as_many_things_on_the_screen_as_are_read_are_read_and_one_more_is_not() {
    let strings = in_english();
    let grants = granting(&[MAIL]);
    let shown =
        |labels: usize| {
            let tree =
                ATree::default().with(
                    ":1.7",
                    Some(MAIL),
                    vec![thing(roles::FRAME, "Inbox").holding(
                        vec![thing(roles::FILLER, "").holding(
                (0..labels).map(|_| thing(roles::LABEL, "A letter")).collect(),
            )],
                    )],
                );
            ReadingWindows::of(
                Authorised::read(&reading(MAIL), &agent(), &grants, noon()).unwrap(),
                &Adapters::default(),
                &grants,
                &installed(),
                &strings,
            )
            .unwrap()
            .read(&tree, &strings)
            .unwrap()
            .shown()
            .not_all_read()
        };
    let most = alo_adapters::walking::MOST_THINGS;
    assert!(!shown(most - 1), "what fits was said as not all read");
    assert!(shown(most), "one past what is read was read as all");
}

/// **With no invocation the tree is never asked anything**, and neither is it
/// asked while anything before it is refused: declaring the verbs, loading the
/// adapters, proposing and approving, and every refusal of the grants, the
/// installed list or an application's own adapter all happen with the tree
/// untouched.
#[test]
fn with_no_invocation_the_tree_is_never_asked() {
    let strings = in_english();
    let tree = the_desktop();
    let grants = granting(&[MAIL, "org.gnome.TextEditor"]);
    let _ = fallback_verbs().unwrap();
    let adapters = shipped_adapters().unwrap();
    let call = pressing(MAIL, "button", "Sign in");
    let authorised = approved(&call, &grants);
    let activating =
        Activating::of(authorised, &adapters, &grants, &installed(), &strings).unwrap();
    assert_eq!(activating.application(), MAIL);
    assert!(tree.asked().is_empty(), "asked before anything was pressed");
    drop(activating);
    assert!(
        tree.asked().is_empty(),
        "asked by an authority dropped unused"
    );

    // The text editor has an adapter of its own.
    let refused = ReadingWindows::of(
        Authorised::read(&reading("org.gnome.TextEditor"), &agent(), &grants, noon()).unwrap(),
        &adapters,
        &grants,
        &installed(),
        &strings,
    )
    .unwrap_err();
    assert_eq!(
        refused.said(&strings).text(),
        "org.gnome.TextEditor has its own list of what the agent can do in it, so nothing was \
         read or pressed in its windows instead"
    );
    assert!(tree.asked().is_empty());
}

/// **Every refusal is recorded in the words a person reads, and nothing is
/// pressed**: not granted (identically whether installed or not), a grant
/// revoked between approval and pressing, an expired grant, not installed, an
/// application with its own adapter, a person's own application, another
/// verb's authority, no window, nothing to read windows through, an
/// application that does not answer while read, no such control, greyed out,
/// not pressable, and the application saying it did not press.
#[test]
fn every_refusal_is_recorded_and_nothing_is_pressed() {
    let strings = in_english();
    let adapters = shipped_adapters().unwrap();
    let mut record = Record::default();
    let mut keep = |refused: &Refused| {
        record.keep(Entry::refused(refused, &agent(), &strings, noon()));
    };
    let tree = the_desktop();

    // Not granted: refused before the tree, and the same whether installed.
    let grants = granting(&[PAYROLL]);
    let not_granted = Authorised::read(&reading(MAIL), &agent(), &grants, noon()).unwrap_err();
    let not_installed_either =
        Authorised::read(&reading("org.example.NotHere"), &agent(), &grants, noon()).unwrap_err();
    assert_eq!(
        not_granted.said(&strings).text().replace(MAIL, "X"),
        not_installed_either
            .said(&strings)
            .text()
            .replace("org.example.NotHere", "X")
    );
    keep(&not_granted);
    assert!(matches!(
        Proposal::checked(
            &pressing(MAIL, "button", "Sign in"),
            &agent(),
            &grants,
            noon(),
            hour()
        )
        .unwrap_err(),
        ProposalError::NotGranted(_)
    ));

    // Revoked between approval and pressing.
    let mut grants = granting(&[MAIL]);
    let authorised = approved(&pressing(MAIL, "button", "Sign in"), &grants);
    let id = grants.active_at(noon()).next().unwrap().id;
    assert!(grants.revoke(id));
    let revoked =
        Activating::of(authorised, &adapters, &grants, &installed(), &strings).unwrap_err();
    keep(&revoked);

    // Expired by the time it would run.
    let grants = granting(&[MAIL]);
    let mut approvals = Approvals::default();
    let id = approvals.propose(
        Proposal::checked(
            &pressing(MAIL, "button", "Sign in"),
            &agent(),
            &grants,
            noon(),
            hour(),
        )
        .unwrap(),
    );
    let expired = approvals
        .approve(id, noon())
        .unwrap()
        .redeem(&grants, noon() + hour() * 2)
        .unwrap_err();
    keep(&expired);

    // Granted and not installed.
    let grants = granting(&[MAIL]);
    let not_installed = ReadingWindows::of(
        Authorised::read(&reading(MAIL), &agent(), &grants, noon()).unwrap(),
        &adapters,
        &grants,
        &Installed::nothing(),
        &strings,
    )
    .unwrap_err();
    keep(&not_installed);

    // A person's own application: the grant cannot be made at all.
    assert_eq!(
        Grant::checked(
            "@alo",
            Reach::Application("app.devsuite.Ptyxis".to_owned()),
            noon(),
            hour()
        )
        .unwrap_err(),
        GrantError::APersonsOwn
    );

    // Another verb's authority.
    let another = Activating::of(
        Authorised::read(&reading(MAIL), &agent(), &grants, noon()).unwrap(),
        &adapters,
        &grants,
        &installed(),
        &strings,
    )
    .unwrap_err();
    assert_eq!(
        another.said(&strings).text(),
        "accessible.read_window is not reading or pressing something in an application's window, \
         so nothing was done"
    );
    keep(&another);
    assert!(
        tree.asked().is_empty(),
        "the tree was asked before a refusal"
    );

    // Refusals that come from the tree, each with its words.
    let press_on = |tree: &ATree, kind: &str, name: &str| {
        Activating::of(
            approved(&pressing(MAIL, kind, name), &grants),
            &adapters,
            &grants,
            &installed(),
            &strings,
        )
        .unwrap()
        .press(tree, WorkingFully::NothingMoreToTurnOn, &strings)
        .unwrap_err()
    };
    let cases: Vec<(ATree, &str, &str, &str)> = vec![
        (
            ATree::default().with(":1.7", Some(MAIL), vec![]),
            "button",
            "Sign in",
            "org.example.Mail has no window on the screen, so there was nothing to read or press",
        ),
        (
            the_desktop().broken(TreeFault::NotThere),
            "button",
            "Sign in",
            "What applications show could not be read on this machine just now, so nothing was \
             read or pressed in org.example.Mail",
        ),
        (
            the_desktop().broken(TreeFault::DidNotAnswer),
            "button",
            "Sign in",
            "org.example.Mail did not answer in time, so nothing was read or pressed in it",
        ),
        (
            the_desktop(),
            "check_box",
            "Sign in",
            "org.example.Mail shows no check box named “Sign in” now, so nothing was pressed",
        ),
        (
            the_desktop(),
            "button",
            "Delete account",
            "The button named “Delete account” in org.example.Mail cannot be used right now, so it \
             was not pressed",
        ),
        (
            the_desktop(),
            "button",
            "Details",
            "The button named “Details” in org.example.Mail cannot be pressed from outside the \
             application, so nothing was pressed",
        ),
        (
            the_desktop().pressing_answers(Ok(false)),
            "check_box",
            "Remember me",
            "org.example.Mail did not press the check box named “Remember me”",
        ),
    ];
    for (tree, kind, name, words) in &cases {
        let refused = press_on(tree, kind, name);
        assert_eq!(refused.said(&strings).text(), *words);
        keep(&refused);
    }
    let pressed_without_answer = cases
        .iter()
        .filter(|(tree, ..)| tree.presses_asked() > 0)
        .count();
    // Only the application that answered "not pressed" was asked to press.
    assert_eq!(pressed_without_answer, 1);

    // A read refused by the tree is recorded the same way.
    let no_window = ReadingWindows::of(
        Authorised::read(&reading(MAIL), &agent(), &grants, noon()).unwrap(),
        &adapters,
        &grants,
        &installed(),
        &strings,
    )
    .unwrap()
    .read(&ATree::default(), &strings)
    .unwrap_err();
    keep(&no_window);

    let entries: Vec<_> = record.everything().collect();
    assert_eq!(entries.len(), 13);
    for entry in entries {
        assert!(!entry.happened().ran(), "a refusal was recorded as run");
    }
}

/// **An application that does not answer the press is recorded as having
/// run**, because it may have pressed it, and the person is told that whether
/// it did is not known.
#[test]
fn an_unanswered_press_is_recorded_as_run_and_said_as_not_known() {
    let strings = in_english();
    let grants = granting(&[MAIL]);
    let tree = the_desktop().pressing_answers(Err(TreeFault::DidNotAnswer));
    let pressed = Activating::of(
        approved(&pressing(MAIL, "check_box", "Remember me"), &grants),
        &Adapters::default(),
        &grants,
        &installed(),
        &strings,
    )
    .unwrap()
    .press(&tree, WorkingFully::NothingMoreToTurnOn, &strings)
    .unwrap();
    assert!(pressed.unanswered());
    assert_eq!(
        pressed.said(&strings).unwrap().text(),
        "org.example.Mail did not answer in time, so whether the check box named “Remember me” \
         was pressed is not known"
    );
    assert_eq!(tree.presses_asked(), 1);
    let mut record = Record::default();
    record.keep(Entry::ran(&pressed.into_authorised(), &strings));
    assert!(record.everything().next().unwrap().happened().ran());
}
