//! Task 8's acceptance, from `docs/autonomy/putting-a-window-aside.md`.
//!
//! > A private window shows a neutral *Preview hidden* surface with a safe identifying name,
//! > and still restores normally. Previews are made locally.
//! >
//! > **Acceptance.** A minimised state is never permission for an agent to inspect, share or
//! > act on the window — **tested as a refusal, not assumed from the absence of a call.**
//!
//! # That last clause decided the shape of the whole task
//!
//! *Assumed from the absence of a call* is the test this forbids: assert that nothing reached
//! the window, and it passes on a machine where the agent is simply not running — then goes on
//! passing after somebody adds the call, because absence of evidence is what it was measuring.
//! ADR 0080's family, and the reason this repository writes refusals down.
//!
//! So there has to be a **door**, it has to be **asked**, and it has to **say no**. That is
//! `what_an_agent_may_do`, whose return type has no success case: every call is a refusal and
//! the type says which kind. The tests below ask it nine times — three things an agent might
//! want, two reasons it might give — and check that each is declined by name.
#![expect(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "in a test, a panic on an unexpected Err is the failure being reported — the same \
              exemption `alo-dock`'s own test modules take, with its words. `clippy::panic` is \
              for the let-else arm, which names which variant arrived"
)]

use std::path::{Path, PathBuf};

use alo_canvas::Zoom;
use alo_dock::on_the_canvas::{Patch, Spot, TheView};
use alo_dock::window::{AppId, HowItSits, Window, WindowId};
use alo_dock::windows::Windows;
use alo_put_aside::a_safe_name::{NotSafe, SafeName};
use alo_put_aside::putting_aside::put_aside;
use alo_put_aside::restoring_into_a_taken_place::{Restored, ask_for};
use alo_put_aside::what_a_preview_is_headed_with::Headline;
use alo_put_aside::what_an_agent_may_do::{
    Doing, NotPermitted, TheReasonGiven, what_an_agent_may_do,
};
use alo_put_aside::{Panel, Privacy};

fn a_zoom() -> Zoom {
    Zoom::of(1_750).expect("1_750 thousandths is inside the canvas's own bounds")
}

/// A window whose title would be a leak if it were ever drawn.
fn window(id: u64, app: &str, called: &str, x: i64) -> Window {
    Window::of(
        WindowId::numbered(id),
        AppId::named(app).expect("a fixture names its application"),
        called,
        Patch::of(Spot::at(x, 0), 800, 600).expect("a fixture gives its patch an extent"),
        HowItSits::OnTheCanvas,
    )
}

/// The title used throughout: the kind of thing a person puts a window away to stop seeing.
const A_TITLE_THAT_LEAKS: &str = "Re: settlement figure — do not forward";

fn a_desk() -> Windows {
    let mut windows = Windows::none();
    windows.opened(window(1, "Mail", A_TITLE_THAT_LEAKS, 4_200));
    windows.opened(window(2, "Docs", "Launch strategy", 9_000));
    windows
}

fn private_as(name: &str) -> Privacy {
    Privacy::Private(SafeName::chosen(name).expect("a fixture names something safe"))
}

fn a_view() -> TheView {
    TheView::showing(Patch::of(Spot::at(4_000, -200), 3_000, 1_080).expect("a view has extent"))
}

/// **A private window's title is not reachable**, so a surface cannot draw it by mistake.
///
/// The strongest form of the clause, and the reason `Preview::called()` was removed: an
/// accessor returning the title is the whole of what a leak needs, and documentation beside it
/// stops nobody. There is now one way to get a heading and it is already correct.
#[test]
fn a_private_preview_is_headed_with_a_safe_name_and_never_the_title() {
    let mut windows = a_desk();
    let mut panel = Panel::new();
    put_aside(
        &mut windows,
        &mut panel,
        WindowId::numbered(1),
        a_zoom(),
        private_as("A Mail window"),
    )
    .unwrap();

    let preview = panel.previews().first().expect("one window is put aside");
    assert!(preview.privacy().is_private());
    assert!(
        preview.headline().is_hidden(),
        "a private window's row is not marked as a hidden one"
    );
    assert_eq!(preview.headline().text(), "A Mail window");
    assert_ne!(
        preview.headline().text(),
        A_TITLE_THAT_LEAKS,
        "the private title reached the heading"
    );
    assert!(
        matches!(preview.headline(), Headline::PreviewHidden(_)),
        "a private window produced an ordinary title heading"
    );
}

/// **An ordinary window is still headed with its own title**, so privacy is not a tax on
/// everybody.
#[test]
fn an_ordinary_preview_is_headed_with_its_own_title() {
    let mut windows = a_desk();
    let mut panel = Panel::new();
    put_aside(
        &mut windows,
        &mut panel,
        WindowId::numbered(2),
        a_zoom(),
        Privacy::Ordinary,
    )
    .unwrap();

    let preview = panel.previews().first().expect("one window is put aside");
    assert!(!preview.privacy().is_private());
    assert!(!preview.headline().is_hidden());
    assert_eq!(preview.headline().text(), "Launch strategy");
    assert_eq!(preview.headline(), Headline::ItsTitle("Launch strategy"));
}

/// **A private window still restores normally**, which is the owner's own clause.
///
/// Privacy changes what is *shown*, never what works. A panel that could not give a private
/// window back would be one a person could not use for private work — and private work is
/// exactly the kind somebody puts aside.
#[test]
fn a_private_window_restores_like_any_other() {
    let mut windows = a_desk();
    let mut panel = Panel::new();
    let was_at = windows.window(WindowId::numbered(1)).unwrap().at();
    put_aside(
        &mut windows,
        &mut panel,
        WindowId::numbered(1),
        a_zoom(),
        private_as("A Mail window"),
    )
    .unwrap();

    let what = ask_for(
        &mut windows,
        &mut panel,
        WindowId::numbered(1),
        a_view(),
        None,
        Patch::of(Spot::at(1, 1), 10, 10).unwrap(),
    )
    .unwrap();

    let Restored::Travelled(travel) = what else {
        panic!("a private window proposed instead of restoring");
    };
    assert!(
        !travel.is_needed(),
        "the view already shows it, so no travel was needed"
    );
    assert_eq!(
        windows.window(WindowId::numbered(1)).unwrap().sits(),
        HowItSits::OnTheCanvas
    );
    assert_eq!(
        windows.window(WindowId::numbered(1)).unwrap().at(),
        was_at,
        "a private window came back somewhere else"
    );
    assert_eq!(panel.holding(), 0);
}

/// **Private, with no safe name, does not exist**, so a surface can never fall back to the
/// title.
///
/// A `bool` beside an `Option<SafeName>` has a fourth state — private and unnamed — and the
/// only thing a surface could do with it is show the title. Here that state is unrepresentable,
/// and a blank name is refused so three hidden rows can still be told apart.
#[test]
fn a_private_window_cannot_exist_without_something_safe_to_call_it() {
    assert_eq!(
        SafeName::chosen("   "),
        Err(NotSafe::ItSaysNothing),
        "three rows reading `Preview hidden` identify nothing"
    );
    assert_eq!(SafeName::chosen(""), Err(NotSafe::ItSaysNothing));
    assert!(SafeName::chosen("A Mail window").is_ok());

    // And the only private case there is carries one.
    let private = private_as("A Mail window");
    assert!(private.safe_name().is_some());
    assert_eq!(Privacy::Ordinary.safe_name(), None);
}

/// **Being put aside is refused as a reason, for all three things, by name.**
///
/// The acceptance's own clause, tested as a refusal. Nine calls in this file ask the door and
/// every one is declined; these three are the ones that matter most, because *nobody is
/// looking at it* is the argument a well-meaning caller actually reaches for.
#[test]
fn being_put_aside_is_never_permission_to_inspect_share_or_act() {
    for doing in [Doing::Inspecting, Doing::Sharing, Doing::ActingOn] {
        let said = what_an_agent_may_do(doing, &TheReasonGiven::ItIsPutAside);
        assert_eq!(
            said,
            NotPermitted::BeingPutAsideIsNotPermission(doing),
            "being put aside was accepted as permission to {doing:?}"
        );
        // And the refusal says why in words a person could read.
        let why = said.to_string();
        assert!(
            why.contains("rather than handing it over"),
            "the refusal does not say why, only that: {why}"
        );
    }
}

/// **A grant is refused here too, and by a different name.**
///
/// Not because it is a bad reason — because this is not where grants are checked. A crate that
/// said yes to a grant it never saw would be guessing, and a second place that decided
/// permission would be a second answer to *may this happen*, with an agent finding whichever
/// is weaker.
#[test]
fn a_grant_is_not_checked_here_and_the_refusal_says_where_it_is() {
    let reason = TheReasonGiven::ThePersonGranted {
        named: "Read this window's messages".to_owned(),
    };
    for doing in [Doing::Inspecting, Doing::Sharing, Doing::ActingOn] {
        let said = what_an_agent_may_do(doing, &reason);
        assert_eq!(
            said,
            NotPermitted::GrantsAreNotCheckedHere(doing, "Read this window's messages".to_owned()),
            "a grant this crate never saw was accepted"
        );
        let why = said.to_string();
        assert!(
            why.contains("visible, revocable and expires"),
            "the refusal does not point at where grants live: {why}"
        );
    }
}

/// **The two refusals are different**, so a caller can tell *not on those grounds* from *not
/// here*.
///
/// One of them means *go and get a grant*; the other means *ask the surface that holds them*. A
/// caller that could not tell them apart would report the wrong thing, and this repository
/// keeps finding that fault — *nothing is put aside* against *that window is not*, and a
/// window that does not exist against one the panel does not hold.
#[test]
fn the_two_refusals_are_not_the_same_refusal() {
    let on_being_aside = what_an_agent_may_do(Doing::Inspecting, &TheReasonGiven::ItIsPutAside);
    let on_a_grant = what_an_agent_may_do(
        Doing::Inspecting,
        &TheReasonGiven::ThePersonGranted {
            named: "Something".to_owned(),
        },
    );
    assert_ne!(on_being_aside, on_a_grant);
    assert_ne!(on_being_aside.to_string(), on_a_grant.to_string());
}

/// **Previews are made locally**, held by this crate having no road off the machine.
///
/// The owner's clause. A preview is a title, an application and a rectangle, and a crate that
/// could send one is a crate that could send what a person put away — so the check is that no
/// road exists, read off this crate's own source. The list is `alo-adapting`'s, which forbids
/// itself a network for the same reason in different words.
#[test]
fn nothing_here_can_send_a_preview_anywhere() {
    const A_ROAD_OFF_THIS_MACHINE: [&str; 9] = [
        "std::net",
        "TcpStream",
        "UdpSocket",
        "ureq",
        "reqwest",
        "hyper",
        "socket2",
        "alo_egress",
        "alo_asking",
    ];

    let src = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut roads: Vec<String> = Vec::new();
    for file in every_file(&src) {
        let text = std::fs::read_to_string(&file).expect("this crate's own source");
        for line in text.lines() {
            let code = only_the_code(line);
            for road in A_ROAD_OFF_THIS_MACHINE {
                if code.contains(road) {
                    roads.push(format!("{}: {}", file.display(), line.trim()));
                }
            }
        }
    }
    assert!(
        roads.is_empty(),
        "this crate has grown a road off the machine, and what it holds is what a person put \
         away: titles, applications, and which windows somebody wanted out of sight.\n\n\
         Previews are made locally. A preview that could travel is a private window \
         travelling.\n\n\
         Found:\n{roads:#?}"
    );

    // And the manifest takes nothing that could carry one.
    let manifest =
        std::fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml"))
            .expect("this crate's own manifest");
    let dependencies = manifest
        .split("[dependencies]")
        .nth(1)
        .unwrap_or_default()
        .split("\n[")
        .next()
        .unwrap_or_default();
    for reaching in [
        "ureq",
        "reqwest",
        "hyper",
        "socket2",
        "alo-egress",
        "alo-asking",
    ] {
        assert!(
            !dependencies.contains(reaching),
            "alo-put-aside depends on {reaching}, which can reach off this machine"
        );
    }
}

/// A line with its documentation and string literals removed, so that prose naming a road in
/// order to forbid it is not read as taking one.
fn only_the_code(line: &str) -> String {
    let trimmed = line.trim_start();
    if trimmed.starts_with("//") {
        return String::new();
    }
    let mut code = String::new();
    let mut inside_a_string = false;
    let mut letters = line.chars().peekable();
    while let Some(letter) = letters.next() {
        match letter {
            '\\' if inside_a_string => {
                letters.next();
            }
            '"' => inside_a_string = !inside_a_string,
            _ if !inside_a_string => code.push(letter),
            _ => {}
        }
    }
    code
}

/// Every `.rs` under a folder.
fn every_file(folder: &Path) -> Vec<PathBuf> {
    let mut found = Vec::new();
    let Ok(entries) = std::fs::read_dir(folder) else {
        return found;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            found.extend(every_file(&path));
        } else if path.extension().is_some_and(|kind| kind == "rs") {
            found.push(path);
        }
    }
    found
}
