//! Task 7's grouping clause, as the owner ruled it on 2026-10-03: grouped by Place by
//! default, **the current Place first and the others in World order**, and *without hiding
//! individual windows behind an application icon*.
//!
//! The hiding half is the one that can go wrong quietly. Grouping a list of windows by the
//! application that owns them is the obvious thing to do and would make this panel a second
//! Dock — so every test that proves grouping *works* is paired with one that proves nothing
//! was **merged**.
//!
//! The ordering half is the one that can be *accidentally* right. `Place` is a `u64`, so
//! anything derived from its `Ord` is the order Places were made in; the fixtures below are
//! built so that creation order and World order **disagree**, because a fixture where they
//! agree passes against every implementation.
#![expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported — \
              the same exemption this crate's other test modules take, with their words"
)]

use alo_canvas::{At, Place, Size, World, Zoom};
use alo_dock::on_the_canvas::{Patch, Spot};
use alo_dock::window::{AppId, HowItSits, Window, WindowId};
use alo_put_aside::a_place_groups_its_windows::{
    Headings, ThePanelInGroups, a_place_groups_its_windows,
};
use alo_put_aside::{Panel, Privacy};

/// The zoom a person happened to be at — deliberately not `LIFE_SIZE`, so a fixture cannot
/// pass against a panel that stored no zoom.
fn a_zoom() -> Zoom {
    Zoom::of(1_750).expect("1_750 thousandths is inside the canvas's own bounds")
}

/// Three Places, **none of them `FIRST`**, numbered so creation order answers nothing.
///
/// `FIRST` is what a fresh machine looks at, so a fixture using it could pass against an
/// implementation that ignored the Place entirely.
fn the_writing_place() -> Place {
    Place::numbered(9).expect("9 is not zero, so it is a Place")
}

fn the_reading_place() -> Place {
    Place::numbered(4).expect("4 is not zero, so it is a Place")
}

fn the_mail_place() -> Place {
    Place::numbered(6).expect("6 is not zero, so it is a Place")
}

fn the_music_place() -> Place {
    Place::numbered(2).expect("2 is not zero, so it is a Place")
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

/// Put a window aside on a Place, failing the test rather than the panel.
fn put(panel: &mut Panel, w: &Window, place: Place) {
    panel
        .put_aside(w, a_zoom(), place, Privacy::Ordinary)
        .expect("a fixture's window is put aside");
}

/// A World laying the given Places out **in the order given**, a viewport apart.
///
/// The order of the argument is the layout, which is the point: the real `Server::the_world`
/// sorts by `Place`'s `Ord` before laying out, so a fixture that also sorted would test
/// nothing about whether this crate reads the layout or re-derives it.
fn a_world_laid_out(places: &[Place]) -> World {
    places
        .iter()
        .enumerate()
        .fold(World::new(), |world, (nth, place)| {
            let nth = i32::try_from(nth).expect("a fixture has few Places");
            world.with(
                *place,
                At::checked(nth * 4_000, 0).expect("a fixture's tile is on the plane"),
                Size::checked(1_920, 1_080).expect("a fixture's tile has an extent"),
            )
        })
}

/// The window numbers of one Place's group, or empty where it has none.
fn ids_on(grouped: &ThePanelInGroups<'_>, place: Place) -> Vec<u64> {
    grouped
        .groups()
        .iter()
        .find(|group| group.place() == place)
        .map(|group| {
            group
                .previews()
                .iter()
                .map(|preview| preview.window().number())
                .collect()
        })
        .unwrap_or_default()
}

/// The Places in the order the grouping answered.
fn places_in_order(grouped: &ThePanelInGroups<'_>) -> Vec<Place> {
    grouped.groups().iter().map(|group| group.place()).collect()
}

/// **The current Place comes first, whatever the World says.**
///
/// The World lays reading out before writing and the person is looking at writing, so World
/// order alone answers reading first — and the ruling's *current Place first* is the only
/// thing that puts writing at the front.
#[test]
fn the_place_the_person_is_looking_at_comes_first() {
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

    let world = a_world_laid_out(&[the_reading_place(), the_writing_place()]);
    let grouped = a_place_groups_its_windows(&panel, the_writing_place(), &world);

    assert_eq!(
        places_in_order(&grouped),
        vec![the_writing_place(), the_reading_place()],
        "the Place the person is looking at is not first"
    );
}

/// **The others follow in World order, not in `Place` order.**
///
/// Three Places follow the current one, laid out mail (6), music (2), reading (4). Ascending
/// by `Place`'s `u64` would be music, reading, mail — so this fails any implementation that
/// sorts, and **the fixture is chosen so that the right answer is not a sorted answer.**
#[test]
fn the_other_places_follow_in_world_order() {
    let mut panel = Panel::new();
    put(
        &mut panel,
        &window(1, "Docs", "Launch strategy", 0),
        the_writing_place(),
    );
    put(
        &mut panel,
        &window(2, "Mail", "Re: the quarter", 900),
        the_mail_place(),
    );
    put(
        &mut panel,
        &window(3, "Reader", "A long article", 1_800),
        the_reading_place(),
    );
    put(
        &mut panel,
        &window(4, "Music", "A quiet album", 2_700),
        the_music_place(),
    );

    let world = a_world_laid_out(&[
        the_writing_place(),
        the_mail_place(),
        the_music_place(),
        the_reading_place(),
    ]);
    let grouped = a_place_groups_its_windows(&panel, the_writing_place(), &world);

    assert_eq!(
        places_in_order(&grouped),
        vec![
            the_writing_place(),
            the_mail_place(),
            the_music_place(),
            the_reading_place(),
        ],
        "the Places after the current one are not in the World's layout order"
    );
}

/// **And laying the World out differently answers differently**, which is what proves the
/// order is read rather than coincidental.
///
/// The same panel and the same current Place, a different layout. **This one is also not a
/// sorted order** — reading (4), mail (6), music (2) against ascending music, reading, mail —
/// so a sort fails this as well as the test above.
///
/// An earlier version of this pair used two following Places and expected `[reading, mail]`,
/// which **is** ascending by `Place`, so it passed against the very sort it was written to
/// forbid. The mutation found it: replacing World order with `Place`'s `Ord` failed one test
/// of the pair instead of both.
#[test]
fn laying_the_world_out_differently_orders_the_groups_differently() {
    let mut panel = Panel::new();
    put(
        &mut panel,
        &window(1, "Docs", "Launch strategy", 0),
        the_writing_place(),
    );
    put(
        &mut panel,
        &window(2, "Mail", "Re: the quarter", 900),
        the_mail_place(),
    );
    put(
        &mut panel,
        &window(3, "Reader", "A long article", 1_800),
        the_reading_place(),
    );
    put(
        &mut panel,
        &window(4, "Music", "A quiet album", 2_700),
        the_music_place(),
    );

    let world = a_world_laid_out(&[
        the_writing_place(),
        the_reading_place(),
        the_mail_place(),
        the_music_place(),
    ]);
    let grouped = a_place_groups_its_windows(&panel, the_writing_place(), &world);

    assert_eq!(
        places_in_order(&grouped),
        vec![
            the_writing_place(),
            the_reading_place(),
            the_mail_place(),
            the_music_place(),
        ],
        "the group order did not follow the World when the World changed"
    );
}

/// **A Place the World does not name goes after the ones it does, in the panel's own order.**
///
/// The case the ruling could not have anticipated: `Server::the_world` is built from the
/// **mapped** surfaces, and a put-aside window is hidden — so a Place whose windows have all
/// been put aside is absent from the World and cannot be ordered by it.
///
/// The World here names only writing. Mail and reading are unnamed, mail was put aside
/// **last**, so the panel's own order is mail then reading — while ascending `Place` order is
/// reading (4) then mail (6). The two disagree, which is what makes this capable of failing.
#[test]
fn a_place_the_world_does_not_name_goes_after_the_ones_it_does() {
    let mut panel = Panel::new();
    put(
        &mut panel,
        &window(1, "Docs", "Launch strategy", 0),
        the_writing_place(),
    );
    put(
        &mut panel,
        &window(2, "Reader", "A long article", 900),
        the_reading_place(),
    );
    put(
        &mut panel,
        &window(3, "Mail", "Re: the quarter", 1_800),
        the_mail_place(),
    );

    // Only the writing Place still has anything on the plane.
    let world = a_world_laid_out(&[the_writing_place()]);
    let grouped = a_place_groups_its_windows(&panel, the_writing_place(), &world);

    assert_eq!(
        places_in_order(&grouped),
        vec![the_writing_place(), the_mail_place(), the_reading_place()],
        "a Place the World cannot name was not placed after the ones it can, in the panel's \
         own most-recent-first order"
    );
}

/// **The current Place takes no group when nothing has been put aside there.**
///
/// Absent rather than present and empty — and the current Place is the one a naive
/// implementation would always include.
#[test]
fn the_current_place_takes_no_group_when_it_holds_nothing() {
    let mut panel = Panel::new();
    put(
        &mut panel,
        &window(1, "Mail", "Re: the quarter", 0),
        the_mail_place(),
    );

    let world = a_world_laid_out(&[the_mail_place()]);
    let grouped = a_place_groups_its_windows(&panel, the_writing_place(), &world);

    assert_eq!(
        places_in_order(&grouped),
        vec![the_mail_place()],
        "the Place the person is looking at took a group with nothing put aside on it"
    );
}

/// **One represented Place takes no heading; several take one each.**
#[test]
fn headings_are_not_needed_for_one_place_and_are_for_several() {
    let mut panel = Panel::new();
    put(
        &mut panel,
        &window(1, "Docs", "Launch strategy", 0),
        the_writing_place(),
    );
    let world = a_world_laid_out(&[the_writing_place()]);
    assert_eq!(
        a_place_groups_its_windows(&panel, the_writing_place(), &world).headings(),
        Headings::NotNeeded,
        "one Place is represented, and a heading above it says what the panel already says"
    );

    put(
        &mut panel,
        &window(2, "Mail", "Re: the quarter", 900),
        the_mail_place(),
    );
    let world = a_world_laid_out(&[the_writing_place(), the_mail_place()]);
    assert_eq!(
        a_place_groups_its_windows(&panel, the_writing_place(), &world).headings(),
        Headings::OnePerGroup,
        "two Places are represented and nothing tells a person which windows are which"
    );
}

/// **Each Place holds the windows put aside on it, and no others.**
///
/// Interleaved between two Places as a person would interleave them — write, read, write,
/// read — so a grouping that split the list in half fails. Membership only: both sides are
/// sorted, because the order inside a group has a test of its own.
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

    let world = a_world_laid_out(&[the_writing_place(), the_reading_place()]);
    let grouped = a_place_groups_its_windows(&panel, the_writing_place(), &world);

    assert_eq!(grouped.groups().len(), 2, "two Places were used");

    let mut writing = ids_on(&grouped, the_writing_place());
    let mut reading = ids_on(&grouped, the_reading_place());
    writing.sort_unstable();
    reading.sort_unstable();

    assert_eq!(writing, vec![1, 3], "the writing Place's group is wrong");
    assert_eq!(reading, vec![2, 4], "the reading Place's group is wrong");
}

/// **Two windows of one application on one Place are two previews in that group.**
///
/// The owner's clause, asked where it would be violated: inside a single group, where
/// grouping by application would collapse them and the group would still look right from
/// outside.
///
/// A count, not a lookup. Asking *is window 2 there* passes against an implementation that
/// kept one preview per application and happened to keep the second; only the count tells
/// *both* from *one of them*.
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

    let world = a_world_laid_out(&[the_writing_place()]);
    let grouped = a_place_groups_its_windows(&panel, the_writing_place(), &world);
    let ids = ids_on(&grouped, the_writing_place());

    assert_eq!(
        ids.len(),
        2,
        "two Browser windows on one Place became {} preview(s), so the panel is grouping by \
         application and has turned into a second Dock",
        ids.len()
    );
}

/// **Inside a group, the panel's own order is kept: most recently put aside first.**
///
/// `Panel` decides that order and documents it in four places; this file must not have an
/// opinion, because two answers to *what order are the previews in* is the fault that shows
/// up as previews moving under a person's hand.
///
/// The ids are chosen so the right answer is neither sorted nor reverse-sorted: put aside in
/// the order 20, 30, 10, the panel holds `[10, 30, 20]`. Ascending is `[10, 20, 30]` and
/// descending is `[30, 20, 10]`, so a sort by window id — the easiest accidental sort — fails
/// this rather than passing by coincidence.
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

    let world = a_world_laid_out(&[the_writing_place()]);
    let grouped = a_place_groups_its_windows(&panel, the_writing_place(), &world);

    assert_eq!(
        ids_on(&grouped, the_writing_place()),
        vec![10, 30, 20],
        "the order inside a Place is not the panel's own, most-recent-first order"
    );
}

/// **Every window the panel holds appears exactly once across the groups.**
///
/// The clause as a property rather than a case: nothing merged, nothing duplicated, nothing
/// dropped. One application appears on **both** Places, which catches a grouping keyed on the
/// application instead of the Place.
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

    let world = a_world_laid_out(&[the_writing_place(), the_reading_place()]);
    let grouped = a_place_groups_its_windows(&panel, the_writing_place(), &world);

    let mut seen: Vec<u64> = grouped
        .groups()
        .iter()
        .flat_map(|group| group.previews().iter().map(|p| p.window().number()))
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

/// **A Place with nothing put aside is absent, not an empty group — even when the World names
/// it.**
#[test]
fn a_place_with_nothing_put_aside_is_absent_rather_than_empty() {
    let mut panel = Panel::new();
    put(
        &mut panel,
        &window(1, "Docs", "Launch strategy", 0),
        the_writing_place(),
    );

    let world = a_world_laid_out(&[the_writing_place(), the_reading_place()]);
    let grouped = a_place_groups_its_windows(&panel, the_writing_place(), &world);

    assert_eq!(
        places_in_order(&grouped),
        vec![the_writing_place()],
        "a Place with nothing put aside has a group, although the World names it"
    );
}

/// **An empty panel groups nothing, and that is not an error.**
#[test]
fn an_empty_panel_has_no_groups() {
    let panel = Panel::new();
    let world = World::new();
    let grouped = a_place_groups_its_windows(&panel, the_writing_place(), &world);
    assert!(grouped.is_empty(), "an empty panel produced a group");
    assert_eq!(
        grouped.headings(),
        Headings::NotNeeded,
        "an empty panel asked for headings"
    );
}

/// **Bringing a window back leaves the grouping correct.**
///
/// `Panel::bring_back` is `previews.remove(at)`, which shifts every later index. A stored
/// grouping would have to be repaired here; a derived one cannot go stale. This is what makes
/// *derived* a measured property rather than a claim in a doc comment.
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

    let world = a_world_laid_out(&[the_writing_place(), the_reading_place()]);
    let grouped = a_place_groups_its_windows(&panel, the_writing_place(), &world);

    assert_eq!(
        ids_on(&grouped, the_writing_place()),
        vec![3],
        "after a window came back, its Place's group is wrong"
    );
    assert_eq!(
        ids_on(&grouped, the_reading_place()),
        vec![2],
        "bringing a window back from one Place changed another Place's group"
    );
}

/// **And a Place whose last window came back loses its group.**
///
/// Not an empty group left behind. Absent-rather-than-empty has to survive removal as well as
/// hold at the start, and those are two different bugs.
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

    let world = a_world_laid_out(&[the_writing_place(), the_reading_place()]);
    let grouped = a_place_groups_its_windows(&panel, the_writing_place(), &world);

    assert_eq!(
        places_in_order(&grouped),
        vec![the_writing_place()],
        "the Place whose only window came back still has a group"
    );
}
