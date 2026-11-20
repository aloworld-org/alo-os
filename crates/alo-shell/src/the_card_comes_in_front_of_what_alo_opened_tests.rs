//! When a card may take the keys back from a window alo opened, and every
//! case where it must not.

use alo_dock::WindowId;

use super::{AloTookTheKeys, the_card_takes_the_keys_back};
use crate::the_tasks_a_person_opened::WhichTask;

/// The task the card arrives in, and another.
fn tasks() -> (WhichTask, WhichTask) {
    (
        WhichTask::numbered_for_a_test(0),
        WhichTask::numbered_for_a_test(1),
    )
}

/// alo's opening of a file in window 7 took the keys from `from`.
fn took_from(from: WhichTask) -> Option<AloTookTheKeys> {
    Some(AloTookTheKeys {
        from,
        to: WindowId::numbered(7),
    })
}

/// **The card takes the keys back** where alo's own opening took them from
/// this task, the seat's focus is still on the window alo opened, no task
/// holds the keys, and the applications are in front — main's walk at
/// 71932e07 (run 44161055698, picture 92): the typing card rose behind the
/// VSCodium copy alo had just opened, where the person could not see it.
#[test]
fn the_card_takes_the_keys_back_from_the_window_alo_opened() {
    let (card, _) = tasks();
    assert!(the_card_takes_the_keys_back(
        took_from(card),
        card,
        Some(WindowId::numbered(7)),
        false,
        true
    ));
}

/// **Never where alo took nothing**: a question arriving never takes the
/// keyboard (the owner, 2026-10-26), so a person typing in a document cannot
/// approve a card with Tab and Enter.
#[test]
fn not_where_alo_took_nothing() {
    let (card, _) = tasks();
    assert!(!the_card_takes_the_keys_back(
        None,
        card,
        Some(WindowId::numbered(7)),
        false,
        true
    ));
}

/// **Never for another task** than the one alo took them from.
#[test]
fn not_for_another_task() {
    let (card, other) = tasks();
    assert!(!the_card_takes_the_keys_back(
        took_from(other),
        card,
        Some(WindowId::numbered(7)),
        false,
        true
    ));
}

/// **Never once the person has gone elsewhere**: the seat's focus is on
/// another window, or on none.
#[test]
fn not_once_the_person_has_gone_elsewhere() {
    let (card, _) = tasks();
    for seat in [Some(WindowId::numbered(8)), None] {
        assert!(
            !the_card_takes_the_keys_back(took_from(card), card, seat, false, true),
            "{seat:?}"
        );
    }
}

/// **Never while a task already holds the keys, or with something other than
/// the applications in front** — the terminal, Settings, the record.
#[test]
fn not_while_a_task_holds_them_or_the_applications_are_behind() {
    let (card, _) = tasks();
    let seat = Some(WindowId::numbered(7));
    assert!(!the_card_takes_the_keys_back(
        took_from(card),
        card,
        seat,
        true,
        true
    ));
    assert!(!the_card_takes_the_keys_back(
        took_from(card),
        card,
        seat,
        false,
        false
    ));
}

/// **Through the server's own road, a card with alo's memo for another task
/// takes nothing**, and the memo stays for the task it names — the same road
/// `a_key_reaches_the_task_in_front`'s *a question arriving never takes the
/// keyboard* holds for a card with no memo at all.
#[test]
fn through_the_server_a_card_for_another_task_takes_nothing() {
    let (_runtime, mut server) = crate::a_key_reaches_the_task_in_front::tests::a_session();
    let card = server
        .tasks_mut()
        .and_now_this_one(alo_tasks::ATask::asked_for(
            "write in my file",
            alo_models::InferenceSource::ThisMachine,
        ));
    let other = server
        .tasks_mut()
        .and_now_this_one(alo_tasks::ATask::asked_for(
            "tidy my invoices",
            alo_models::InferenceSource::ThisMachine,
        ));
    server.alo_took_the_keys = took_from(other);
    let daemon = crate::a_key_reaches_the_task_in_front::tests::holding(7, "Type it");

    server.an_answer_came_back(
        card,
        "Shall I?".to_owned(),
        false,
        None,
        Some(&daemon),
        &crate::a_key_reaches_the_task_in_front::tests::in_english(),
    );

    assert!(
        server.the_change_waiting.is_asking(),
        "the control: a question arrived"
    );
    assert!(
        !server.a_task_is_taking_the_keys(),
        "a card took the keys alo had taken from another task"
    );
    assert_eq!(server.alo_took_the_keys, took_from(other));
}
