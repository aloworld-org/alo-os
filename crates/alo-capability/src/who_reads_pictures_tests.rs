use super::*;

/// **A machine arrives with no one reading pictures**, so alo takes none and
/// nothing is sent.
#[test]
fn a_machine_arrives_with_no_one_reading_pictures() {
    let arrives = WhoReadsPictures::default();
    assert_eq!(arrives, WhoReadsPictures::NoOne);
    assert!(!arrives.may_take_pictures());
    assert!(!arrives.sends_pictures_away());
    assert_eq!(arrives.the_provider(), None);
}

/// **The person's model on this computer reads them, and nothing leaves.**
#[test]
fn this_computer_reads_them_and_sends_nothing() {
    let here = WhoReadsPictures::ThisComputer;
    assert!(here.may_take_pictures());
    assert!(!here.sends_pictures_away());
    assert_eq!(here.the_provider(), None);
    assert_eq!(here.what_it_costs(), None);
}

/// **An outside service reads them, and every picture leaves for it**, said
/// in ADR 0147 §4's own sentence with the service's name in it.
#[test]
fn a_provider_reads_them_and_each_picture_leaves_for_it() {
    let away = WhoReadsPictures::a_provider("Mistral").unwrap();
    assert!(away.may_take_pictures());
    assert!(away.sends_pictures_away());
    assert_eq!(away.the_provider(), Some("Mistral"));
    assert_eq!(away.what_it_costs(), Some(words::PICTURES_SENT_TO));
    assert_eq!(
        words::PICTURES_SENT_TO.says(),
        format!("A picture of the window alo is reading is sent to {{{THE_PROVIDER}}} each time")
    );
}

/// **No one costs nothing to say**, because nothing is taken.
#[test]
fn no_one_costs_nothing() {
    assert_eq!(WhoReadsPictures::NoOne.what_it_costs(), None);
}

/// **A provider's name nothing could be is not a choice**: empty, or with
/// space around it, which trimming would turn into a different service.
#[test]
fn a_name_nothing_could_be_is_not_a_provider() {
    for name in ["", " ", " Mistral", "Mistral ", "Mistral\n"] {
        assert_eq!(WhoReadsPictures::a_provider(name), None, "{name:?}");
    }
    // **A name is matched as written**: no case folding.
    assert_eq!(
        WhoReadsPictures::a_provider("mistral")
            .unwrap()
            .the_provider(),
        Some("mistral")
    );
}

/// **A file that will not read is no one** — never a reader the file might
/// have meant, and above all never a service a picture would leave for.
#[test]
fn a_file_that_will_not_read_is_no_one() {
    let read: Result<WhoReadsPictures, ()> = Err(());
    assert_eq!(WhoReadsPictures::as_read(read), WhoReadsPictures::NoOne);
    let read: Result<WhoReadsPictures, ()> = Ok(WhoReadsPictures::ThisComputer);
    assert_eq!(WhoReadsPictures::as_read(read), WhoReadsPictures::ThisComputer);
}

/// **Each choice has its own name and line**, and the two named ones say
/// different things — a choice drawn with another's words is a choice the
/// person cannot tell apart.
#[test]
fn each_choice_says_what_it_is_in_its_own_words() {
    let provider = WhoReadsPictures::a_provider("Mistral").unwrap();
    assert_eq!(WhoReadsPictures::NoOne.the_word(), Some(words::READ_BY_NO_ONE));
    assert_eq!(
        WhoReadsPictures::ThisComputer.the_word(),
        Some(words::READ_BY_THIS_COMPUTER)
    );
    // **A provider is drawn by its own name**, which is data and never
    // translated.
    assert_eq!(provider.the_word(), None);
    assert_eq!(
        WhoReadsPictures::NoOne.what_it_is(),
        words::READ_BY_NO_ONE_IS
    );
    assert_eq!(
        WhoReadsPictures::ThisComputer.what_it_is(),
        words::READ_BY_THIS_COMPUTER_IS
    );
    assert_eq!(provider.what_it_is(), words::READ_BY_A_PROVIDER_IS);
}

/// **The two choices that are always offered**, in the order Settings draws
/// them: no one first, as a machine arrives.
#[test]
fn no_one_and_this_computer_are_always_offered_in_that_order() {
    assert_eq!(
        WhoReadsPictures::ALWAYS_OFFERED,
        [WhoReadsPictures::NoOne, WhoReadsPictures::ThisComputer]
    );
}

// The second reader's cases (law 7), written from the promise without the
// implementation, 2026-11-17.

/// **What is always offered sends nothing**, and each is drawn by a word of
/// the product's own — no built-in reader is among them, because it is not
/// built.
#[test]
fn what_is_always_offered_sends_nothing_and_has_its_own_word() {
    for offered in WhoReadsPictures::ALWAYS_OFFERED {
        assert!(
            !offered.sends_pictures_away(),
            "{offered:?} is offered to everyone and sends"
        );
        assert!(
            offered.the_word().is_some(),
            "{offered:?} has no word to be drawn by"
        );
        assert_eq!(offered.the_provider(), None);
    }
}

/// **Space inside a name is a name, and case is part of it**: "lantern" is not
/// the service the person added as "Lantern".
#[test]
fn a_name_keeps_its_inner_space_and_its_case() {
    assert_eq!(
        WhoReadsPictures::a_provider("Lantern AI")
            .unwrap()
            .the_provider(),
        Some("Lantern AI")
    );
    assert_ne!(
        WhoReadsPictures::a_provider("lantern"),
        WhoReadsPictures::a_provider("Lantern")
    );
    assert_eq!(WhoReadsPictures::a_provider("\t"), None);
}
