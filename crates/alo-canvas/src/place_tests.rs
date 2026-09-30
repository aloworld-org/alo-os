use super::Place;

#[test]
fn a_place_is_the_number_it_was_given() {
    let place = Place::numbered(7).expect("7 is a place");
    assert_eq!(place.number(), 7);
}

#[test]
fn zero_is_not_a_place() {
    // The number serde invents for a missing field, refused here so that *no
    // Place at all* cannot arrive looking like a Place. `TheFile` in
    // `alo-arranging` carries `#[serde(default)]`, which is where it would.
    assert_eq!(Place::numbered(0), None);
}

#[test]
fn the_first_place_is_one_and_is_a_place() {
    assert_eq!(Place::FIRST.number(), 1);
    assert_eq!(Place::numbered(1), Some(Place::FIRST));
}

#[test]
fn the_canvas_hands_out_the_next_one() {
    let first = Place::FIRST;
    let second = first.next().expect("there is a second place");
    assert_ne!(first, second);
    assert_eq!(second.number(), 2);
}

#[test]
fn the_numbers_run_out_rather_than_wrapping() {
    // A wrap would hand out a Place that already exists, and two surfaces with
    // one identity is the fault this type was added to make impossible.
    let last = Place::numbered(u64::MAX).expect("the largest number is a place");
    assert_eq!(last.next(), None);
}

#[test]
fn places_order_and_hash_so_they_can_key_a_map() {
    // `alo-arranging` will key an arrangement per Place, and the panel keys a
    // collapse choice by one. Both need this and neither should discover it.
    use std::collections::BTreeMap;
    let mut by_place = BTreeMap::new();
    let first = Place::FIRST;
    let second = first.next().expect("there is a second place");
    by_place.insert(second, "the second");
    by_place.insert(first, "the first");
    let in_order: Vec<_> = by_place.into_iter().map(|(_, what)| what).collect();
    assert_eq!(in_order, vec!["the first", "the second"]);
}
