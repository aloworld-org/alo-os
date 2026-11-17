//! What the catalogue has to be, and every way it is refused.
//!
//! Each refusal here is a mistake somebody editing `catalogue.toml` can make,
//! and the test is the sentence they get back. The *nothing already shipped*
//! one is the reason this reader takes the shipped list as a parameter:
//! a check that is only ever run against the one list that passes it is not a
//! check that has been run.

#![expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use super::*;
use crate::label::Label;
use crate::role::Role;

/// The decided catalogue with one line of it replaced.
fn replacing(from: &str, to: &str) -> String {
    assert!(THE_CATALOGUE.contains(from), "{from}");
    THE_CATALOGUE.replacen(from, to, 1)
}

/// The decided shipped list, which every refusal here is read against.
fn what_a_machine_ships() -> Shipped {
    Shipped::decided().unwrap()
}

/// One more entry, appended — the shape a person adds an application in.
fn with_one_more(category: &str, identifier: &str, version: &str, licence: &str) -> String {
    format!(
        "{THE_CATALOGUE}\n[[application]]\ncategory = \"{category}\"\nidentifier = \
         \"{identifier}\"\nsource = \"flathub\"\nversion = \"{version}\"\nlicence = \
         \"{licence}\"\nverified = true\nearly = false\n"
    )
}

/// **A label is read off its entry**: proprietary terms are *not open source*,
/// `verified = false` is *not verified*, `early = true` is *early version*, in
/// that order — and an open entry its makers published, not early, has none.
#[test]
fn an_entrys_labels_are_read_off_it() {
    let shipped = what_a_machine_ships();
    let text = format!(
        "{THE_CATALOGUE}\n[[application]]\ncategory = \"audio\"\nidentifier = \
         \"com.example.All\"\nsource = \"flathub\"\nversion = \"1.0\"\nlicence = \
         \"LicenseRef-proprietary\"\nverified = false\nearly = true\n"
    );
    let catalogue = Catalogue::read(&text, &shipped).unwrap();
    let labels_of = |identifier: &str| {
        catalogue
            .every()
            .iter()
            .find(|one| one.application().identifier() == identifier)
            .unwrap()
            .labels()
    };
    assert_eq!(
        labels_of("com.example.All"),
        vec![
            Label::NotOpenSource,
            Label::NotVerified,
            Label::EarlyVersion
        ]
    );
    assert_eq!(labels_of("org.gimp.GIMP"), Vec::<Label>::new());
    assert_eq!(
        labels_of("org.darktable.Darktable"),
        vec![Label::NotVerified]
    );
}

/// **An entry that does not say whether it is verified, or whether it is
/// early, is refused** — so a forgotten field is a refused file and never a
/// label quietly missing from a row.
#[test]
fn an_entry_that_does_not_say_verified_or_early_is_refused() {
    let shipped = what_a_machine_ships();
    let gimps = "licence = \"GPL-3.0+ AND LGPL-3.0+\"\nverified = true\nearly = false\n";
    for instead in [
        "licence = \"GPL-3.0+ AND LGPL-3.0+\"\nearly = false\n",
        "licence = \"GPL-3.0+ AND LGPL-3.0+\"\nverified = true\n",
        "licence = \"GPL-3.0+ AND LGPL-3.0+\"\nverified = \"yes\"\nearly = false\n",
    ] {
        let text = replacing(gimps, instead);
        assert!(
            matches!(
                Catalogue::read(&text, &shipped),
                Err(NotCatalogued::NotRead { .. })
            ),
            "{instead:?} was read"
        );
    }
}

#[test]
fn the_decided_catalogue_holds() {
    let catalogue = Catalogue::decided().unwrap();
    assert_eq!(catalogue.how_many(), 62);
    for category in Category::EVERY {
        assert!(
            catalogue.under(category).next().is_some(),
            "nothing under {}",
            category.named()
        );
    }
}

/// **Every category is drawn in order**, so a screen showing the shelves does
/// not sort the list itself.
#[test]
fn the_catalogue_is_kept_in_its_categories_order() {
    let catalogue = Catalogue::decided().unwrap();
    let mut places: Vec<usize> = catalogue
        .every()
        .iter()
        .map(|one| one.category().place())
        .collect();
    let in_order = places.clone();
    places.sort_unstable();
    assert_eq!(places, in_order);
}

/// **A catalogued application installs exactly as any other does**: the same
/// value, from its own source, granted nothing.
#[test]
fn what_is_catalogued_is_wanted_the_way_anything_is() {
    let catalogue = Catalogue::decided().unwrap();
    for one in catalogue.every() {
        let wanted = one.wanted();
        assert_eq!(wanted.application(), one.application());
        assert_eq!(one.source().as_str(), "flathub");
    }
}

/// **An application a machine already has is refused.** This is the check that
/// keeps the two decided lists one story: without it a market screen would
/// offer to install the browser a person is reading it in.
#[test]
fn an_application_the_machine_already_ships_is_refused() {
    let shipped = what_a_machine_ships();
    let already = with_one_more("pictures", "org.mozilla.firefox", "156.0", "MPL-2.0");
    assert_eq!(
        Catalogue::read(&already, &shipped).unwrap_err(),
        NotCatalogued::AlreadyShipped {
            identifier: "org.mozilla.firefox".to_owned()
        }
    );
    // And every one of the eight, not just the one that was easy to reach.
    for role in Role::EVERY {
        let identifier = shipped.the(role).application().identifier();
        // The terminal is refused for being a person's own first, which is a
        // different and stricter sentence — tested on its own below.
        if is_a_persons_own(identifier) {
            continue;
        }
        let text = with_one_more("video", identifier, "1.0", "MIT");
        assert_eq!(
            Catalogue::read(&text, &shipped).unwrap_err(),
            NotCatalogued::AlreadyShipped {
                identifier: identifier.to_owned()
            },
            "{}",
            role.named()
        );
    }
}

/// **The refusal depends on what the machine ships**, which is why the shipped
/// list is a parameter: read against a machine that ships something else, the
/// same catalogue holds.
#[test]
fn the_refusal_is_about_this_machine_and_not_about_the_name() {
    let ships_something_else = Shipped::read(&crate::shipped::THE_LIST.replacen(
        "identifier = \"org.gnome.Loupe\"",
        "identifier = \"org.gimp.GIMP\"",
        1,
    ));
    let shipped = ships_something_else.unwrap();
    // GIMP is shipped on that machine, so the catalogue may not offer it.
    assert_eq!(
        Catalogue::read(THE_CATALOGUE, &shipped).unwrap_err(),
        NotCatalogued::AlreadyShipped {
            identifier: "org.gimp.GIMP".to_owned()
        }
    );
    // And on the machine we do ship, the same text holds.
    assert!(Catalogue::read(THE_CATALOGUE, &what_a_machine_ships()).is_ok());
}

/// **A terminal is refused**, because there is no category for one and the
/// shipped list is where a reader looks for it (ADR 0043).
#[test]
fn a_persons_own_application_is_refused() {
    let shipped = what_a_machine_ships();
    for terminal in alo_capability::A_PERSONS_OWN {
        let text = with_one_more("documents", terminal, "1.0", "GPL-2.0-or-later");
        assert_eq!(
            Catalogue::read(&text, &shipped).unwrap_err(),
            NotCatalogued::APersonsOwn {
                identifier: terminal.to_owned()
            },
            "{terminal}"
        );
    }
}

/// **A category with nothing in it is refused**, naming it — a shelf a market
/// screen would draw empty.
#[test]
fn a_category_with_nothing_in_it_is_refused() {
    let shipped = what_a_machine_ships();
    for category in Category::EVERY {
        let without = THE_CATALOGUE
            .split("[[application]]")
            .filter(|part| !part.contains(&format!("category = \"{}\"", category.named())))
            .collect::<Vec<_>>()
            .join("[[application]]");
        assert_eq!(
            Catalogue::read(&without, &shipped).unwrap_err(),
            NotCatalogued::NothingUnder { category },
            "{}",
            category.named()
        );
    }
}

/// **One application twice is refused**, whether under one category or two.
#[test]
fn an_application_offered_twice_is_refused() {
    let shipped = what_a_machine_ships();
    let twice_over = with_one_more("painting", "org.gimp.GIMP", "3.2.6", "GPL-3.0+");
    assert_eq!(
        Catalogue::read(&twice_over, &shipped).unwrap_err(),
        NotCatalogued::Twice {
            identifier: "org.gimp.GIMP".to_owned()
        }
    );
    let same_category = with_one_more("pictures", "org.gimp.GIMP", "3.2.6", "GPL-3.0+");
    assert_eq!(
        Catalogue::read(&same_category, &shipped).unwrap_err(),
        NotCatalogued::Twice {
            identifier: "org.gimp.GIMP".to_owned()
        }
    );
}

#[test]
fn a_format_this_reader_does_not_understand_is_refused() {
    let shipped = what_a_machine_ships();
    let later = replacing("format = 1", "format = 2");
    assert_eq!(
        Catalogue::read(&later, &shipped).unwrap_err(),
        NotCatalogued::Format { format: 2 }
    );
}

/// **A key this reader does not know is refused rather than ignored**, so a
/// line meaning *recommended* cannot sit in the file promising something.
#[test]
fn a_key_nothing_reads_is_refused() {
    let shipped = what_a_machine_ships();
    let promising = replacing(
        "identifier = \"org.gimp.GIMP\"",
        "identifier = \"org.gimp.GIMP\"\nrecommended = true",
    );
    let why = Catalogue::read(&promising, &shipped).unwrap_err();
    assert!(
        matches!(why, NotCatalogued::NotRead { .. }),
        "an unknown key was not refused: {why:?}"
    );
    let unknown_category = replacing("category = \"video\"", "category = \"games\"");
    assert!(matches!(
        Catalogue::read(&unknown_category, &shipped).unwrap_err(),
        NotCatalogued::NotRead { .. }
    ));
}

#[test]
fn an_identifier_no_tool_could_be_handed_is_refused() {
    let shipped = what_a_machine_ships();
    for bad in ["", "-delete-everything", "not an identifier", "../../etc"] {
        let text = replacing(
            "identifier = \"org.kde.kdenlive\"",
            &format!("identifier = \"{bad}\""),
        );
        assert_eq!(
            Catalogue::read(&text, &shipped).unwrap_err(),
            NotCatalogued::NotAnIdentifier {
                category: Category::Video,
                identifier: bad.to_owned()
            },
            "{bad:?}"
        );
    }
}

#[test]
fn a_place_that_is_not_a_sources_name_is_refused() {
    let shipped = what_a_machine_ships();
    let nowhere = replacing("source = \"flathub\"", "source = \"--add-remote\"");
    assert_eq!(
        Catalogue::read(&nowhere, &shipped).unwrap_err(),
        NotCatalogued::NotASource {
            identifier: "org.gimp.GIMP".to_owned(),
            place: "--add-remote".to_owned()
        }
    );
}

#[test]
fn a_version_that_is_not_a_release_and_a_licence_that_is_not_one_are_refused() {
    let shipped = what_a_machine_ships();
    let unpinned = replacing("version = \"3.2.8\"", "version = \"latest\"");
    assert_eq!(
        Catalogue::read(&unpinned, &shipped).unwrap_err(),
        NotCatalogued::NotAVersion {
            identifier: "org.gimp.GIMP".to_owned(),
            version: "latest".to_owned()
        }
    );
    let unlicensed = replacing("licence = \"GPL-3.0+ AND LGPL-3.0+\"", "licence = \"\"");
    assert_eq!(
        Catalogue::read(&unlicensed, &shipped).unwrap_err(),
        NotCatalogued::NoLicence {
            identifier: "org.gimp.GIMP".to_owned()
        }
    );
}

/// **A release and a licence are taken as Flathub writes them**, and the
/// licence is read rather than copied: proprietary terms come through as not
/// open source, with the address of the terms kept as written.
#[test]
fn a_release_and_a_licence_as_flathub_writes_them_are_read() {
    let shipped = what_a_machine_ships();
    let text = with_one_more(
        "audio",
        "com.example.Player",
        "1.2.95.453.g0eeebbed",
        "LicenseRef-proprietary=https://example.org/terms",
    );
    let catalogue = Catalogue::read(&text, &shipped).unwrap();
    let player = catalogue
        .every()
        .iter()
        .find(|one| one.application().identifier() == "com.example.Player")
        .unwrap();
    assert_eq!(player.version(), "1.2.95.453.g0eeebbed");
    assert_eq!(
        player.licence(),
        "LicenseRef-proprietary=https://example.org/terms"
    );
    assert!(!player.is_open_source());
    let gimp = catalogue
        .every()
        .iter()
        .find(|one| one.application().identifier() == "org.gimp.GIMP")
        .unwrap();
    assert!(gimp.is_open_source());
}

/// **What the looser rules still refuse, through the catalogue**: a reference
/// to terms nobody here has read, an address that is not `https`, and a
/// release that is a word.
#[test]
fn a_licence_reference_nobody_read_and_a_word_for_a_release_are_still_refused() {
    let shipped = what_a_machine_ships();
    for licence in [
        "LicenseRef-Discord",
        "LicenseRef-proprietary=http://example.org",
        "MIT Apache-2.0",
    ] {
        let text = with_one_more("audio", "com.example.Player", "1.0", licence);
        assert_eq!(
            Catalogue::read(&text, &shipped).unwrap_err(),
            NotCatalogued::NoLicence {
                identifier: "com.example.Player".to_owned()
            },
            "{licence}"
        );
    }
    for version in ["stable", "-1", "1 .2"] {
        let text = with_one_more("audio", "com.example.Player", version, "MIT");
        assert_eq!(
            Catalogue::read(&text, &shipped).unwrap_err(),
            NotCatalogued::NotAVersion {
                identifier: "com.example.Player".to_owned(),
                version: version.to_owned()
            },
            "{version}"
        );
    }
}

/// **Every licence is an open one a person keeps, or is labelled as not one.**
///
/// Until 2026-11-17 this refused any proprietary licence outright. The owner
/// decided that day that proprietary applications are listed, *labelled 'not
/// open source'* — so the promise that survives is the one a person can check:
/// an entry is either under a licence family that lets them keep using it, or
/// its row says it is not open source. Nothing proprietary passes unlabelled.
#[test]
fn every_licence_is_one_a_person_keeps_or_is_labelled_not_open_source() {
    let catalogue = Catalogue::decided().unwrap();
    let mut open = 0;
    let mut labelled = 0;
    for one in catalogue.every() {
        let licence = one.licence();
        if one.is_open_source() {
            assert!(
                ["GPL", "MPL", "MIT", "Apache", "BSD"]
                    .iter()
                    .any(|family| licence.contains(family)),
                "{} is offered under {licence} and not labelled",
                one.application().identifier()
            );
            open += 1;
        } else {
            assert!(
                one.labels().contains(&Label::NotOpenSource),
                "{} is proprietary and its row would not say so",
                one.application().identifier()
            );
            labelled += 1;
        }
    }
    assert_eq!(
        open + labelled,
        catalogue.how_many(),
        "the loop read every entry"
    );
    assert!(
        labelled > 0,
        "the premise: the catalogue lists proprietary applications"
    );
}

/// **There is no English in the data.** Not a name, not a summary — those
/// belong to the source, already translated, and a string here would stand in
/// twenty-four languages' way.
#[test]
fn the_catalogue_holds_no_words_a_person_would_read() {
    let mut read = 0;
    for line in THE_CATALOGUE.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') || line.starts_with("[[") {
            continue;
        }
        // One assertion for both ways this fails — a key nothing reads, and a
        // line that is not a key at all. Written this way round because the
        // `else` branch of a `let`-`else` can only panic, and this crate does
        // not panic, in a test or anywhere else.
        let key = line.split_once(" = ").map(|(key, _)| key);
        assert!(
            matches!(
                key,
                Some(
                    "format"
                        | "category"
                        | "identifier"
                        | "source"
                        | "version"
                        | "licence"
                        | "verified"
                        | "early"
                )
            ),
            "the catalogue holds {line:?}, which this reader does not know — and if it is \
             something a person reads, it belongs in crate::words"
        );
        read += 1;
    }
    assert_eq!(
        read, 435,
        "format, and seven keys for each of sixty-two entries"
    );
}
