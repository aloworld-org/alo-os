//! One list of what has been granted to what.
//!
//! Task 2 of `docs/autonomy/v0-5-applications-and-what-they-expect-plan.md`,
//! each clause of its acceptance a test here:
//!
//! - **an application's grants beside an agent's, in one order, with one shape
//!   per row — who, what, until when — and no row told apart by anything but
//!   the name** — [`a_mixed_list_has_no_row_told_apart_but_by_the_name`];
//! - **revoking an application's grant is the same action as revoking an
//!   agent's, and takes effect at the next portal request, with a request in
//!   flight** —
//!   [`revoking_an_applications_grant_stops_the_request_in_flight_the_way_an_agents_verb_is_stopped`];
//! - **an application that has been granted nothing does not appear** —
//!   [`an_application_granted_nothing_does_not_appear`];
//! - **every sentence the list shows is in the vocabulary `alo-saying`
//!   collects** — [`every_sentence_the_one_list_shows_is_collected`].
//!
//! And the plan's constraint: **the list is derived, never a second store**.
//! Every list here is read back through the grants file's own text
//! (`alo_remembering::written` and `alo_remembering::read`) — the format-2 file
//! an application's grant makes — and the portal is judged against the same
//! value the list was derived from.
//!
//! It runs nothing on the machine: the portal is `alo-portals`' decision, not a
//! bus, so it holds on any host.

#![expect(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::path::PathBuf;
use std::sync::mpsc;
use std::sync::{Arc, RwLock};
use std::thread;
use std::time::{Duration, SystemTime};

use alo_capability::{
    Agent, Applicant, Ask, Authorised, Call, Effect, Facility, Given, Grant, Grantee, Grants,
    NotAllowed, Reach, Requires, Takes, Verb,
};
use alo_granted::{Listing, Revoked, Seen};
use alo_portals::{Portal, Refused, Request};
use alo_strings::{Strings, Word};

/// A fixed moment, so that everything about time here is arithmetic.
fn noon() -> SystemTime {
    SystemTime::UNIX_EPOCH + Duration::from_secs(1_760_000_000)
}

/// How long the grants these tests read last.
fn hour() -> Duration {
    Duration::from_secs(60 * 60)
}

/// The agent on every list here.
const FILES: &str = "@files";
/// An application granted the same folder as the agent.
const PAPERS: &str = "org.gnome.Papers";
/// An application granted facilities.
const CHEESE: &str = "org.gnome.Cheese";
/// The folder granted to both kinds.
const SHARED: &str = "/home/anna/Shared";

/// The machine's one vocabulary, which is the `Strings` a real shell holds.
fn what_the_machine_can_say() -> Strings {
    Strings::of(alo_saying::everything_this_machine_can_say().unwrap())
}

/// Grants as a machine keeps them: written into the grants file's text and
/// read back, so the list is derived from what the daemon itself would read.
fn kept(made: &Grants) -> Grants {
    let text = alo_remembering::written(made, noon()).unwrap();
    alo_remembering::read(&text, noon()).unwrap()
}

/// A grant to an application, as a person's allowing makes one.
fn to_application(application: &str, reach: Reach) -> Grant {
    Grant::checked_for(
        &Applicant::named(application).grantee(),
        reach,
        noon(),
        hour(),
    )
    .unwrap()
}

/// A grant to an agent, as a person's pick makes one.
fn to_agent(agent: &str, reach: Reach) -> Grant {
    Grant::checked(agent, reach, noon(), hour()).unwrap()
}

/// The one read verb the agent's side of these tests is refused.
const LISTING: Word = Word::saying(
    "testing.verb.list-folder.purpose",
    "list what is in a folder",
);
/// What a person is shown while it happens.
const LISTING_SENTENCE: Word = Word::saying(
    "testing.verb.list-folder.sentence",
    "list what is in {folder}",
);
/// What it lists.
const LISTING_FOLDER: Word = Word::saying(
    "testing.verb.list-folder.argument.folder",
    "the folder to list",
);

/// Listing the shared folder, as the daemon would be asked it.
fn listing_the_shared_folder() -> Call {
    let verb = Verb::checked(
        "list_folder",
        LISTING,
        Effect::Read,
        vec![alo_capability::Arg::taking(
            "folder",
            LISTING_FOLDER,
            Takes::Path,
        )],
        Requires::grants_over(["folder"]),
        LISTING_SENTENCE,
    )
    .unwrap();
    Call::of(&verb, &[("folder", Given::text(SHARED))]).unwrap()
}

/// The row a listing holds for this name.
fn row_for(listing: &Listing, name: &str) -> Seen {
    listing
        .rows()
        .iter()
        .find(|row| row.to() == name)
        .unwrap_or_else(|| unreachable!("no row for {name} in {listing:?}"))
        .clone()
}

/// **One list, one order, one shape, and nothing but the name to tell an
/// application's row from an agent's.**
///
/// An agent and an application are granted the same folder at the same moment
/// for the same hour, with an application's camera and an agent's application
/// between and after them. The listing, read back off the kept file, holds all
/// four in the order they were made. The agent's folder row and the
/// application's folder row are then compared on everything a row has — every
/// accessor, the sentence in two languages, and the row's whole `Debug` form —
/// and, once each name is put in the same place, the only other difference is
/// the handle, which is unique to every grant by construction and is how a
/// revocation lands, not something that says whose it is.
#[test]
fn a_mixed_list_has_no_row_told_apart_but_by_the_name() {
    let mut made = Grants::default();
    made.grant(to_agent(FILES, Reach::Folder(PathBuf::from(SHARED))));
    made.grant(to_application(CHEESE, Reach::Facility(Facility::Camera)));
    made.grant(to_application(PAPERS, Reach::Folder(PathBuf::from(SHARED))));
    made.grant(to_agent(
        "@blender",
        Reach::Application("org.blender.Blender".to_owned()),
    ));
    let grants = kept(&made);

    let listing = Listing::of(&grants, noon());
    let named: Vec<&str> = listing.rows().iter().map(Seen::to).collect();
    assert_eq!(named, [FILES, CHEESE, PAPERS, "@blender"], "one order");

    let agents = row_for(&listing, FILES);
    let applications = row_for(&listing, PAPERS);

    // Who, what, until when — and only who differs.
    assert_ne!(agents.to(), applications.to());
    assert_eq!(agents.over(), applications.over());
    assert_eq!(agents.granted_at(), applications.granted_at());
    assert_eq!(agents.expires_in(), applications.expires_in());
    assert_ne!(agents.id(), applications.id());

    // The sentence is one clause for both, in English and in German.
    let german = {
        let vocabulary = alo_saying::everything_this_machine_can_say().unwrap();
        let language = alo_strings::Language::written("de").unwrap();
        let translation = alo_strings::Translation::into_language(language.clone()).says(
            alo_granted::words::ONE_GRANT.key(),
            "{who} wurde {what} gewährt",
        );
        let speaking = vocabulary.check(translation).unwrap();
        let mut strings = Strings::of(vocabulary);
        strings.speaks(speaking).unwrap();
        strings.prefers(&[language]);
        strings
    };
    for strings in [what_the_machine_can_say(), german] {
        let agent_said = agents.said(&strings);
        let application_said = applications.said(&strings);
        assert!(!agent_said.is_a_bug(), "{agent_said}");
        assert_eq!(agent_said.is_translated(), application_said.is_translated());
        assert_eq!(
            agent_said.text().replacen(FILES, "{who}", 1),
            application_said.text().replacen(PAPERS, "{who}", 1),
            "the two rows are worded differently"
        );
    }

    // And the whole row, field by field, with the name and handle in the same
    // place: nothing in it says which kind of grantee it is.
    let without = |row: &Seen, name: &str| {
        format!("{row:?}").replacen(name, "{who}", 1).replacen(
            &format!("GrantId({})", row.id().as_u64()),
            "GrantId(_)",
            1,
        )
    };
    assert_eq!(without(&agents, FILES), without(&applications, PAPERS));
    let debugged = format!("{applications:?}").to_ascii_lowercase();
    assert!(!debugged.contains("applica"), "{debugged}");
    assert!(!debugged.contains("agent"), "{debugged}");

    // The rows between are the same shape too: the camera row is the clause
    // every row is.
    let camera = row_for(&listing, CHEESE).said(&what_the_machine_can_say());
    assert_eq!(
        camera.text(),
        "org.gnome.Cheese has been granted the camera"
    );
}

/// **Revoking an application's grant is the same action as revoking an
/// agent's, and a request already in flight is refused.**
///
/// A portal backend runs on its own thread with the machine's one list behind
/// a lock, the way a daemon holds it. Cheese holds the camera and the
/// microphone; `@files` holds a folder. A camera request is received and
/// allowed, then a second one is received and held — in flight, checked and
/// not yet judged. The person revokes Cheese's camera row with
/// [`Seen::revoke`], the one method, and then `@files`' row with the same
/// method. The held request is released and judged at that moment: refused
/// with `alo-capability`'s *never granted*, while the microphone, never
/// revoked, is still allowed, and the agent's verb is refused as well. Once
/// the microphone's row goes too the application holds nothing and is refused
/// before anything it asked for is looked at.
#[test]
fn revoking_an_applications_grant_stops_the_request_in_flight_the_way_an_agents_verb_is_stopped() {
    let mut made = Grants::default();
    made.grant(to_application(CHEESE, Reach::Facility(Facility::Camera)));
    made.grant(to_agent(FILES, Reach::Folder(PathBuf::from(SHARED))));
    made.grant(to_application(
        CHEESE,
        Reach::Facility(Facility::Microphone),
    ));
    let machine = Arc::new(RwLock::new(kept(&made)));

    // The backend: receives a request, waits until told to judge it, answers.
    let (requests, received) = mpsc::channel::<(Request, mpsc::Receiver<()>)>();
    let (answers, answered) = mpsc::channel::<Result<Portal, Refused>>();
    let backend = {
        let machine = Arc::clone(&machine);
        thread::spawn(move || {
            for (request, judge_now) in received {
                judge_now
                    .recv()
                    .expect("the test lets every request be judged");
                let grants = machine.read().expect("nobody panicked holding the list");
                let answer = request
                    .judged(&grants, noon())
                    .map(|allowed| allowed.portal());
                answers
                    .send(answer)
                    .expect("the test waits for every answer");
            }
        })
    };
    let ask = |portal: Portal| {
        let (go, judge_now) = mpsc::channel();
        requests
            .send((Request::of(CHEESE, portal).unwrap(), judge_now))
            .unwrap();
        go
    };

    // Before: the camera is allowed.
    ask(Portal::Camera).send(()).unwrap();
    assert_eq!(answered.recv().unwrap(), Ok(Portal::Camera));

    // A second camera request arrives and is held in flight.
    let in_flight = ask(Portal::Camera);

    // The person revokes both kinds of row, through the one method.
    {
        let mut grants = machine.write().unwrap();
        let listing = Listing::of(&grants, noon());
        let camera = listing
            .rows()
            .iter()
            .find(|row| row.over() == &Reach::Facility(Facility::Camera))
            .unwrap()
            .clone();
        let folder = row_for(&listing, FILES);
        assert_eq!(camera.revoke(&mut grants), Revoked::Now);
        assert_eq!(folder.revoke(&mut grants), Revoked::Now);
        // And the same answer to a second revocation of either.
        assert_eq!(camera.revoke(&mut grants), Revoked::AlreadyGone);
        assert_eq!(folder.revoke(&mut grants), Revoked::AlreadyGone);
    }

    // The request in flight is judged now, against the list as it now is.
    in_flight.send(()).unwrap();
    let refused = answered.recv().unwrap().unwrap_err();
    match &refused {
        Refused::NotAllowed {
            portal: Portal::Camera,
            why:
                NotAllowed::Never {
                    application,
                    wanted,
                },
        } => {
            assert_eq!(application, &Applicant::named(CHEESE));
            assert_eq!(wanted, &Ask::facility(Facility::Camera));
        }
        other => unreachable!("the revoked camera was answered {other:?}"),
    }
    let said = refused.said(&what_the_machine_can_say());
    assert!(!said.is_a_bug(), "{said}");

    // What was not revoked still stands.
    ask(Portal::Microphone).send(()).unwrap();
    assert_eq!(answered.recv().unwrap(), Ok(Portal::Microphone));

    // The agent's verb, revoked by the same method, is refused the same way.
    let refused_verb = Authorised::read(
        &listing_the_shared_folder(),
        &Grantee::named(FILES),
        &machine.read().unwrap(),
        noon(),
    )
    .unwrap_err();
    assert!(matches!(
        refused_verb.why(),
        alo_capability::NotAuthorised::NotGranted(alo_capability::NotGranted::Never { .. })
    ));

    // The last of Cheese's rows goes, and it is refused before anything is
    // looked at — and has no row left.
    {
        let mut grants = machine.write().unwrap();
        let microphone = row_for(&Listing::of(&grants, noon()), CHEESE);
        assert_eq!(microphone.revoke(&mut grants), Revoked::Now);
        assert!(Listing::of(&grants, noon()).is_nothing_granted());
    }
    ask(Portal::Microphone).send(()).unwrap();
    assert!(matches!(
        answered.recv().unwrap(),
        Err(Refused::NothingGranted {
            portal: Portal::Microphone,
            ..
        })
    ));

    drop(requests);
    backend.join().unwrap();
}

/// **An application granted nothing does not appear**, whichever way it came
/// to hold nothing: it asked and was refused, its grant expired, or its grant
/// was revoked. The refusal path beside it: asking never adds a row.
#[test]
fn an_application_granted_nothing_does_not_appear() {
    let mut made = Grants::default();
    made.grant(to_agent(FILES, Reach::Folder(PathBuf::from(SHARED))));
    let lasting_a_minute = Grant::checked_for(
        &Applicant::named(PAPERS).grantee(),
        Reach::File(PathBuf::from("/home/anna/Shared/march.pdf")),
        noon(),
        Duration::from_secs(60),
    )
    .unwrap();
    made.grant(lasting_a_minute);
    let revoked = made.grant(to_application(CHEESE, Reach::Facility(Facility::Camera)));
    assert!(made.revoke(revoked));
    let grants = kept(&made);

    // A stranger asks, a hundred times, and is refused before any dialog.
    let before = serde_json::to_string(&grants).unwrap();
    for _ in 0..100 {
        let asked = Request::of("org.example.Stranger", Portal::ScreenCapture).unwrap();
        assert!(matches!(
            asked.judged(&grants, noon()),
            Err(Refused::NothingGranted { .. })
        ));
    }
    assert_eq!(serde_json::to_string(&grants).unwrap(), before);

    let names = |at: SystemTime| -> Vec<String> {
        Listing::of(&grants, at)
            .rows()
            .iter()
            .map(|row| row.to().to_owned())
            .collect()
    };
    // At noon: the agent and Papers. Not the stranger, not revoked Cheese.
    assert_eq!(names(noon()), [FILES, PAPERS]);
    // Past Papers' minute: the agent alone.
    assert_eq!(names(noon() + Duration::from_secs(60)), [FILES]);

    // A declined machine's list is the applications' grants, and an
    // application whose grant ended has no row there either.
    let mut declined = Agent::present();
    assert!(
        declined
            .allow(to_application(
                CHEESE,
                Reach::Facility(Facility::Microphone)
            ))
            .is_some()
    );
    assert_eq!(declined.declining(noon()), 0, "no agent's grant to end");
    let listing = Listing::of(declined.allowed(), noon());
    assert_eq!(listing.rows().len(), 1);
    let row = listing.rows().first().unwrap().clone();
    assert_eq!(row.revoke_on(&mut declined), Revoked::Now);
    assert!(Listing::of(declined.allowed(), noon()).is_nothing_granted());
}

/// **Every sentence the list shows is in the vocabulary `alo-saying`
/// collects** — the row for every kind of reach an agent or an application can
/// be granted, the empty list, and both answers to a revocation — so a machine
/// showing an application's row never shows a key in guillemets.
#[test]
fn every_sentence_the_one_list_shows_is_collected() {
    let strings = what_the_machine_can_say();
    let vocabulary = alo_saying::everything_this_machine_can_say().unwrap();
    for word in alo_granted::words::EVERY_WORD {
        assert!(
            vocabulary.phrase(&word.key()).is_some(),
            "{} is not collected",
            word.named()
        );
    }

    let mut made = Grants::default();
    made.grant(to_agent(FILES, Reach::Folder(PathBuf::from(SHARED))));
    made.grant(to_agent(
        FILES,
        Reach::File(PathBuf::from("/home/anna/Shared/march.pdf")),
    ));
    made.grant(to_agent(
        FILES,
        Reach::Application("org.gnome.Papers".to_owned()),
    ));
    made.grant(to_application(PAPERS, Reach::Folder(PathBuf::from(SHARED))));
    for facility in Facility::EVERY {
        made.grant(to_application(CHEESE, Reach::Facility(facility)));
    }
    let mut grants = kept(&made);
    let listing = Listing::of(&grants, noon());
    assert_eq!(listing.rows().len(), 4 + Facility::EVERY.len());
    for row in listing.rows() {
        let said = row.said(&strings);
        assert!(!said.is_a_bug(), "{said}");
        assert!(said.text().contains(" has been granted "), "{said}");
    }

    let row = listing.rows().first().unwrap();
    for revoked in [row.revoke(&mut grants), row.revoke(&mut grants)] {
        let said = revoked.said(&strings);
        assert!(!said.is_a_bug(), "{said}");
    }

    let nothing = Listing::of(&Grants::default(), noon())
        .said(&strings)
        .unwrap();
    assert!(!nothing.is_a_bug(), "{nothing}");
}
