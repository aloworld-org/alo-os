//! A portal request is a grant, and is refused like one.
//!
//! Task 1 of `docs/autonomy/v0-5-applications-and-what-they-expect-plan.md`,
//! each clause of its acceptance a test here:
//!
//! - **the portals `docs/features.md` lists for v0.5 are a closed enum**, each
//!   with a sentence in the vocabulary, and the v1 portals absent from it —
//!   [`the_portals_are_the_closed_list_the_promise_names`];
//! - **a request from a named application is evaluated the way
//!   `alo-capability` evaluates a verb** — against a grant naming the
//!   application and the reach, refused outside it, never widened —
//!   [`a_request_is_judged_against_a_grant_naming_the_application_and_the_reach`];
//! - **and the evaluation reuses that crate's types rather than restating its
//!   rules** — [`the_evaluation_is_alo_capabilitys_and_is_not_restated_here`];
//! - **a request from an application nobody has granted anything is refused
//!   before any dialog is imagined** —
//!   [`an_application_granted_nothing_is_refused_before_any_dialog`];
//! - **what a request is for is the same `Reach` an agent's grant names, so one
//!   list can hold both** —
//!   [`what_a_request_is_for_is_the_reach_an_agents_grant_names`].
//!
//! And the two things ADR 0040 was accepted on that a portal can see: a
//! declined machine still judges applications, and the decision's table is
//! still the list this crate models.
//!
//! It reads files and runs nothing on the machine, so it holds on any host.

#![expect(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::panic,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

use alo_capability::{
    Agent, Applicant, Ask, Facility, Grant, Grantee, Grants, NotAllowed, NotGranted, Reach,
};
use alo_portals::{NotARequest, Over, Portal, Refused, Request};
use alo_strings::{Key, Strings};

/// The moment these tests call noon.
fn noon() -> SystemTime {
    SystemTime::UNIX_EPOCH + Duration::from_secs(1_760_000_000)
}

fn hour() -> Duration {
    Duration::from_secs(60 * 60)
}

const CHEESE: &str = "org.gnome.Cheese";
const PAPERS: &str = "org.gnome.Papers";

fn cheese() -> Applicant {
    Applicant::named(CHEESE)
}

/// A grant to an application, checked the way a person's pick is.
fn granted_to(application: &str, reach: Reach, lasting: Duration) -> Grant {
    Grant::checked_for(
        &Applicant::named(application).grantee(),
        reach,
        noon(),
        lasting,
    )
    .unwrap()
}

/// The machine's one vocabulary, in English.
fn in_english() -> Strings {
    Strings::of(alo_saying::everything_this_machine_can_say().unwrap())
}

/// Where the repository is, from this crate.
fn the_repository() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .canonicalize()
        .expect("the repository is where this crate says it is")
}

/// A file of this repository, read.
fn reading(named: &str) -> String {
    let at = the_repository().join(named);
    fs::read_to_string(&at)
        .unwrap_or_else(|why| panic!("{} could not be read: {why}", at.display()))
}

/// The v0.5 portal line of `docs/features.md`, whole.
///
/// Held whole so that a portal added to or taken from the promise fails here,
/// and sends whoever changed it to [`Portal`] and to ADR 0040's table.
const THE_PORTALS_PROMISED: &str = "- [v0.5] Portals: file chooser and documents, open-with and \
                                    default applications, notifications, print, screenshot, \
                                    screen capture, camera, microphone, clipboard, trash, \
                                    wallpaper, settings, inhibit (no sleep mid-presentation), \
                                    network and power-profile monitors";

/// The v0.5 line that promises the Secret portal, on its own.
const THE_SECRET_PORTAL_PROMISED: &str = "- [v0.5] **Secret storage** — one keyring behind the Secret \
                                          portal, so applications stop inventing credential storage";

/// The v1 portal line, which names what this crate must not.
const THE_LATER_PORTALS: &str = "- [v1] Portals: USB devices, global shortcuts an application \
                                 registers, dynamic launchers, remote desktop";

/// **The portals are the closed list the promise names**, in its order, each
/// with a sentence in the machine's one vocabulary — and the v1 portals absent.
#[test]
fn the_portals_are_the_closed_list_the_promise_names() {
    let features = reading("docs/features.md");
    assert!(
        features.lines().any(|line| line == THE_PORTALS_PROMISED),
        "the v0.5 portal line of docs/features.md is no longer the one Portal was built from"
    );
    assert!(
        features
            .lines()
            .any(|line| line == THE_SECRET_PORTAL_PROMISED),
        "the Secret portal's line of docs/features.md is no longer the one Portal::Secret was \
         built from"
    );
    assert!(
        features.lines().any(|line| line == THE_LATER_PORTALS),
        "the v1 portal line of docs/features.md moved, so what must be absent is unknown"
    );

    // Every portal is named in the promise, in the promise's order. The line
    // says "network and power-profile monitors", so the two monitors are found
    // by their first word.
    let mut from = 0;
    let (secret, the_portal_line) = Portal::EVERY.split_last().expect("there are portals");
    for portal in the_portal_line {
        let name = portal.promised_as();
        let looked_for = name.strip_suffix(" monitor").unwrap_or(name);
        let at = THE_PORTALS_PROMISED[from..]
            .find(looked_for)
            .unwrap_or_else(|| panic!("`{name}` is not in the promise after where it should be"));
        from += at + looked_for.len();
    }
    assert_eq!(the_portal_line.len(), 15);
    // And the sixteenth is the Secret portal, promised on the line after.
    assert_eq!(*secret, Portal::Secret);
    assert!(
        THE_SECRET_PORTAL_PROMISED
            .to_lowercase()
            .contains(secret.promised_as()),
        "`{}` is not what the Secret portal's line promises",
        secret.promised_as()
    );

    // And each has its sentence in the vocabulary the machine collects.
    let strings = in_english();
    for portal in Portal::EVERY {
        let said = portal.said(&strings);
        assert!(!said.is_a_bug(), "{portal:?} has no sentence: {said}");
    }

    // The v1 portals are not variants that answer no; they are not here.
    let listed: Vec<String> = Portal::EVERY
        .iter()
        .map(|portal| format!("{portal:?} {}", portal.promised_as()).to_lowercase())
        .collect();
    for later in ["usb", "shortcut", "launcher", "remote"] {
        assert!(
            !listed.iter().any(|one| one.contains(later)),
            "a v1 portal, `{later}`, is on the list"
        );
    }
}

/// **A request is judged against a grant naming the application and the
/// reach, refused outside it, and never widened.**
#[test]
fn a_request_is_judged_against_a_grant_naming_the_application_and_the_reach() {
    let mut grants = Grants::default();
    let camera = grants.grant(granted_to(
        CHEESE,
        Reach::Facility(Facility::Camera),
        hour(),
    ));
    let invoices = grants.grant(granted_to(
        PAPERS,
        Reach::Folder(PathBuf::from("/home/anna/Invoices")),
        hour(),
    ));
    grants.grant(granted_to(
        PAPERS,
        Reach::Application("org.gnome.Papers".to_owned()),
        hour(),
    ));
    let before = serde_json::to_string(&grants).unwrap();

    // Allowed, by name: the grant a person can find and revoke.
    let allowed = Request::of(CHEESE, Portal::Camera)
        .unwrap()
        .judged(&grants, noon())
        .unwrap();
    assert_eq!(allowed.against(), [camera]);
    assert_eq!(allowed.application(), &cheese());
    assert_eq!(allowed.portal(), Portal::Camera);

    // Outside the reach: every other facility portal is refused, carrying
    // alo-capability's own refusal.
    for portal in Portal::EVERY {
        let Over::Facility(facility) = portal.over() else {
            continue;
        };
        if facility == Facility::Camera {
            continue;
        }
        let refused = Request::of(CHEESE, portal)
            .unwrap()
            .judged(&grants, noon())
            .unwrap_err();
        assert_eq!(
            refused,
            Refused::NotAllowed {
                portal,
                why: NotAllowed::Never {
                    application: cheese(),
                    wanted: Ask::facility(facility),
                },
            },
            "{portal:?}"
        );
    }

    // Outside the application: Papers holds grants, and not Cheese's camera.
    assert!(matches!(
        Request::of(PAPERS, Portal::Camera)
            .unwrap()
            .judged(&grants, noon()),
        Err(Refused::NotAllowed { .. })
    ));

    // A file inside the granted folder is allowed; one beside it is not, and
    // neither is the folder above.
    let inside = Request::over(
        PAPERS,
        Portal::Print,
        Path::new("/home/anna/Invoices/march.pdf"),
    )
    .unwrap()
    .judged(&grants, noon())
    .unwrap();
    assert_eq!(inside.against(), [invoices]);
    for outside in [
        "/home/anna/Taxes/2024.pdf",
        "/home/anna",
        "/home/anna/Invoices2/x.pdf",
    ] {
        assert!(
            matches!(
                Request::over(PAPERS, Portal::Trash, Path::new(outside))
                    .unwrap()
                    .judged(&grants, noon()),
                Err(Refused::NotAllowed { .. })
            ),
            "{outside}"
        );
    }

    // Open-with needs the file and the opener both granted.
    let open = Request::opening_with(
        PAPERS,
        Path::new("/home/anna/Invoices/march.pdf"),
        "org.gnome.Papers",
    )
    .unwrap()
    .judged(&grants, noon())
    .unwrap();
    assert_eq!(open.against().len(), 2);
    let refused = Request::opening_with(
        PAPERS,
        Path::new("/home/anna/Invoices/march.pdf"),
        "org.gnome.Terminal",
    )
    .unwrap()
    .judged(&grants, noon())
    .unwrap_err();
    assert_eq!(
        refused,
        Refused::NotAllowed {
            portal: Portal::OpenWith,
            why: NotAllowed::Never {
                application: Applicant::named(PAPERS),
                wanted: Ask::application("org.gnome.Terminal"),
            },
        }
    );

    // Expired, and Cheese holding nothing else: refused as holding nothing.
    assert!(matches!(
        Request::of(CHEESE, Portal::Camera)
            .unwrap()
            .judged(&grants, noon() + hour()),
        Err(Refused::NothingGranted { .. })
    ));

    // Never widened: every question above left the list as it was.
    assert_eq!(serde_json::to_string(&grants).unwrap(), before);

    // Revoked: refused at the next request.
    assert!(grants.revoke(camera));
    assert!(
        Request::of(CHEESE, Portal::Camera)
            .unwrap()
            .judged(&grants, noon())
            .is_err()
    );

    // And a refusal an application is shown never calls it an agent.
    let said = refused.said(&in_english());
    assert!(!said.is_a_bug(), "{said}");
    assert!(!said.text().contains("agent"), "{said}");
    assert!(said.text().contains("org.gnome.Terminal"), "{said}");
}

/// **An expired grant beside a live one says it expired**, so a person knows
/// to grant it again rather than wonder whether they ever did.
#[test]
fn a_request_over_an_expired_grant_says_it_expired() {
    let mut grants = Grants::default();
    grants.grant(granted_to(
        CHEESE,
        Reach::Facility(Facility::Camera),
        hour(),
    ));
    grants.grant(granted_to(
        CHEESE,
        Reach::Facility(Facility::Microphone),
        hour() * 3,
    ));
    let later = noon() + hour() * 2;
    let refused = Request::of(CHEESE, Portal::Camera)
        .unwrap()
        .judged(&grants, later)
        .unwrap_err();
    assert_eq!(
        refused,
        Refused::NotAllowed {
            portal: Portal::Camera,
            why: NotAllowed::Lapsed {
                application: cheese(),
                reach: Reach::Facility(Facility::Camera),
                wanted: Ask::facility(Facility::Camera),
            },
        }
    );
    assert!(refused.said(&in_english()).text().contains("has expired"));
}

/// **An application nobody has granted anything is refused before any dialog
/// is imagined** — whatever it asks for, whatever agents hold, and whatever it
/// is called.
#[test]
fn an_application_granted_nothing_is_refused_before_any_dialog() {
    let mut grants = Grants::default();
    // An agent holds exactly what the stranger will ask for, under the
    // stranger's own name. It is not the stranger's.
    grants.grant(
        Grant::checked(
            "org.example.Stranger",
            Reach::Folder(PathBuf::from("/home/anna/Invoices")),
            noon(),
            hour(),
        )
        .unwrap(),
    );
    grants.grant(granted_to(
        CHEESE,
        Reach::Facility(Facility::Camera),
        hour(),
    ));

    let stranger = "org.example.Stranger";
    let mut requests: Vec<Request> = Portal::EVERY
        .iter()
        .filter(|portal| matches!(portal.over(), Over::Facility(_)))
        .map(|portal| Request::of(stranger, *portal).unwrap())
        .collect();
    for portal in [
        Portal::FileChooser,
        Portal::OpenWith,
        Portal::Print,
        Portal::Trash,
    ] {
        requests.push(
            Request::over(stranger, portal, Path::new("/home/anna/Invoices/march.pdf")).unwrap(),
        );
    }
    assert_eq!(requests.len(), Portal::EVERY.len());

    for request in &requests {
        assert_eq!(
            request.judged(&grants, noon()).unwrap_err(),
            Refused::NothingGranted {
                application: Applicant::named(stranger),
                portal: request.portal(),
            },
            "{:?}",
            request.portal()
        );
    }

    // The sentence says nothing was put to the person.
    let said = requests
        .first()
        .unwrap()
        .judged(&grants, noon())
        .unwrap_err()
        .said(&in_english());
    assert!(!said.is_a_bug(), "{said}");
    assert!(said.text().contains("without asking you"), "{said}");
    assert!(said.text().contains(stranger), "{said}");

    // A request that is not one never reaches the grants at all.
    assert_eq!(
        Request::of("", Portal::Camera).unwrap_err(),
        NotARequest::NoApplication
    );
    assert_eq!(
        Request::of("org.example.Stranger\n", Portal::Print).unwrap_err(),
        NotARequest::NeedsAPath
    );
    let refusal = NotARequest::NotAnIdentifier.said(&in_english());
    assert!(!refusal.is_a_bug(), "{refusal}");
}

/// **What a request is for is the `Reach` an agent's grant names**, so one list
/// holds an agent's folder and an application's folder side by side, and
/// neither answers for the other.
#[test]
fn what_a_request_is_for_is_the_reach_an_agents_grant_names() {
    let invoices = Reach::Folder(PathBuf::from("/home/anna/Invoices"));
    let march = Path::new("/home/anna/Invoices/march.pdf");

    let mut grants = Grants::default();
    let agents = grants.grant(Grant::checked("@files", invoices.clone(), noon(), hour()).unwrap());
    let papers = grants.grant(granted_to(PAPERS, invoices.clone(), hour()));

    // The same reach, on the same list, under two grantees.
    let reaches: Vec<&Reach> = grants
        .active_at(noon())
        .map(|held| &held.grant.reach)
        .collect();
    assert_eq!(reaches, [&invoices, &invoices]);

    // What the request is for is exactly the ask the agent's verb makes.
    let request = Request::over(PAPERS, Portal::FileChooser, march).unwrap();
    assert_eq!(request.wanted(), [Ask::path(march)]);
    assert!(invoices.covers(request.wanted().first().unwrap()));

    // Each is answered by its own grant, from the one list.
    assert_eq!(
        grants.permitting(&Grantee::named("@files"), &Ask::path(march), noon()),
        Ok(agents)
    );
    assert_eq!(request.judged(&grants, noon()).unwrap().against(), [papers]);

    // Revoking the application's grant is the list's own revocation, and
    // leaves the agent's.
    assert!(grants.revoke(papers));
    assert!(request.judged(&grants, noon()).is_err());
    assert_eq!(
        grants.permitting(&Grantee::named("@files"), &Ask::path(march), noon()),
        Ok(agents)
    );

    // And an agent never borrows the application's reach, or the other way.
    let mut only_papers = Grants::default();
    only_papers.grant(granted_to(PAPERS, invoices, hour()));
    assert!(matches!(
        only_papers.permitting(&Grantee::named(PAPERS), &Ask::path(march), noon()),
        Err(NotGranted::Never { .. })
    ));
}

/// **The evaluation is `alo-capability`'s, not restated here.** A refusal is
/// the very value `Grants::allowing` answers with, and nothing in this crate's
/// shipped source compares paths, identifiers or moments itself.
#[test]
fn the_evaluation_is_alo_capabilitys_and_is_not_restated_here() {
    let mut grants = Grants::default();
    grants.grant(granted_to(
        CHEESE,
        Reach::Facility(Facility::Camera),
        hour(),
    ));
    let request = Request::of(CHEESE, Portal::Microphone).unwrap();
    let Err(Refused::NotAllowed { why, .. }) = request.judged(&grants, noon()) else {
        panic!("the microphone was allowed with only the camera granted");
    };
    assert_eq!(
        Err(why),
        grants.allowing(&cheese(), &Ask::facility(Facility::Microphone), noon())
    );

    // What deciding looks like, which this crate must leave to the crate that
    // decides.
    let deciding = [
        ".covers(",
        "is_inside",
        "is_exactly",
        ".expires",
        "is_active_at",
        "expires_in",
        "is_for(",
        "grants_mut",
        ".grant(",
        ".revoke(",
    ];
    let source = the_repository().join("crates/alo-portals/src");
    let mut read = 0;
    for entry in fs::read_dir(&source).unwrap() {
        let path = entry.unwrap().path();
        let text = fs::read_to_string(&path).unwrap();
        // Doc comments and tests may name them; the code may not.
        let code: String = text
            .split("#[cfg(test)]")
            .next()
            .unwrap_or_default()
            .lines()
            .filter(|line| !line.trim_start().starts_with("//"))
            .collect::<Vec<_>>()
            .join("\n");
        for word in deciding {
            assert!(
                !code.contains(word),
                "{} decides for itself with `{word}`",
                path.display()
            );
        }
        read += 1;
    }
    assert!(read >= 6, "the crate's source was not found");
}

/// **A machine whose person declined the agent still judges applications**
/// (ADR 0040, part 3): the camera granted to a video-call application survives
/// declining, and a request is judged against it.
#[test]
fn a_machine_with_no_agent_still_judges_what_applications_ask() {
    let mut machine = Agent::present();
    machine.grants_mut().unwrap().grant(
        Grant::checked(
            "@files",
            Reach::Folder(PathBuf::from("/home/anna/Invoices")),
            noon(),
            hour(),
        )
        .unwrap(),
    );
    let camera = machine
        .allow(granted_to(
            CHEESE,
            Reach::Facility(Facility::Camera),
            hour(),
        ))
        .unwrap();
    assert_eq!(machine.declining(noon()), 1);

    let request = Request::of(CHEESE, Portal::Camera).unwrap();
    assert_eq!(
        request.judged(machine.allowed(), noon()).unwrap().against(),
        [camera]
    );
    assert!(
        Request::of(CHEESE, Portal::ScreenCapture)
            .unwrap()
            .judged(machine.allowed(), noon())
            .is_err()
    );

    // And on that machine an agent's grant still cannot be made.
    assert!(machine.grants_mut().is_none());
    assert_eq!(
        machine.allow(
            Grant::checked(
                "@files",
                Reach::Folder(PathBuf::from("/home/anna/Invoices")),
                noon(),
                hour()
            )
            .unwrap()
        ),
        None
    );
}

/// **The decision this was built from still reads every portal this crate
/// models**: a row per portal in ADR 0040's table, and the rows no path could
/// name are exactly the portals over a facility.
#[test]
fn the_decision_this_was_built_from_still_reads_every_portal() {
    let decision = reading("docs/decisions/0040-what-an-applications-grant-is-over.md");
    let status = decision
        .lines()
        .find(|line| line.starts_with("**Status:**"))
        .expect("the decision has a status line");
    assert!(
        status.contains("accepted"),
        "ADR 0040 is not accepted, and this crate is built on it: `{status}`"
    );
    for portal in Portal::EVERY {
        let begins = format!("| {} |", portal.promised_as());
        let rows: Vec<&str> = decision
            .lines()
            .map(str::trim_start)
            .filter(|line| line.starts_with(&begins))
            .collect();
        let [row] = rows.as_slice() else {
            panic!(
                "ADR 0040's table has {} rows for `{}`",
                rows.len(),
                portal.promised_as()
            );
        };
        assert_eq!(
            row.contains("**none**"),
            matches!(portal.over(), Over::Facility(_)),
            "ADR 0040 and Portal::over disagree about `{}`",
            portal.promised_as()
        );
    }
}

/// **Every sentence this crate says is collected** into the machine's one
/// vocabulary, which is where a translator finds them.
#[test]
fn every_sentence_is_in_the_machines_one_vocabulary() {
    let vocabulary = alo_saying::everything_this_machine_can_say().unwrap();
    for word in alo_portals::EVERY_WORD {
        let key = Key::named(word.named()).unwrap();
        assert!(
            vocabulary.phrase(&key).is_some(),
            "{} is not collected",
            word.named()
        );
    }
}
