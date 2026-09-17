//! Dropping a file onto the agent's surface offers it for that question, and
//! **no grant exists afterwards**.
//!
//! `docs/autonomy/v0-5-hands-on-the-desktop-plan.md` task 4: *dropping a file
//! onto an agent's surface offers it as context for that turn only and is not a
//! grant — a test holds that no grant exists afterwards, because a grant is made
//! by `alo-picking` and nothing else (ADR 0001 §3).* The plan's constraint says
//! it in five words: **no *drop to grant*, however convenient it looks.**
//!
//! This file is that test, and it asks the question three ways, because one way
//! would be easy to satisfy by accident:
//!
//! 1. the machine's own grant list is **empty before and empty after**, and the
//!    agent may reach neither the file that was dropped nor the folder it was in;
//! 2. the offer is read **once** and only **while the question lasts**;
//! 3. this crate has no `alo-capability` dependency at all outside its tests, so
//!    there is nothing linked into it that could make a grant in the first place.

#![expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected Err is the failure being reported"
)]

use std::time::{Duration, SystemTime};

use alo_capability::{Ask, Grantee, Grants};
use alo_clipboard::{CouldNotGive, Gives, Kind, Offer};
use alo_handing::{Drag, LetGo, NotHanded, Over, the_agents_surface};

/// Where the invoice a person dragged actually lives.
const THE_INVOICE: &str = "/home/anna/Invoices/march.pdf";

/// The list of files a drag of that invoice carries.
const THE_LIST: &[u8] = b"file:///home/anna/Invoices/march.pdf\n";

/// A moment, as seconds after the epoch.
fn at(seconds: u64) -> SystemTime {
    SystemTime::UNIX_EPOCH + Duration::from_secs(seconds)
}

/// The files window, which is where the drag came from.
struct TheFilesWindow;

impl Gives for TheFilesWindow {
    fn give(&mut self, _form: &Kind) -> Result<Vec<u8>, CouldNotGive> {
        Ok(THE_LIST.to_vec())
    }
}

/// A drag of that invoice.
fn dragging_the_invoice() -> Drag {
    Drag::begun(
        Offer::copied(vec![Kind::files()]).expect("a form is an offer"),
        Box::new(TheFilesWindow),
    )
}

/// **No grant exists afterwards.** A person drags an invoice onto the agent's
/// panel to ask what is in it; the agent can read what was dropped, for that
/// question, and the machine's grant list is exactly as empty as it was before
/// they picked the file up.
///
/// The obvious implementation is the other one — grant the agent that path so it
/// can open the file — and it is the one this test exists to fail. ADR 0001 §3
/// names the two deliberate acts that make a grant, a folder chosen in a picker
/// and the document offered at invocation, and a drag of the wrist is neither.
#[test]
fn no_grant_exists_after_a_file_is_dropped_on_the_agents_surface() {
    let now = at(100);
    let grants = Grants::default();
    let agent = Grantee::named("@files");
    assert!(grants.is_empty());

    let drag = dragging_the_invoice();
    let surface = the_agents_surface(at(400));
    assert_eq!(
        drag.over(&surface),
        Over::WouldOfferItForThisQuestion {
            form: Kind::files()
        }
    );
    let LetGo::OfferedForThisQuestion(offered) =
        drag.let_go_on(&surface).expect("the agent's surface")
    else {
        unreachable!("it was dropped on the agent")
    };

    // What the agent can read is what was dropped. Nothing else changed.
    assert!(offered.offers(&Kind::files()));
    assert_eq!(offered.read(&Kind::files(), now), Ok(THE_LIST.to_vec()));

    // **The list is empty, and the agent reaches neither the file nor its
    // folder** — not while the question lasts, and not afterwards.
    assert!(grants.is_empty(), "a drop made a grant");
    assert_eq!(grants.len(), 0);
    assert!(!grants.permits(&agent, &Ask::path(THE_INVOICE), now));
    assert!(!grants.permits(&agent, &Ask::path("/home/anna/Invoices"), now));
    assert!(!grants.permits(&agent, &Ask::path(THE_INVOICE), at(10_000)));
    assert!(grants.active_at(now).next().is_none());
}

/// **For that turn only**, and the two halves of it: the question ends and the
/// offer answers nothing, and a form nobody offered is refused without the
/// window that had it being woken up at all.
#[test]
fn what_was_dropped_is_offered_for_that_question_and_no_longer() {
    let LetGo::OfferedForThisQuestion(offered) = dragging_the_invoice()
        .let_go_on(&the_agents_surface(at(400)))
        .expect("the agent's surface")
    else {
        unreachable!("it was dropped on the agent")
    };
    assert_eq!(offered.ends(), at(400));
    assert_eq!(
        offered.read(&Kind::files(), at(400)),
        Err(NotHanded::TheQuestionIsOver)
    );

    let LetGo::OfferedForThisQuestion(offered) = dragging_the_invoice()
        .let_go_on(&the_agents_surface(at(400)))
        .expect("the agent's surface")
    else {
        unreachable!("it was dropped on the agent")
    };
    assert_eq!(
        offered.read(&Kind::text(), at(100)),
        Err(NotHanded::NotThatForm { form: Kind::text() })
    );
}

/// **There is nothing in this crate to make a grant with.** `alo-capability`
/// appears in its manifest under `[dev-dependencies]` and nowhere else, so the
/// promise above is not a rule somebody remembers: the types a grant is made of
/// are not linked into the code that ships.
///
/// The shape is `alo-clipboard`'s `the_clipboard_is_not_a_turns_context.rs`,
/// which reads its own manifest for the same reason.
#[test]
fn nothing_that_ships_in_this_crate_could_make_a_grant() {
    let manifest = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/Cargo.toml"))
        .expect("this crate's own manifest");
    let (dependencies, development) = manifest
        .split_once("[dev-dependencies]")
        .expect("this crate has dev-dependencies");

    assert!(
        !dependencies.contains("alo-capability"),
        "alo-capability is a dependency of alo-handing, so a drop could make a grant"
    );
    assert!(
        development.contains("alo-capability"),
        "the test above needs alo-capability as a dev-dependency"
    );
    // And no road to one through `alo-picking` either, which is where ADR 0001
    // §3 says a grant is made.
    assert!(
        !dependencies.contains("alo-picking"),
        "alo-picking is a dependency of alo-handing"
    );
}
