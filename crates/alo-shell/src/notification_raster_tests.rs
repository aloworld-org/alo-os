//! What a notification card draws, and what no card can be made of.
#![expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use super::*;
use alo_accounts::Session;
use alo_capability::Grantee;
use alo_locking::Seat;
use alo_notifying::arriving::{from_alo_os, from_the_agent};
use alo_notifying::deciding::arrives;
use alo_notifying::quiet::Quiet;
use alo_notifying::{Became, Missed};

/// Everything this machine can say, with nothing translated.
fn words() -> Strings {
    Strings::of(alo_saying::everything_this_machine_can_say().unwrap())
}

/// The ordinary light look, read left to right.
fn light() -> NotificationLook {
    NotificationLook {
        scheme: Scheme::Light,
        scale: TextScale::ordinary(),
        reading: Direction::LeftToRight,
        contrast: Contrast::AsDesigned,
    }
}

/// A seat nobody has locked, for the one account this machine has.
fn an_open_seat() -> Seat<alo_notifying::Notification> {
    let mut store = alo_accounts::Accounts::none().unwrap();
    store.created("ada", 1000, "the-one-they-typed").unwrap();
    let who = store.signs_in("ada", "the-one-they-typed").unwrap();
    Seat::opened(Session::opened(who, 1000).unwrap())
}

/// One notification alo OS sent, shown because nothing is holding it.
fn from_the_machine(title: &str, body: &str, offering: &[(&str, &str)]) -> Shown {
    shown(from_alo_os(title, body, offering).unwrap())
}

/// One notification the agent sent, shown for the same reason.
fn from_an_agent(title: &str) -> Shown {
    shown(from_the_agent(&Grantee::named("@mail"), title, "", &[]).unwrap())
}

/// Put a notification through the one door that makes a `Shown`.
fn shown(notification: alo_notifying::Notification) -> Shown {
    let mut seat = an_open_seat();
    let mut missed = Missed::nothing();
    match arrives(notification, &mut seat, Quiet::No, &mut missed) {
        Became::Shown(shown) => shown,
        Became::Held(why) => unreachable!("an open machine held a notification: {why:?}"),
    }
}

/// The cards drawn for these notifications on a 1920×1080 output.
fn drawn(showing: &[Shown]) -> NotificationPicture {
    let mut labels = WindowControlLabels::new().unwrap();
    picture(
        showing,
        &words(),
        &Dock::shipped(),
        &mut labels,
        (1920, 1080),
        light(),
    )
    .unwrap()
}

/// **Nothing to show draws nothing at all.**
#[test]
fn a_machine_with_no_notifications_draws_none() {
    assert!(drawn(&[]).is_empty());
}

/// **A card says what its sender said, and says who sent it.**
///
/// Every line is compared against the crate's own words: a title shortened, a
/// body re-worded or an action renamed here would fail.
#[test]
fn every_line_is_the_senders_own_words() {
    let shown = from_the_machine(
        "Your disk is nearly full",
        "About 2 GB left",
        &[("open", "Open")],
    );
    let strings = words();
    let notification = shown.notification();

    let picture = drawn(std::slice::from_ref(&shown));
    let card = picture.cards.first().unwrap();

    assert!(
        card.lines
            .contains(&notification.sent_by(&strings).text().to_owned()),
        "the card does not say who it is from: {:?}",
        card.lines
    );
    assert!(card.lines.contains(&"Your disk is nearly full".to_owned()));
    assert!(card.lines.contains(&"About 2 GB left".to_owned()));
    assert!(
        card.lines.contains(&"Open".to_owned()),
        "the action the sender offered is not on the card: {:?}",
        card.lines
    );
}

/// **A notification with no body has no empty line where one would be.**
#[test]
fn a_notification_with_nothing_more_to_say_draws_no_blank_line() {
    let picture = drawn(&[from_the_machine("Printer ready", "", &[])]);
    let card = picture.cards.first().unwrap();

    assert!(
        card.lines.iter().all(|line| !line.is_empty()),
        "an empty line was drawn: {:?}",
        card.lines
    );
}

/// **The agent's card is terracotta and says *the agent* in words.**
///
/// ADR 0010: the colour never arrives alone. Somebody who cannot tell
/// terracotta from anything else reads the sentence instead, and it is
/// `alo-notifying`'s sentence rather than one assembled here.
#[test]
fn the_agents_card_carries_both_the_colour_and_the_word() {
    let terracotta = Contrast::AsDesigned.accent(Scheme::Light, Token::Terracotta.colour());

    let agents = drawn(&[from_an_agent("I have drafted a reply")]);
    let machines = drawn(&[from_the_machine("Printer ready", "", &[])]);

    assert!(
        agents.cards.first().unwrap().the_agents,
        "the agent's card is not marked as the agent's"
    );
    assert!(
        agents.solids.iter().any(|solid| solid.colour == terracotta),
        "the agent's card is not drawn in terracotta"
    );
    assert!(
        !machines
            .solids
            .iter()
            .any(|solid| solid.colour == terracotta),
        "a card that is not the agent's was drawn in the agent's colour"
    );
    assert!(
        !agents
            .cards
            .first()
            .unwrap()
            .lines
            .first()
            .unwrap()
            .is_empty(),
        "the agent's card does not say who sent it"
    );
}

/// **Cards never cover the status area.**
///
/// The two indicators own the corner the dock's status area is at, and they are
/// permanent; a notification arrives and goes. One that covered *what is
/// leaving this machine* would be trading a promise for a convenience.
#[test]
fn a_card_is_drawn_at_the_other_end_from_what_is_leaving() {
    let size = (1920, 1080);
    let dock = Dock::shipped();
    let measure = Measure::of(TextScale::ordinary());
    let layout = dock.layout_on(
        Screen::of(1920, 1080).unwrap(),
        TextScale::ordinary(),
        Direction::LeftToRight,
    );
    let status = Place::of(layout, size, measure.px(8));
    let cards = Place::of_the_other_end(layout, size, measure.px(8));

    assert_ne!(
        format!("{:?}", status.across),
        format!("{:?}", cards.across),
        "notifications are drawn at the same end as the status area"
    );
}

/// **An output too small to lay a dock on refuses rather than drawing half a
/// card.**
#[test]
fn an_output_that_cannot_hold_a_card_is_refused() {
    let mut labels = WindowControlLabels::new().unwrap();

    let refused = picture(
        &[from_the_machine("Printer ready", "", &[])],
        &words(),
        &Dock::shipped(),
        &mut labels,
        (LARGEST_SIDE + 1, 1080),
        light(),
    );

    assert!(
        matches!(refused, Err(RenderError::NotificationScene)),
        "{refused:?}"
    );
}
