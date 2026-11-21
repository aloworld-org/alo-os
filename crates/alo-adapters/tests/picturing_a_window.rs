//! **Taking one picture of an application's window** (`accessible.picture_window`,
//! ADR 0147 §1), against a stand-in tree — every refusal beside what it
//! refuses.
//!
//! The adapter's part is the asking before any picture is taken: the grants,
//! the application installed with no adapter of its own, the person's
//! *Pictures and keys* switch for that application, somebody chosen to read
//! pictures, and a tree that shows no password field and describes nothing.
//! The picture itself is the compositor's, and `alo-turn`'s tests hold the
//! order around both.

#![expect(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::collections::BTreeMap;
use std::time::{Duration, SystemTime};

use alo_adapters::fallback_verbs::{self, APPLICATION, PICTURE_WINDOW};
use alo_adapters::picture_words as words;
use alo_adapters::{
    AccessibilityTree, Adapters, Facts, NodeAt, PicturingAWindow, Running, TreeFault,
};
use alo_applications::{Application, Installed};
use alo_capability::{
    Approvals, Authorised, Call, Effect, Given, Grant, Grantee, Grants, PicturesAndKeys, Proposal,
    Refused, Takes, WhoReadsPictures,
};
use alo_strings::{Filling, Strings, Vocabulary};

/// The game pictured.
const GAME: &str = "net.supertuxkart.SuperTuxKart";

/// `ATSPI_ROLE_FRAME`, `_PANEL`, `_CANVAS`, `_PASSWORD_TEXT`, `_PUSH_BUTTON`.
const FRAME: u32 = 23;
const PANEL: u32 = 39;
const CANVAS: u32 = 6;
const PASSWORD: u32 = 40;
const BUTTON: u32 = 43;

/// Showing.
const SHOWING: u32 = (1 << 25) | (1 << 30);

fn noon() -> SystemTime {
    SystemTime::UNIX_EPOCH + Duration::from_secs(1_760_000_000)
}

fn hour() -> Duration {
    Duration::from_secs(60 * 60)
}

fn in_english() -> Strings {
    let mut vocabulary = Vocabulary::empty();
    alo_capability::declare_into(&mut vocabulary).unwrap();
    alo_applications::words::declare_into(&mut vocabulary).unwrap();
    alo_adapters::words::declare_into(&mut vocabulary).unwrap();
    Strings::of(vocabulary)
}

fn granting(applications: &[&str]) -> Grants {
    let mut grants = Grants::default();
    for application in applications {
        grants.grant(
            Grant::checked(
                "@alo",
                alo_capability::Reach::Application((*application).to_owned()),
                noon(),
                hour(),
            )
            .unwrap(),
        );
    }
    grants
}

fn installed() -> Installed {
    Installed::holding([Application::called(GAME, "SuperTuxKart").unwrap()])
}

fn a_picture_call() -> Call {
    fallback_verbs::fallback_verbs()
        .unwrap()
        .call(
            PICTURE_WINDOW,
            &[
                (APPLICATION, Given::text(GAME)),
                (fallback_verbs::QUESTION, Given::text("Is a race on?")),
            ],
        )
        .unwrap()
}

fn approved(call: &Call, grants: &Grants) -> Authorised {
    let mut approvals = Approvals::default();
    let id = approvals
        .propose(Proposal::checked(call, &Grantee::named("@alo"), grants, noon(), hour()).unwrap());
    approvals
        .approve(id, noon())
        .unwrap()
        .redeem(grants, noon())
        .unwrap()
}

fn switched_on() -> PicturesAndKeys {
    PicturesAndKeys::of([GAME])
}

fn a_reader() -> WhoReadsPictures {
    WhoReadsPictures::a_provider("Mistral").unwrap()
}

fn asking(
    pictures: &PicturesAndKeys,
    reading: &WhoReadsPictures,
) -> Result<PicturingAWindow, Refused> {
    let grants = granting(&[GAME]);
    PicturingAWindow::of(
        approved(&a_picture_call(), &grants),
        &Adapters::default(),
        &grants,
        &installed(),
        pictures,
        reading,
        &in_english(),
    )
}

fn picturing() -> PicturingAWindow {
    asking(&switched_on(), &a_reader()).expect("granted, installed, switched on, a reader")
}

fn at(object: &str) -> NodeAt {
    NodeAt {
        holder: ":1.9".to_owned(),
        object: object.to_owned(),
    }
}

/// A tree with one window of the game holding things of these roles and
/// names, none with text — or, with `on_the_bus` false, no game at all.
struct ATree {
    facts: BTreeMap<NodeAt, Facts>,
    on_the_bus: bool,
}

impl ATree {
    fn of(things: &[(u32, &str)]) -> Self {
        let children: Vec<NodeAt> = (0..things.len()).map(|n| at(&format!("/t/{n}"))).collect();
        let facts_of = |role, name: &str, children| Facts {
            role,
            name: name.to_owned(),
            states: vec![SHOWING, 0],
            children,
            has_text: false,
            unlisted_children: false,
        };
        let mut facts = BTreeMap::from([
            (at("/root"), facts_of(75, "", vec![at("/window")])),
            (
                at("/window"),
                facts_of(FRAME, "SuperTuxKart", children.clone()),
            ),
        ]);
        for ((role, name), place) in things.iter().zip(children) {
            facts.insert(place, facts_of(*role, name, Vec::new()));
        }
        Self {
            facts,
            on_the_bus: true,
        }
    }

    fn not_on_the_bus() -> Self {
        Self {
            facts: BTreeMap::new(),
            on_the_bus: false,
        }
    }
}

impl AccessibilityTree for ATree {
    fn applications(&self) -> Result<Vec<Running>, TreeFault> {
        Ok(if self.on_the_bus {
            vec![Running {
                at: at("/root"),
                is: Some(GAME.to_owned()),
            }]
        } else {
            Vec::new()
        })
    }

    fn facts(&self, place: &NodeAt) -> Result<Facts, TreeFault> {
        self.facts.get(place).cloned().ok_or(TreeFault::Vanished)
    }

    fn text(&self, _: &NodeAt) -> Result<String, TreeFault> {
        Ok(String::new())
    }

    fn actions(&self, _: &NodeAt) -> Result<Vec<String>, TreeFault> {
        Ok(Vec::new())
    }

    fn act(&self, _: &NodeAt, _: usize) -> Result<bool, TreeFault> {
        Ok(false)
    }
}

/// A tree that cannot be reached at all.
struct NoTree;

impl AccessibilityTree for NoTree {
    fn applications(&self) -> Result<Vec<Running>, TreeFault> {
        Err(TreeFault::NotThere)
    }

    fn facts(&self, _: &NodeAt) -> Result<Facts, TreeFault> {
        Err(TreeFault::NotThere)
    }

    fn text(&self, _: &NodeAt) -> Result<String, TreeFault> {
        Err(TreeFault::NotThere)
    }

    fn actions(&self, _: &NodeAt) -> Result<Vec<String>, TreeFault> {
        Err(TreeFault::NotThere)
    }

    fn act(&self, _: &NodeAt, _: usize) -> Result<bool, TreeFault> {
        Err(TreeFault::NotThere)
    }
}

fn said(refused: &Refused) -> String {
    refused.said(&in_english()).text().to_owned()
}

fn expected(word: alo_strings::Word) -> String {
    in_english()
        .say(&word.key(), &Filling::of("application", GAME.to_owned()))
        .text()
        .to_owned()
}

/// **One argument, the application, and nothing else**: no place, no region
/// and no second window can arrive in a call.
#[test]
fn the_verb_is_a_change_over_a_granted_application_taking_nothing_else() {
    let verbs = fallback_verbs::fallback_verbs().unwrap();
    let verb = verbs.of(PICTURE_WINDOW).unwrap();
    assert_eq!(verb.effect(), Effect::Change);
    assert_eq!(verb.what_it_changes(), Some(&words::PICTURE_CHANGES));
    assert_eq!(verb.what_it_leaves(), Some(&words::PICTURE_LEAVES));
    assert_eq!(verb.args().len(), 2);
    assert!(matches!(
        verb.args().first().map(alo_capability::Arg::takes),
        Some(Takes::Application)
    ));
    assert!(matches!(
        verb.args().get(1).map(alo_capability::Arg::takes),
        Some(Takes::Text)
    ));
    assert!(
        verbs
            .call(
                PICTURE_WINDOW,
                &[
                    (APPLICATION, Given::text(GAME)),
                    (fallback_verbs::QUESTION, Given::text("Is a race on?")),
                    ("region", Given::text("0,0,1,1")),
                ],
            )
            .is_err(),
        "a second argument was taken"
    );
}

#[test]
fn a_window_that_describes_nothing_may_be_pictured() {
    for tree in [
        ATree::of(&[]),
        ATree::of(&[(PANEL, ""), (CANVAS, "SuperTuxKart")]),
    ] {
        picturing().may_be_pictured(&tree, &in_english()).unwrap();
    }
    assert_eq!(picturing().application(), GAME);
}

/// **A game that is not on the accessibility bus at all describes nothing**,
/// and is the very application the screen road is for: the compositor answers
/// whether a window of it is in front.
#[test]
fn an_application_the_tree_does_not_know_describes_nothing() {
    picturing()
        .may_be_pictured(&ATree::not_on_the_bus(), &in_english())
        .unwrap();
}

#[test]
fn a_window_that_describes_itself_is_read_instead() {
    let tree = ATree::of(&[(PANEL, ""), (BUTTON, "Start race")]);
    let refused = picturing()
        .may_be_pictured(&tree, &in_english())
        .unwrap_err();
    assert_eq!(said(&refused), expected(words::IT_DESCRIBES_ITSELF));
}

/// **A password field anywhere is no picture**, even a nameless one, and even
/// where it is the only thing the window describes.
#[test]
fn a_password_field_is_never_pictured() {
    for tree in [
        ATree::of(&[(PASSWORD, "")]),
        ATree::of(&[(BUTTON, "Sign in"), (PASSWORD, "Password")]),
    ] {
        let refused = picturing()
            .may_be_pictured(&tree, &in_english())
            .unwrap_err();
        assert_eq!(said(&refused), expected(words::A_PASSWORD_FIELD));
    }
}

/// **A tree that cannot be read cannot say there is no password field**, so
/// no picture is taken.
#[test]
fn a_tree_that_cannot_be_read_is_no_picture() {
    assert!(picturing().may_be_pictured(&NoTree, &in_english()).is_err());
}

/// **Pictures are off for an application until the person switches them on
/// for it** — another application switched on is not this one.
#[test]
fn pictures_off_for_the_application_are_refused() {
    for pictures in [
        PicturesAndKeys::default(),
        PicturesAndKeys::of(["org.kde.krita"]),
    ] {
        let refused = asking(&pictures, &a_reader()).unwrap_err();
        assert_eq!(said(&refused), expected(words::NOT_SWITCHED_ON));
    }
}

/// **Nobody chosen to read pictures is no picture**, whatever the switch says.
#[test]
fn nobody_chosen_to_read_is_refused() {
    let refused = asking(&switched_on(), &WhoReadsPictures::NoOne).unwrap_err();
    assert_eq!(said(&refused), expected(words::NO_ONE_READS));
    // Either reader is enough to go on to the window.
    for reader in [WhoReadsPictures::ThisComputer, a_reader()] {
        assert!(asking(&switched_on(), &reader).is_ok(), "{reader:?}");
    }
}

#[test]
fn an_application_not_granted_is_refused_before_anything_else() {
    let granted = granting(&[GAME]);
    let authorised = approved(&a_picture_call(), &granted);
    let refused = PicturingAWindow::of(
        authorised,
        &Adapters::default(),
        &Grants::default(),
        &installed(),
        &switched_on(),
        &a_reader(),
        &in_english(),
    )
    .unwrap_err();
    assert_ne!(said(&refused), expected(words::NOT_SWITCHED_ON));
}

/// **A read is told a picture could say more only where one may be taken**:
/// windows that describe nothing, no password field, the switch on for that
/// application, and somebody reading pictures.
#[test]
fn a_read_is_told_of_a_picture_only_where_one_may_be_taken() {
    use alo_adapters::shown::{Seen, Shown, ShownWindow};
    use alo_adapters::{Role, a_picture_could_say_more};
    let of = |seen: Vec<Seen>| {
        Shown::of(
            GAME.to_owned(),
            vec![ShownWindow::titled("SuperTuxKart".to_owned(), seen)],
            false,
        )
    };
    let nothing = of(vec![Seen::of(Role::Canvas, String::new(), None, 1)]);
    let strings = in_english();
    let said = a_picture_could_say_more(&nothing, &switched_on(), &a_reader(), &strings)
        .expect("a picture may be taken");
    assert_eq!(said.text(), expected(words::A_PICTURE_CAN_BE_READ));
    for (shown, pictures, reader) in [
        (&nothing, PicturesAndKeys::default(), a_reader()),
        (&nothing, switched_on(), WhoReadsPictures::NoOne),
        (
            &of(vec![Seen::of(
                Role::Button,
                "Start race".to_owned(),
                None,
                1,
            )]),
            switched_on(),
            a_reader(),
        ),
        (
            &of(vec![Seen::of(Role::PasswordField, String::new(), None, 1)]),
            switched_on(),
            a_reader(),
        ),
    ] {
        assert!(
            a_picture_could_say_more(shown, &pictures, &reader, &strings).is_none(),
            "told of a picture that may not be taken: {pictures:?} {reader:?}"
        );
    }
}
