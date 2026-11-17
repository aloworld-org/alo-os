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
use crate::installing_a_signed_release::NotASignedRelease;
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
         \"{identifier}\"\nname = \"An application\"\nsource = \"flathub\"\nversion = \"{version}\"\nlicence = \
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
         \"com.example.All\"\nname = \"All of them\"\nsource = \"flathub\"\nversion = \"1.0\"\nlicence = \
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
    // GIMP asks for the whole machine's files (Flathub, 2026-11-17), and
    // darktable as well, and is not verified.
    assert_eq!(labels_of("org.gimp.GIMP"), vec![Label::ReachesYourFiles]);
    assert_eq!(
        labels_of("org.darktable.Darktable"),
        vec![Label::ReachesYourFiles, Label::NotVerified]
    );
}

/// **The name a person reads is the makers' own, and an entry without one,
/// or with one that is not a name, is refused** — so the market never falls
/// back to drawing an identifier (the owner, 2026-11-17).
#[test]
fn an_entry_is_read_with_its_name_and_one_without_a_name_is_refused() {
    let shipped = what_a_machine_ships();
    let catalogue = Catalogue::read(THE_CATALOGUE, &shipped).unwrap();
    let gimp = catalogue
        .every()
        .iter()
        .find(|one| one.application().identifier() == "org.gimp.GIMP")
        .unwrap();
    assert_eq!(gimp.name(), "GNU Image Manipulation Program");
    let named = "name = \"GNU Image Manipulation Program\"\n";
    let too_long = format!("name = \"{}\"\n", "a".repeat(THE_LONGEST_NAME + 1));
    for instead in [
        String::new(),
        "name = \"\"\n".to_owned(),
        "name = \"   \"\n".to_owned(),
        "name = \"GIMP\\nand more\"\n".to_owned(),
        too_long,
    ] {
        let text = replacing(named, &instead);
        assert!(
            matches!(
                Catalogue::read(&text, &shipped),
                Err(NotCatalogued::NotRead { .. } | NotCatalogued::NotAName { .. })
            ),
            "{instead:?} was read"
        );
    }
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
    assert_eq!(catalogue.how_many(), 69);
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
        // **From Flathub, or from its maker's signed release**, and never
        // both: a maker's entry is the one with a release to check (ADR 0158).
        if one.signed_release().is_some() {
            assert_eq!(one.source().as_str(), "storytold");
        } else {
            assert_eq!(one.source().as_str(), "flathub");
        }
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
    let unknown_category = replacing("category = \"video\"", "category = \"gardening\"");
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
                        | "name"
                        | "source"
                        | "version"
                        | "licence"
                        | "verified"
                        | "early"
                        | "bundle"
                        | "sha256"
                        | "reaches"
                )
            ),
            "the catalogue holds {line:?}, which this reader does not know — and if it is \
             something a person reads, it belongs in crate::words"
        );
        read += 1;
    }
    assert_eq!(
        read, 567,
        "format, seven keys for each of sixty-two entries from Flathub, and nine for each of \
         seven from their maker's signed release"
    );
}

/// **A Craft application is read as its maker's signed release**: early, with
/// no automatic updates, published by its makers, and pinned to a bundle on
/// their own release page with its checksum.
#[test]
fn a_craft_application_is_read_as_its_makers_signed_release() {
    let catalogue = Catalogue::decided().unwrap();
    let craft: Vec<&Listed> = catalogue
        .every()
        .iter()
        .filter(|one| one.signed_release().is_some())
        .collect();
    assert_eq!(craft.len(), 7, "the seven the owner named");
    for one in craft {
        let release = one.signed_release().unwrap();
        assert_eq!(release.maker().name(), "storytold");
        assert_eq!(release.application(), one.application());
        assert!(release.bundle().ends_with("-linux-x86_64.flatpak"));
        assert_eq!(
            one.labels(),
            vec![
                Label::RunsProgramsOutsideItsBox,
                Label::EarlyVersion,
                Label::NoAutomaticUpdates
            ],
            "{}",
            one.application().identifier()
        );
    }
    let gimp = catalogue
        .every()
        .iter()
        .find(|one| one.application().identifier() == "org.gimp.GIMP")
        .unwrap();
    assert!(gimp.signed_release().is_none());
}

/// **What a signed release in the file may not be**: a maker with no bundle,
/// a maker's bundle on somebody else's page, a checksum that is not one, and a
/// bundle on an entry from a place.
#[test]
fn a_signed_release_that_does_not_hold_is_refused() {
    let shipped = what_a_machine_ships();
    let one_more = |source: &str, rest: &str| {
        format!(
            "{THE_CATALOGUE}\n[[application]]\ncategory = \"audio\"\nidentifier = \
             \"ai.storyteller.example\"\nsource = \"{source}\"\nversion = \"0.1.0\"\nlicence = \
             \"Apache-2.0\"\nverified = true\nearly = true\nreaches = []\n{rest}"
        )
    };
    let sum = "5c9deb58f07bd54f323a5d985bed295b4c5b15789e02651e2f203d5e8ef211a5";
    let theirs = "https://github.com/storytold/x/releases/download/v1/x.flatpak";
    let somebody_elses = "https://github.com/someone/x/releases/download/v1/x.flatpak";
    for (text, why) in [
        (one_more("storytold", ""), NotASignedRelease::NotTheMakers),
        (
            one_more("storytold", &format!("bundle = \"{theirs}\"\n")),
            NotASignedRelease::NotAChecksum,
        ),
        (
            one_more(
                "storytold",
                &format!("bundle = \"{somebody_elses}\"\nsha256 = \"{sum}\"\n"),
            ),
            NotASignedRelease::NotTheMakers,
        ),
        (
            one_more(
                "storytold",
                &format!("bundle = \"{theirs}\"\nsha256 = \"not-a-sum\"\n"),
            ),
            NotASignedRelease::NotAChecksum,
        ),
    ] {
        assert_eq!(
            Catalogue::read(&text, &shipped).unwrap_err(),
            NotCatalogued::NotASignedRelease {
                identifier: "ai.storyteller.example".to_owned(),
                why
            }
        );
    }
    let from_a_place = one_more(
        "flathub",
        &format!("bundle = \"{theirs}\"\nsha256 = \"{sum}\"\n"),
    );
    assert_eq!(
        Catalogue::read(&from_a_place, &shipped).unwrap_err(),
        NotCatalogued::NotFromAMaker {
            identifier: "ai.storyteller.example".to_owned()
        }
    );
}

/// **What its makers ask for past its box is on its row, read from its real
/// metadata** (the owner, 2026-11-17): OBS Studio can run programs outside its
/// box, GIMP reaches the person's files, and an application asking for neither
/// carries neither — the three cases a row can be in.
#[test]
fn what_an_application_reaches_past_its_box_is_on_its_row() {
    let catalogue = Catalogue::decided().unwrap();
    let labels_of = |identifier: &str| {
        catalogue
            .every()
            .iter()
            .find(|one| one.application().identifier() == identifier)
            .unwrap()
            .labels()
    };
    let obs = labels_of("com.obsproject.Studio");
    assert!(obs.contains(&Label::RunsProgramsOutsideItsBox), "{obs:?}");
    assert!(
        !obs.contains(&Label::ReachesYourFiles),
        "the larger reach is said, and the lesser not said twice: {obs:?}"
    );
    assert_eq!(labels_of("org.gimp.GIMP"), vec![Label::ReachesYourFiles]);
    let quiet = catalogue
        .every()
        .iter()
        .find(|one| one.reaches().is_empty())
        .unwrap();
    assert!(
        !quiet.labels().iter().any(|label| matches!(
            label,
            Label::RunsProgramsOutsideItsBox | Label::ReachesYourFiles
        )),
        "{} asks for nothing past its box and is labelled as if it did",
        quiet.application().identifier()
    );
}

/// **An entry that does not say what it reaches is refused**, and one naming
/// a reach this reader does not know — so a forgotten field is a refused file,
/// never a row that says nothing about what it can do.
#[test]
fn an_entry_that_does_not_say_what_it_reaches_is_refused() {
    let shipped = what_a_machine_ships();
    let gimps = "early = false\nreaches = [\"your-files\"]\n";
    assert!(THE_CATALOGUE.contains(gimps));
    for instead in [
        "early = false\n",
        "early = false\nreaches = [\"the-camera\"]\n",
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

/// **Installed, an application that reaches past its box is not told it was
/// given nothing**; one that asks for nothing past it, or one nothing here
/// knows, keeps the ordinary sentence.
#[test]
fn what_arrives_with_access_past_its_box_is_not_told_it_was_given_nothing() {
    use crate::installing::Installed;
    let english = crate::testing::in_english();
    let gimp = alo_applications::Application::identified("org.gimp.GIMP").unwrap();
    let reaching = Installed::sandboxed(gimp.clone(), Some(vec![crate::Reach::YourFiles]));
    let said = reaching.said(&english).into_text();
    assert!(!said.contains("given nothing"), "{said}");
    assert!(said.contains("more than its box"), "{said}");
    for quiet in [
        Installed::sandboxed(gimp.clone(), Some(Vec::new())),
        Installed::sandboxed(gimp, None),
    ] {
        assert!(quiet.said(&english).into_text().contains("given nothing"));
    }
}
