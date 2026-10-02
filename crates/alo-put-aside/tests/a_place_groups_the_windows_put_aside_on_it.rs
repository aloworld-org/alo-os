//! Task 7's grouping clause, from `docs/autonomy/the-canvas-and-its-places.md`:
//! *grouping by Place **without hiding individual windows behind an application icon***.
//!
//! The second half is the half that can go wrong quietly. Grouping a list of windows by
//! the application that owns them is the obvious thing to do and would make this panel a
//! second Dock — so every test here that proves grouping *works* is paired with one that
//! proves nothing was **merged**.
#![expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected Err is the failure being reported — \
              the same exemption this crate's other test modules take, with their words"
)]

use alo_canvas::{Place, Zoom};
use alo_dock::on_the_canvas::{Patch, Spot};
use alo_dock::window::{AppId, HowItSits, Window, WindowId};
use alo_put_aside::a_place_groups_its_windows::a_place_groups_its_windows;
use alo_put_aside::{Panel, Preview, Privacy};
use std::collections::BTreeMap;

/// The zoom a person happened to be at — deliberately not `LIFE_SIZE`, so a fixture
/// cannot pass against a panel that stored no zoom. This crate's other suites take the
/// same care and for the same reason.
fn a_zoom() -> Zoom {
    Zoom::of(1_750).expect("1_750 thousandths is inside the canvas's own bounds")
}

/// Two Places that are **neither of them `FIRST`**.
///
/// `Place::FIRST` is what a fresh machine looks at, so a grouping test using it could pass
/// against an implementation that ignored the Place and answered with the default. Both
/// are numbered, and the **higher one is used first** in every test below, so Place order
/// cannot be satisfied by put-aside order happening to agree with it.
fn the_writing_place() -> Place {
    Place::numbered(9).expect("9 is not zero, so it is a Place")
}

fn the_reading_place() -> Place {
    Place::numbered(4).expect("4 is not zero, so it is a Place")
}

fn window(id: u64, app: &str, called: &str, x: i64) -> Window {
    Window::of(
        WindowId::numbered(id),
        Some(AppId::named(app).expect("a fixture names its application")),
        called,
        Patch::of(Spot::at(x, 0), 800, 600).expect("a fixture gives its patch an extent"),
        HowItSits::OnTheCanvas,
    )
}

/// The group for a Place, named in the failure rather than panicking anonymously.
///
/// `grouped[&place]` is an index, and this crate takes no `clippy::indexing_slicing`
/// exemption anywhere — this file will not be the first to ask for one. The message also
/// `expect` rather than a formatted panic, because `panic` is denied in this workspace
/// even in tests — `Cargo.toml` says so — and the Place is on the calling line anyway.
fn group_on<'a>(grouped: &'a BTreeMap<Place, Vec<&'a Preview>>, place: Place) -> &'a [&'a Preview] {
    grouped
        .get(&place)
        .map(Vec::as_slice)
        .expect("the fixture put a window aside on this Place, so it has a group")
}

/// Put a window aside on a Place, failing the test rather than the panel.
fn put(panel: &mut Panel, w: &Window, place: Place) {
    panel
        .put_aside(w, a_zoom(), place, Privacy::Ordinary)
        .expect("a fixture's window is put aside");
}

/// **Each Place holds the windows that were put aside on it, and no others.**
///
/// The windows are interleaved between the two Places as a person would interleave them —
/// write, read, write, read — so a grouping that simply split the list in half would fail.
/// That is the shape a first attempt has, and it passes a test whose fixture put all of
/// one Place's windows away before any of the other's.
///
/// **Membership only.** Both sides are sorted before comparing, so this cannot pass or
/// fail on ordering: the order inside a group has a test of its own, and a test that
/// asserted both would not say which had broken.
#[test]
fn each_place_holds_the_windows_put_aside_on_it_and_no_others() {
    let mut panel = Panel::new();
    put(
        &mut panel,
        &window(1, "Docs", "Launch strategy", 0),
        the_writing_place(),
    );
    put(
        &mut panel,
        &window(2, "Mail", "Re: the quarter", 900),
        the_reading_place(),
    );
    put(
        &mut panel,
        &window(3, "Docs", "Notes", 1_800),
        the_writing_place(),
    );
    put(
        &mut panel,
        &window(4, "Reader", "A long article", 2_700),
        the_reading_place(),
    );

    let grouped = a_place_groups_its_windows(&panel);

    assert_eq!(
        grouped.len(),
        2,
        "two Places were used and the grouping has {} groups",
        grouped.len()
    );

    let mut writing: Vec<u64> = group_on(&grouped, the_writing_place())
        .iter()
        .map(|p| p.window().number())
        .collect();
    let mut reading: Vec<u64> = group_on(&grouped, the_reading_place())
        .iter()
        .map(|p| p.window().number())
        .collect();
    writing.sort_unstable();
    reading.sort_unstable();

    assert_eq!(
        writing,
        vec![1, 3],
        "the writing Place's group is not the windows put aside on it"
    );
    assert_eq!(
        reading,
        vec![2, 4],
        "the reading Place's group is not the windows put aside on it"
    );
}

/// **Two windows of one application on one Place are two previews in that group.**
///
/// This is the owner's clause — *without hiding individual windows behind an application
/// icon* — asked where it would actually be violated: inside a single group, where
/// grouping by application would collapse them and the group would still look right from
/// the outside.
///
/// A count, not a lookup. Asking *is window 2 in the group* passes against an
/// implementation that kept one preview per application and happened to keep the second
/// one; only the count can tell *both* from *one of them*.
#[test]
fn two_windows_of_one_application_on_one_place_are_two_previews() {
    let mut panel = Panel::new();
    put(
        &mut panel,
        &window(1, "Browser", "The quarter", 0),
        the_writing_place(),
    );
    put(
        &mut panel,
        &window(2, "Browser", "A different tab", 900),
        the_writing_place(),
    );

    let grouped = a_place_groups_its_windows(&panel);
    let group = &group_on(&grouped, the_writing_place());

    assert_eq!(
        group.len(),
        2,
        "two Browser windows on one Place became {} preview(s), so the panel is grouping \
         by application and has turned into a second Dock",
        group.len()
    );
    let titles: Vec<&str> = group.iter().map(|p| p.headline().text()).collect();
    assert!(
        titles.contains(&"The quarter") && titles.contains(&"A different tab"),
        "both windows are counted but they are not both named: {titles:?}"
    );
}

/// **Groups come out in Place order, whatever order the person used.**
///
/// Place 9 is written to first and Place 4 second, so put-aside order and Place order
/// disagree — a grouping that preserved insertion order would answer 9 before 4.
/// Two identical panels must produce one layout, or a draw moves a person's previews for
/// a reason they cannot see.
///
/// **This is a guard on the container, and it cannot fail against today's code**, because
/// `BTreeMap::keys` yields ascending by construction. It is here deliberately rather than
/// as decoration: the requirement is a layout that does not depend on insertion order, and
/// the day somebody returns a `HashMap` for speed or a `Vec` for convenience, this is what
/// says no. Stated so nobody mistakes it for evidence that an ordering decision was
/// measured — the ordering *inside* a group is the one with a mutation behind it.
#[test]
fn the_groups_come_out_in_place_order_not_in_the_order_they_were_used() {
    let mut panel = Panel::new();
    put(
        &mut panel,
        &window(1, "Docs", "Launch strategy", 0),
        the_writing_place(),
    );
    put(
        &mut panel,
        &window(2, "Mail", "Re: the quarter", 900),
        the_reading_place(),
    );

    let order: Vec<Place> = a_place_groups_its_windows(&panel).keys().copied().collect();

    assert_eq!(
        order,
        vec![the_reading_place(), the_writing_place()],
        "the groups are in the order the person used rather than in Place order"
    );
}

/// **Inside a group, the panel's own order is kept: most recently put aside first.**
///
/// `Panel` decides that order and documents it in four places. This file must not have an
/// opinion, because two answers to *what order are the previews in* is the fault that
/// would show up as previews moving under a person's hand.
///
/// **The ids are chosen so that the right answer is neither sorted nor reverse-sorted.**
/// Put aside in the order 20, 30, 10, the panel holds `[10, 30, 20]`. Ascending by id is
/// `[10, 20, 30]` and descending is `[30, 20, 10]`, so an implementation that sorted by
/// window id — the easiest accidental sort, and one that `Place` being `Ord` invites —
/// fails this rather than passing it by coincidence.
///
/// An earlier version of this test used 30, 20, 10 and expected `[10, 20, 30]`, which is
/// exactly what sorting ascending by id produces. It would have passed against a sort.
#[test]
fn inside_a_group_the_panels_own_order_is_kept() {
    let mut panel = Panel::new();
    put(
        &mut panel,
        &window(20, "Docs", "First away", 0),
        the_writing_place(),
    );
    put(
        &mut panel,
        &window(30, "Docs", "Second away", 900),
        the_writing_place(),
    );
    put(
        &mut panel,
        &window(10, "Docs", "Third away", 1_800),
        the_writing_place(),
    );

    let ids: Vec<u64> = group_on(&a_place_groups_its_windows(&panel), the_writing_place())
        .iter()
        .map(|p| p.window().number())
        .collect();

    assert_eq!(
        ids,
        vec![10, 30, 20],
        "the order inside a Place is not the panel's own, most-recent-first order"
    );
}

/// **Every window the panel holds appears exactly once across the groups.**
///
/// The clause stated as a property rather than as a case: nothing merged, nothing
/// duplicated, nothing dropped. Four windows, three applications, two Places, with one
/// application appearing on **both** Places — which is the arrangement that would catch a
/// grouping keyed on the application instead of the Place, because `Docs` would collapse
/// across two Places into one group.
#[test]
fn every_window_appears_once_and_nothing_is_merged() {
    let mut panel = Panel::new();
    put(
        &mut panel,
        &window(1, "Docs", "Launch strategy", 0),
        the_writing_place(),
    );
    put(
        &mut panel,
        &window(2, "Docs", "Yesterday's reading", 900),
        the_reading_place(),
    );
    put(
        &mut panel,
        &window(3, "Mail", "Re: the quarter", 1_800),
        the_writing_place(),
    );
    put(
        &mut panel,
        &window(4, "Reader", "A long article", 2_700),
        the_reading_place(),
    );

    let grouped = a_place_groups_its_windows(&panel);
    let mut seen: Vec<u64> = grouped
        .values()
        .flat_map(|group| group.iter().map(|p| p.window().number()))
        .collect();
    seen.sort_unstable();

    assert_eq!(
        seen,
        vec![1, 2, 3, 4],
        "the grouping does not hold every window exactly once"
    );
    assert_eq!(
        seen.len(),
        panel.holding(),
        "the grouping holds {} windows and the panel holds {}",
        seen.len(),
        panel.holding()
    );
}

/// **A Place with nothing put aside is absent, not an empty group.**
///
/// The same answer `TheCollapseChoice` gives for a Place nobody collapsed, and for the
/// same reason: a group for a Place a person merely looked at is a row that means nothing
/// to them. Asserted while *another* Place has windows, so it cannot pass by the whole
/// answer being empty.
#[test]
fn a_place_with_nothing_put_aside_is_absent_rather_than_empty() {
    let mut panel = Panel::new();
    put(
        &mut panel,
        &window(1, "Docs", "Launch strategy", 0),
        the_writing_place(),
    );

    let grouped = a_place_groups_its_windows(&panel);

    assert!(
        !grouped.contains_key(&the_reading_place()),
        "a Place with nothing put aside has a group"
    );
    assert_eq!(grouped.len(), 1, "only one Place has anything put aside");
}

/// **An empty panel groups nothing, and that is not an error.**
#[test]
fn an_empty_panel_has_no_groups() {
    assert!(
        a_place_groups_its_windows(&Panel::new()).is_empty(),
        "an empty panel produced a group"
    );
}

/// **Bringing a window back leaves the grouping correct.**
///
/// `Panel::bring_back` is `previews.remove(at)`, which shifts every later index. A stored
/// grouping would have to be repaired here; a derived one cannot go stale. This test is
/// what makes *derived* a measured property rather than a claim in a doc comment.
#[test]
fn bringing_a_window_back_leaves_the_grouping_correct() {
    let mut panel = Panel::new();
    put(
        &mut panel,
        &window(1, "Docs", "Launch strategy", 0),
        the_writing_place(),
    );
    put(
        &mut panel,
        &window(2, "Mail", "Re: the quarter", 900),
        the_reading_place(),
    );
    put(
        &mut panel,
        &window(3, "Docs", "Notes", 1_800),
        the_writing_place(),
    );

    panel
        .bring_back(WindowId::numbered(1))
        .expect("window 1 was put aside, so it can come back");

    let grouped = a_place_groups_its_windows(&panel);
    let writing: Vec<u64> = group_on(&grouped, the_writing_place())
        .iter()
        .map(|p| p.window().number())
        .collect();

    assert_eq!(
        writing,
        vec![3],
        "after a window came back, its Place's group is wrong: {writing:?}"
    );
    assert_eq!(
        group_on(&grouped, the_reading_place()).len(),
        1,
        "bringing a window back from one Place changed another Place's group"
    );
}

/// **And a Place whose last window came back loses its group.**
///
/// Not an empty group left behind. The absent-rather-than-empty rule has to survive
/// removal as well as hold at the start, and those are two different bugs.
#[test]
fn a_place_whose_last_window_came_back_has_no_group() {
    let mut panel = Panel::new();
    put(
        &mut panel,
        &window(1, "Docs", "Launch strategy", 0),
        the_writing_place(),
    );
    put(
        &mut panel,
        &window(2, "Mail", "Re: the quarter", 900),
        the_reading_place(),
    );

    panel
        .bring_back(WindowId::numbered(2))
        .expect("window 2 was put aside, so it can come back");

    let grouped = a_place_groups_its_windows(&panel);

    assert!(
        !grouped.contains_key(&the_reading_place()),
        "the Place whose only window came back still has a group"
    );
    assert_eq!(
        grouped.len(),
        1,
        "one Place still has a window, so there is one group"
    );
}
