//! Task 1's acceptance, from `docs/autonomy/putting-a-window-aside.md`.
//!
//! Three things have to be true before the panel is worth building on: a preview is one
//! window rather than one application, collapsing touches nothing but the panel, and an
//! empty panel is its own state rather than a list of length zero.
#![expect(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected Err is the failure being reported — \
              the same exemption `alo-dock`'s own test modules take, with its words"
)]

use alo_canvas::Zoom;
use alo_dock::on_the_canvas::{Patch, Spot};
use alo_dock::window::{AppId, HowItSits, Window, WindowId};
use alo_put_aside::{Chosen, HowItShows, NotPutAside, Panel, Privacy};

/// The zoom a person happened to be at.
///
/// Nothing in this file asserts anything about it — task 1's behaviours are about which
/// windows the panel holds and how it shows them. It is here because putting a window
/// aside now takes the view it was put aside from, and a fixture passing
/// [`Zoom::LIFE_SIZE`] would quietly make every test in the crate agree with a panel that
/// stored no zoom at all.
fn a_zoom() -> Zoom {
    Zoom::of(1_750).expect("1_750 thousandths is inside the canvas's own bounds")
}

/// A window of an application, called something, sitting somewhere.
///
/// Both constructors refuse — an application with no name, a patch with no extent — and
/// a fixture that unwrapped silently would be a fixture that could build a window this
/// repository refuses to make. They are unwrapped here because the values are written
/// two lines above and a failure is this file's own mistake.
fn window(id: u64, app: &str, called: &str, x: i64) -> Window {
    Window::of(
        WindowId::numbered(id),
        AppId::named(app).expect("a fixture names its application"),
        called,
        Patch::of(Spot::at(x, 0), 800, 600).expect("a fixture gives its patch an extent"),
        HowItSits::OnTheCanvas,
    )
}

/// **Three windows of one application are three previews.**
///
/// The owner's second behaviour rule, and the reason this panel is not the Dock. The
/// Dock answers *Browser is open*; this answers *which Browser window*, and a person
/// who put three away has to be able to choose between them by name.
#[test]
fn three_windows_of_one_application_are_three_previews() {
    let mut panel = Panel::new();
    panel
        .put_aside(
            &window(1, "Browser", "Launch strategy", 0),
            a_zoom(),
            Privacy::Ordinary,
        )
        .unwrap();
    panel
        .put_aside(
            &window(2, "Browser", "Pricing", 1_000),
            a_zoom(),
            Privacy::Ordinary,
        )
        .unwrap();
    panel
        .put_aside(
            &window(3, "Browser", "The wiki", 2_000),
            a_zoom(),
            Privacy::Ordinary,
        )
        .unwrap();

    assert_eq!(
        panel.holding(),
        3,
        "three windows put aside is three previews"
    );

    let names: Vec<&str> = panel
        .previews()
        .iter()
        .map(|p| p.headline().text())
        .collect();
    assert_eq!(
        names,
        ["The wiki", "Pricing", "Launch strategy"],
        "most recently put aside is first, and every one is named"
    );

    let apps: Vec<&str> = panel.previews().iter().map(|p| p.app().name()).collect();
    assert_eq!(
        apps,
        ["Browser", "Browser", "Browser"],
        "all three belong to one application and are still three rows"
    );
}

/// **Collapsing changes the panel and nothing else.**
///
/// The subject of this test is an **absence**, which is the only kind of test that can
/// hold a rule about what does not happen. It compares the whole preview list before and
/// after rather than checking that the windows still look right — the second would pass
/// for an implementation that changed something and put it back.
#[test]
fn collapsing_touches_no_window_at_all() {
    let mut panel = Panel::new();
    panel
        .put_aside(
            &window(1, "Docs", "Launch strategy", 0),
            a_zoom(),
            Privacy::Ordinary,
        )
        .unwrap();
    panel
        .put_aside(&window(2, "Mail", "Anna", 900), a_zoom(), Privacy::Ordinary)
        .unwrap();

    let before = panel.previews().to_vec();

    panel.collapse();
    assert_eq!(
        panel.previews(),
        before.as_slice(),
        "collapsing changed a preview, and it may change only the panel"
    );

    panel.expand();
    assert_eq!(
        panel.previews(),
        before.as_slice(),
        "expanding changed a preview, and it may change only the panel"
    );

    assert_eq!(panel.holding(), 2, "and neither lost nor gained a window");
}

/// **An empty panel is its own state, not a list of length zero.**
///
/// *The panel is holding nothing* and *there is no panel here* are different, and a
/// count is the same number for both. So the empty case is named, and emptiness wins
/// over the person's choice without overwriting it.
#[test]
fn an_empty_panel_shows_a_handle_and_remembers_the_choice() {
    let mut panel = Panel::new();
    assert_eq!(panel.showing(), HowItShows::AnEdgeHandle);

    panel.collapse();
    assert_eq!(
        panel.showing(),
        HowItShows::AnEdgeHandle,
        "empty beats the choice: there is nothing to draw a rail of"
    );
    assert_eq!(
        panel.chosen(),
        Chosen::Collapsed,
        "and the choice is remembered rather than overwritten by being empty"
    );

    panel
        .put_aside(
            &window(1, "Docs", "Launch strategy", 0),
            a_zoom(),
            Privacy::Ordinary,
        )
        .unwrap();
    assert_eq!(
        panel.showing(),
        HowItShows::ARailOfIcons,
        "putting one aside gives back the rail the person had chosen"
    );

    panel.expand();
    assert_eq!(panel.showing(), HowItShows::NamedPreviews);
}

/// **A window put aside twice is refused, not duplicated.**
///
/// Two previews of one window is a row a person cannot get rid of: restoring either
/// brings back the same window and leaves the other behind.
#[test]
fn a_window_is_not_put_aside_twice() {
    let mut panel = Panel::new();
    let it = window(1, "Docs", "Launch strategy", 0);
    panel.put_aside(&it, a_zoom(), Privacy::Ordinary).unwrap();

    assert_eq!(
        panel.put_aside(&it, a_zoom(), Privacy::Ordinary),
        Err(NotPutAside::ItIsAlreadyThere)
    );
    assert_eq!(panel.holding(), 1);
}

/// **Nothing put aside and this one is not are different answers.**
///
/// A caller that could not tell them apart would tell a person the wrong thing about
/// their own window.
#[test]
fn a_window_that_is_not_there_is_its_own_refusal() {
    let mut panel = Panel::new();
    assert_eq!(
        panel.bring_back(WindowId::numbered(7)),
        Err(NotPutAside::ItIsNotThere),
        "an empty panel refuses by naming the window, not by being empty"
    );

    panel
        .put_aside(
            &window(1, "Docs", "Launch strategy", 0),
            a_zoom(),
            Privacy::Ordinary,
        )
        .unwrap();
    assert_eq!(
        panel.bring_back(WindowId::numbered(7)),
        Err(NotPutAside::ItIsNotThere),
        "and a panel holding something else refuses the same way"
    );
}

/// **Bringing a window back gives the patch it was put aside at.**
///
/// Restoring has to name the saved place rather than wherever the person is looking,
/// and this is the half of that a crate can hold: the panel gives back what it kept.
#[test]
fn bringing_back_gives_the_patch_it_was_put_aside_at() {
    let mut panel = Panel::new();
    let it = window(1, "Docs", "Launch strategy", 4_200);
    let was_at = it.at();
    panel.put_aside(&it, a_zoom(), Privacy::Ordinary).unwrap();

    let back = panel.bring_back(WindowId::numbered(1)).unwrap();
    assert_eq!(
        back.at(),
        was_at,
        "it goes back where it was, not where we are"
    );
    assert_eq!(panel.holding(), 0);
    assert_eq!(panel.showing(), HowItShows::AnEdgeHandle);
}
