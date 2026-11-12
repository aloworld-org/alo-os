//! `OpenURI` on a real bus: a link opens in the person's default application
//! with **no grant asked**, every link is recorded, and every refusal is too.
//!
//! The owner's decisions of 2026-11-12, quoted as chosen: *"No — open in my
//! default, but record it (Recommended)"* and *"Yes, add it (Recommended)"* —
//! the portal asks the desktop to start the application as a press does. The
//! desktop is a stand-in here ([`ADesktop`]) that keeps what it was asked and
//! answers as the test says; the wire between the two is held by
//! `alo-portald`'s own tests and `alo-shell`'s door tests.
//!
//! What this cannot show is a real Firefox opening a real page on a booted
//! machine; that is a walk, and it is named as owed rather than implied.

#![cfg(target_os = "linux")]
#![expect(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::panic,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::SystemTime;

use alo_applications::{Application, Chosen, Declared, Installed, Link, LinkKind};
use alo_capability::{Applicant, Grants};
use alo_keyring_fixture::AKeyringOfOurOwn;
use alo_portals::{
    Allowed, Answered, Appearance, Applications, Backend, KeepsSecrets, Kept, KeptOutcome,
    LinkChosen, LinkStarted, NotChosen, NotKept, OpensLinks, Outcome, Request, Sandboxes, Served,
    TheMachine, TimeOfDay, Unanswered,
};
use zbus::blocking::{Connection, Proxy};
use zbus::zvariant::{OwnedObjectPath, OwnedValue, Value};

const MAIL: &str = "org.gnome.Geary";
const FIREFOX: &str = "org.mozilla.firefox";
const LIBREWOLF: &str = "io.gitlab.librewolf-community";
const THUNDERBIRD: &str = "org.mozilla.Thunderbird";

const DESKTOP: &str = "org.freedesktop.portal.Desktop";
const PORTALS: &str = "/org/freedesktop/portal/desktop";
const OPEN_URI: &str = "org.freedesktop.portal.OpenURI";

/// A machine with Firefox and Librewolf declaring web links, and — where
/// `with_mail` — Thunderbird declaring `mailto:`. **Nothing is ever granted.**
struct AMachine {
    with_mail: bool,
    chosen: Chosen,
}

impl TheMachine for AMachine {
    fn grants(&self) -> Option<Grants> {
        Some(Grants::default())
    }

    fn applications(&self) -> Option<Applications> {
        let firefox = Application::identified(FIREFOX).unwrap();
        let librewolf = Application::identified(LIBREWOLF).unwrap();
        let thunderbird = Application::identified(THUNDERBIRD).unwrap();
        let mut declared = Declared::nothing();
        declared.declares_media_types(
            &firefox,
            ["x-scheme-handler/http", "x-scheme-handler/https"],
        );
        declared.declares_media_types(&librewolf, ["x-scheme-handler/https"]);
        let mut installed = vec![firefox, librewolf, Application::identified(MAIL).unwrap()];
        if self.with_mail {
            declared.declares_media_types(&thunderbird, ["x-scheme-handler/mailto"]);
            installed.push(thunderbird);
        }
        Some(Applications {
            installed: Installed::holding(installed),
            declared,
            chosen: self.chosen.clone(),
        })
    }

    fn appearance(&self) -> Option<Appearance> {
        Some(Appearance::shipped())
    }

    fn time_of_day(&self) -> Option<TimeOfDay> {
        TimeOfDay::checked(12, 0).ok()
    }

    fn reaching(&self) -> Option<alo_portals::Reaching> {
        None
    }
}

/// No keyring: nothing here asks the Secret portal.
struct NoKeyring;

impl KeepsSecrets for NoKeyring {
    fn hand_over(
        &self,
        _request: &Request,
        _grants: &Grants,
        _now: SystemTime,
        _into: &mut dyn std::io::Write,
    ) -> Result<Allowed, NotKept> {
        Err(NotKept::Unavailable)
    }
}

/// A desktop that keeps every application and link it is asked to start, and
/// answers `answering` — and every sheet it is asked to show, which the person
/// answers `the_person`.
struct ADesktop {
    asked: Mutex<Vec<(String, String)>>,
    answering: LinkStarted,
    shown: Mutex<Vec<(String, String, Vec<String>)>>,
    the_person: Mutex<LinkChosen>,
}

impl OpensLinks for ADesktop {
    fn opened(&self, application: &str, link: &Link) -> LinkStarted {
        self.asked
            .lock()
            .unwrap()
            .push((application.to_owned(), link.as_str().to_owned()));
        self.answering
    }

    fn chosen(&self, asking: &str, link: &Link, offered: &[String]) -> LinkChosen {
        self.shown.lock().unwrap().push((
            asking.to_owned(),
            link.as_str().to_owned(),
            offered.to_vec(),
        ));
        self.the_person.lock().unwrap().clone()
    }
}

/// One backend on a bus of its own, and a client to ask it from.
struct Serving {
    _bus: AKeyringOfOurOwn,
    record: Arc<Kept>,
    desktop: Option<Arc<ADesktop>>,
    processes: PathBuf,
    client: Connection,
    _served: Served,
}

impl Serving {
    /// `desktop` is how the desktop answers, or [`None`] for a backend never
    /// told how to reach one.
    fn started(what: &str, machine: AMachine, desktop: Option<LinkStarted>) -> Self {
        let bus = AKeyringOfOurOwn::a_bus_with_no_keyring_on_it(what);
        let place =
            std::env::temp_dir().join(format!("alo-portal-links-{what}-{}", std::process::id()));
        let processes = place.join("proc");
        std::fs::create_dir_all(processes.join(std::process::id().to_string()).join("root"))
            .unwrap();
        let record = Arc::new(Kept::nothing());
        let backend = Backend::answering_from(
            Arc::new(machine),
            Arc::new(NoKeyring),
            Sandboxes::under(&processes),
            record.clone(),
        );
        let desktop = desktop.map(|answering| {
            Arc::new(ADesktop {
                asked: Mutex::new(Vec::new()),
                answering,
                shown: Mutex::new(Vec::new()),
                the_person: Mutex::new(LinkChosen::Cancelled),
            })
        });
        let backend = match &desktop {
            Some(desktop) => backend.opening_links_with(desktop.clone()),
            None => backend,
        };
        let served = backend
            .serve_on(&bus.address())
            .expect("the portals are served on the test's bus");
        let client = zbus::blocking::connection::Builder::address(bus.address().as_str())
            .unwrap()
            .build()
            .unwrap();
        let serving = Self {
            _bus: bus,
            record,
            desktop,
            processes,
            client,
            _served: served,
        };
        serving.become_(Some(MAIL));
        serving
    }

    /// From now on this test process is `application`'s sandbox, or for
    /// [`None`] a program with none.
    fn become_(&self, application: Option<&str>) {
        let info = self
            .processes
            .join(std::process::id().to_string())
            .join("root")
            .join(".flatpak-info");
        match application {
            Some(name) => std::fs::write(info, format!("[Application]\nname={name}\n")).unwrap(),
            None => {
                let _ = std::fs::remove_file(info);
            }
        }
    }

    /// `OpenURI` for `uri`, listening on the predicted handle first; the
    /// response code.
    fn open_uri(&self, uri: &str, token: &str, ask: bool) -> u32 {
        let sender = self.client.unique_name().unwrap().as_str();
        let handle = format!(
            "{PORTALS}/request/{}/{token}",
            sender.trim_start_matches(':').replace('.', "_")
        );
        let request = Proxy::new(
            &self.client,
            DESKTOP,
            handle.as_str(),
            "org.freedesktop.portal.Request",
        )
        .unwrap();
        let mut responses = request.receive_signal("Response").unwrap();
        let mut options: HashMap<&str, Value<'_>> = HashMap::new();
        options.insert("handle_token", Value::from(token));
        if ask {
            options.insert("ask", Value::from(true));
        }
        let reply = self
            .client
            .call_method(
                Some(DESKTOP),
                PORTALS,
                Some(OPEN_URI),
                "OpenURI",
                &("", uri, options),
            )
            .expect("OpenURI answers the call");
        let returned: OwnedObjectPath = reply.body().deserialize().unwrap();
        assert_eq!(returned.as_str(), handle);
        let response = responses.next().expect("a response");
        let (code, results): (u32, HashMap<String, OwnedValue>) =
            response.body().deserialize().unwrap();
        assert!(results.is_empty(), "results named something: {results:?}");
        code
    }

    fn everything(&self) -> Vec<Answered> {
        self.record.everything()
    }

    fn asked_of_the_desktop(&self) -> Vec<(String, String)> {
        self.desktop
            .as_ref()
            .map(|desktop| desktop.asked.lock().unwrap().clone())
            .unwrap_or_default()
    }
}

impl Drop for Serving {
    fn drop(&mut self) {
        if let Some(place) = self.processes.parent() {
            let _ = std::fs::remove_dir_all(place);
        }
    }
}

fn untouched(with_mail: bool) -> AMachine {
    AMachine {
        with_mail,
        chosen: Chosen::untouched(),
    }
}

/// **A web link opens in the first browser declaring it, with nothing granted
/// to anybody**: the desktop is asked once, the answer is `0`, and the record
/// names who asked, the address and what opened it.
#[test]
fn a_web_link_opens_in_the_default_with_nothing_granted_and_is_recorded() {
    let serving = Serving::started("web", untouched(true), Some(LinkStarted::Asked));
    assert_eq!(
        serving.open_uri("https://example.com/invoice?n=7", "web", false),
        0
    );
    assert_eq!(
        serving.asked_of_the_desktop(),
        [(
            FIREFOX.to_owned(),
            "https://example.com/invoice?n=7".to_owned()
        )]
    );
    let everything = serving.everything();
    assert_eq!(everything.len(), 1, "one link, one line");
    let answered = everything.first().unwrap();
    assert_eq!(answered.application(), Some(&Applicant::named(MAIL)));
    assert_eq!(
        KeptOutcome::from(answered.outcome()),
        KeptOutcome::LinkOpened {
            opener: FIREFOX.to_owned(),
            link: "web".to_owned(),
            address: "https://example.com/invoice?n=7".to_owned(),
            chosen: false,
            asked_to_choose: false,
            chosen_on_the_sheet: false,
        }
    );
}

/// **The person's chosen default wins over every declaration.**
#[test]
fn the_persons_chosen_browser_is_the_one_asked() {
    let mut chosen = Chosen::untouched();
    chosen.choose_for_links(LinkKind::Web, &Application::identified(LIBREWOLF).unwrap());
    let serving = Serving::started(
        "chosen",
        AMachine {
            with_mail: true,
            chosen,
        },
        Some(LinkStarted::Asked),
    );
    assert_eq!(serving.open_uri("http://example.com/", "chosen", false), 0);
    assert_eq!(serving.asked_of_the_desktop().first().unwrap().0, LIBREWOLF);
    match KeptOutcome::from(serving.everything().first().unwrap().outcome()) {
        KeptOutcome::LinkOpened { chosen, .. } => assert!(chosen),
        other => panic!("{other:?}"),
    }
}

/// **An email address opens in the mail application, and with none installed
/// it opens in nothing** — refused naming the kind, and the desktop is never
/// asked.
#[test]
fn mailto_opens_in_the_mail_application_or_is_refused_naming_the_kind() {
    let with = Serving::started("mail", untouched(true), Some(LinkStarted::Asked));
    assert_eq!(
        with.open_uri("mailto:anna@example.com?subject=Hi", "mail", false),
        0
    );
    assert_eq!(with.asked_of_the_desktop().first().unwrap().0, THUNDERBIRD);

    let without = Serving::started("no-mail", untouched(false), Some(LinkStarted::Asked));
    assert_eq!(
        without.open_uri("mailto:anna@example.com", "no_mail", false),
        2
    );
    assert!(without.asked_of_the_desktop().is_empty());
    match without.everything().first().unwrap().outcome() {
        Outcome::NothingOpensLinks(nothing) => assert_eq!(nothing.kind(), LinkKind::Mail),
        other => panic!("{other:?}"),
    }
}

/// **With no desktop to ask, the link is refused and the record says so** —
/// the answer it was handed to, then why it did not open — and the same for a
/// desktop that could not start it.
#[test]
fn with_no_desktop_the_link_is_refused_and_the_record_says_why() {
    let serving = Serving::started("no-desktop", untouched(true), None);
    assert_eq!(serving.open_uri("https://example.com/", "nobody", false), 2);
    let everything = serving.everything();
    assert_eq!(everything.len(), 2);
    assert!(matches!(
        everything.first().unwrap().outcome(),
        Outcome::LinkOpened(_)
    ));
    assert_eq!(
        everything.get(1).unwrap().outcome(),
        &Outcome::Unanswered(Unanswered::NoDesktop {
            opener: FIREFOX.to_owned()
        })
    );

    let refusing = Serving::started("could-not", untouched(true), Some(LinkStarted::CouldNotAsk));
    assert_eq!(
        refusing.open_uri("https://example.com/", "could_not", false),
        2
    );
    assert_eq!(
        refusing.everything().get(1).unwrap().outcome(),
        &Outcome::Unanswered(Unanswered::NotOpenedALink {
            opener: FIREFOX.to_owned()
        })
    );
}

impl Serving {
    /// From now on the person answers the sheet with `chose`.
    fn the_person_answers(&self, chose: LinkChosen) {
        if let Some(desktop) = &self.desktop {
            *desktop.the_person.lock().unwrap() = chose;
        }
    }

    /// Every sheet the desktop was asked to show.
    fn sheets_shown(&self) -> Vec<(String, String, Vec<String>)> {
        self.desktop
            .as_ref()
            .map(|desktop| desktop.shown.lock().unwrap().clone())
            .unwrap_or_default()
    }
}

/// **`ask` shows the sheet with the default first and every other handler
/// after it; what the person chooses is recorded, then started** — and the
/// default is not what opens.
#[test]
fn asking_shows_the_sheet_and_opens_what_the_person_chose() {
    let serving = Serving::started("ask", untouched(true), Some(LinkStarted::Asked));
    serving.the_person_answers(LinkChosen::Chosen(LIBREWOLF.to_owned()));
    assert_eq!(serving.open_uri("https://example.com/", "ask", true), 0);
    assert_eq!(
        serving.sheets_shown(),
        [(
            MAIL.to_owned(),
            "https://example.com/".to_owned(),
            vec![FIREFOX.to_owned(), LIBREWOLF.to_owned()],
        )]
    );
    assert_eq!(
        serving.asked_of_the_desktop(),
        [(LIBREWOLF.to_owned(), "https://example.com/".to_owned())]
    );
    let everything = serving.everything();
    assert_eq!(everything.len(), 1);
    assert_eq!(
        KeptOutcome::from(everything.first().unwrap().outcome()),
        KeptOutcome::LinkOpened {
            opener: LIBREWOLF.to_owned(),
            link: "web".to_owned(),
            address: "https://example.com/".to_owned(),
            chosen: false,
            asked_to_choose: false,
            chosen_on_the_sheet: true,
        }
    );
}

/// **Cancel is response `1` and opens nothing; nobody answering, no sheet,
/// no desktop and an answer that was not offered are `2` and open nothing** —
/// each recorded with its reason, and never the default instead.
#[test]
fn nothing_chosen_opens_nothing_and_says_why() {
    for (what, chose, response, why) in [
        ("cancel", LinkChosen::Cancelled, 1, NotChosen::Cancelled),
        ("nobody", LinkChosen::Unanswered, 2, NotChosen::Unanswered),
        ("no-sheet", LinkChosen::CannotShow, 2, NotChosen::CannotShow),
        ("gone", LinkChosen::NoDesktop, 2, NotChosen::NoDesktop),
        ("broken", LinkChosen::CouldNotAsk, 2, NotChosen::CouldNotAsk),
        (
            "not-offered",
            LinkChosen::Chosen(MAIL.to_owned()),
            2,
            NotChosen::CouldNotAsk,
        ),
    ] {
        let serving = Serving::started(what, untouched(true), Some(LinkStarted::Asked));
        serving.the_person_answers(chose);
        assert_eq!(
            serving.open_uri("https://example.com/", "sheet", true),
            response,
            "{what}"
        );
        assert!(
            serving.asked_of_the_desktop().is_empty(),
            "{what} opened something"
        );
        let everything = serving.everything();
        assert_eq!(everything.len(), 1, "{what}");
        assert_eq!(
            KeptOutcome::from(everything.first().unwrap().outcome()),
            KeptOutcome::LinkNotChosen {
                link: "web".to_owned(),
                address: "https://example.com/".to_owned(),
                why,
            },
            "{what}"
        );
    }
}

/// **With no desktop at all, a sheet is never shown and nothing opens.**
#[test]
fn asking_with_no_desktop_opens_nothing() {
    let serving = Serving::started("ask-nobody", untouched(true), None);
    assert_eq!(serving.open_uri("https://example.com/", "nobody", true), 2);
    match KeptOutcome::from(serving.everything().first().unwrap().outcome()) {
        KeptOutcome::LinkNotChosen { why, .. } => assert_eq!(why, NotChosen::NoDesktop),
        other => panic!("{other:?}"),
    }
}

/// **What is not a link, and a caller with no sandbox, are refused, recorded,
/// and never reach the desktop.**
#[test]
fn what_is_not_a_link_and_an_unnamed_caller_are_refused_and_recorded() {
    let serving = Serving::started("refused", untouched(true), Some(LinkStarted::Asked));
    for (uri, token) in [
        ("file:///etc/shadow", "a_file"),
        ("javascript:alert(1)", "code"),
        ("https://anna:hunter2@example.com/", "password"),
    ] {
        assert_eq!(serving.open_uri(uri, token, false), 2, "{uri}");
        assert!(
            matches!(
                serving.everything().last().unwrap().outcome(),
                Outcome::NotALink(_)
            ),
            "{uri}"
        );
    }
    serving.become_(None);
    assert_eq!(
        serving.open_uri("https://example.com/", "unnamed", false),
        2
    );
    let last = serving.everything().pop().unwrap();
    assert_eq!(last.application(), None);
    assert_eq!(
        last.outcome(),
        &Outcome::Unanswered(Unanswered::NotIdentified)
    );
    assert!(serving.asked_of_the_desktop().is_empty());
}
