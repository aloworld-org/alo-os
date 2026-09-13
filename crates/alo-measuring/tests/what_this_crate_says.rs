//! What this crate says, against the vocabulary the code actually uses.
//!
//! The unit tests read the list; this reads what a person would see, in a
//! language that is not the one the code is written in. Polish, because the
//! countable sentences here — *counted together with N other processes*, *the
//! count stopped after N things* — have three forms for whole numbers in
//! Polish where English has two, and a sentence assembled out of English
//! pieces would show it at five.

#![expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::path::PathBuf;

use alo_measuring::{
    Counted, Gone, Holding, Kind, Network, Node, NotMeasured, Number, Source, measuring_words,
};
use alo_strings::{Form, Key, Language, Said, Strings, Translation};

/// A key, from this crate's list.
fn key(named: &str) -> Key {
    Key::named(named).expect("a key")
}

/// Polish, and this crate's seventeen strings in it.
fn in_polish() -> Strings {
    let vocabulary = measuring_words().expect("this crate's own words");
    let polish = Language::written("pl").expect("a language");
    let shared = key("measuring.network.shared");
    let cut_short = key("measuring.filling.cut-short");
    let unnamed = key("measuring.filling.unnamed");
    let translation = Translation::into_language(polish.clone())
        .says(
            key("measuring.not-on-this-host"),
            "Co jest uruchomione, można odczytać tylko na samym alo OS.",
        )
        .says(
            key("measuring.unreadable"),
            "Nie udało się odczytać, co jest uruchomione, z {at}.",
        )
        .says(
            key("measuring.no-interval"),
            "Dwa odczyty wymagają czasu między nimi.",
        )
        .says(
            key("measuring.same-moment"),
            "Oba odczyty pochodzą z tej samej chwili, więc nie można nic powiedzieć o tempie.",
        )
        .says(
            key("measuring.process.gone"),
            "Zakończony od ostatniego odczytu.",
        )
        .says(
            key("measuring.number.not-yet"),
            "Uruchomiony od ostatniego odczytu.",
        )
        .says(key("measuring.number.withheld"), "Jądro tego nie pokazało.")
        .says(
            key("measuring.number.not-said"),
            "Nie liczone dla tego procesu.",
        )
        .says(
            key("measuring.network.this-process-alone"),
            "Ruch sieciowy liczony tylko dla tego procesu.",
        )
        .says(
            shared.for_form(Form::One),
            "Ruch sieciowy liczony razem z jednym innym procesem.",
        )
        .says(
            shared.for_form(Form::Few),
            "Ruch sieciowy liczony razem z {others} innymi procesami.",
        )
        .says(
            shared.for_form(Form::Many),
            "Ruch sieciowy liczony razem z {others} innymi procesami.",
        )
        .says(
            key("measuring.filling.counted-elsewhere"),
            "Policzone pod inną ze swoich nazw, {at}.",
        )
        .says(
            key("measuring.filling.not-read"),
            "Nie udało się odczytać, więc nic w środku nie jest policzone.",
        )
        .says(
            key("measuring.filling.another-filesystem"),
            "Na innym systemie plików, więc to, co w środku, nie jest tu liczone.",
        )
        .says(
            key("measuring.filling.not-finished"),
            "Liczenie zatrzymało się, zanim zobaczono wszystko w środku.",
        )
        .says(
            key("measuring.filling.not-counted"),
            "Nie udało się policzyć, co wypełnia {at}. {why}",
        )
        .says(
            cut_short.for_form(Form::One),
            "Liczenie zatrzymało się po jednej rzeczy, więc te rozmiary nie są całością.",
        )
        .says(
            cut_short.for_form(Form::Few),
            "Liczenie zatrzymało się po {most} rzeczach, więc te rozmiary nie są całością.",
        )
        .says(
            cut_short.for_form(Form::Many),
            "Liczenie zatrzymało się po {most} rzeczach, więc te rozmiary nie są całością.",
        )
        .says(
            unnamed.for_form(Form::One),
            "Jedna rzecz, której nazwy nie można pokazać, nie jest policzona.",
        )
        .says(
            unnamed.for_form(Form::Few),
            "{unnamed} rzeczy, których nazw nie można pokazać, nie są policzone.",
        )
        .says(
            unnamed.for_form(Form::Many),
            "{unnamed} rzeczy, których nazw nie można pokazać, nie są policzone.",
        );
    let speaking = vocabulary
        .check(translation)
        .expect("a translation of seventeen strings with no gaps in it");
    let mut strings = Strings::of(vocabulary);
    strings.speaks(speaking).expect("a checked translation");
    strings.prefers(&[polish]);
    strings
}

/// A source for these tests.
fn from() -> Source {
    Source::of("/proc/42/io", "rchar")
}

/// A count of a folder, finished or not, with things unnamed or not.
fn a_holding(finished: bool, unnamed: usize) -> Holding {
    Holding {
        folder: PathBuf::from("Documents"),
        tree: Node {
            name: "Documents".to_owned(),
            at: PathBuf::from("Documents"),
            kind: Kind::Folder,
            own: 0,
            size: 0,
            counted: Counted::Whole,
            children: Vec::new(),
        },
        finished,
        most: 20_000,
        unnamed,
    }
}

/// Every sentence this crate says itself, rendered against these strings.
///
/// Every sentence but one: *what is filling this folder could not be
/// counted* carries `alo-files`' own sentence inside it, and
/// [`the_one_refusal_worded_by_the_file_half_carries_its_sentence`] reads
/// that one against both crates' words.
fn everything_said_here(strings: &Strings) -> Vec<Said> {
    let mut said = vec![
        NotMeasured::NotOnThisHost.said(strings),
        NotMeasured::Unreadable {
            at: PathBuf::from("/proc/stat"),
            why: "no such file".to_owned(),
        }
        .said(strings),
        NotMeasured::NoInterval.said(strings),
        NotMeasured::SameMoment.said(strings),
        Gone {
            pid: 42,
            name: "cat".to_owned(),
        }
        .said(strings),
        Network {
            namespace: Some(1),
            shared_with: 0,
        }
        .said(strings),
        Network {
            namespace: Some(1),
            shared_with: 1,
        }
        .said(strings),
        Network {
            namespace: Some(1),
            shared_with: 3,
        }
        .said(strings),
        Network {
            namespace: Some(1),
            shared_with: 5,
        }
        .said(strings),
    ];
    for number in [
        Number::Withheld {
            from: from(),
            why: String::new(),
        },
        Number::NotSaid { from: from() },
        Number::NotYet { from: from() },
    ] {
        said.push(
            number
                .instead(strings)
                .expect("a sentence in place of a number"),
        );
    }
    for counted in [
        Counted::Elsewhere {
            at: PathBuf::from("one.txt"),
        },
        Counted::NotRead {
            why: "permission denied".to_owned(),
        },
        Counted::OnAnotherFilesystem,
        Counted::NotFinished,
    ] {
        said.push(counted.said(strings).expect("a sentence beside a size"));
    }
    for unnamed in [1, 3, 22] {
        let holding = a_holding(false, unnamed);
        said.push(
            holding
                .not_the_whole(strings)
                .expect("a sentence above a count that stopped"),
        );
        said.push(
            holding
                .left_unnamed(strings)
                .expect("a sentence above a count that left names out"),
        );
    }
    said
}

/// A machine with no translations shows the English, and says so about
/// itself; nothing is a key on somebody's screen and nothing has a gap.
#[test]
fn a_machine_with_no_translations_still_says_everything_in_english() {
    let strings = Strings::of(measuring_words().expect("this crate's own words"));
    let said = everything_said_here(&strings);
    assert_eq!(said.len(), 22);
    for said in said {
        assert!(!said.is_a_bug(), "{said}");
        assert!(!said.is_translated(), "{said}");
        assert!(said.unfilled().is_empty(), "{said}");
        assert!(!said.text().starts_with("measuring."), "{said}");
    }
    assert!(Counted::Whole.said(&strings).is_none());
    assert!(a_holding(true, 0).not_the_whole(&strings).is_none());
    assert!(a_holding(true, 0).left_unnamed(&strings).is_none());
}

/// And a machine that has them shows those, whole — with the Polish forms
/// chosen by Polish rules: one, a few, and many.
#[test]
fn every_sentence_is_read_in_the_language_the_person_reads() {
    let strings = in_polish();
    for said in everything_said_here(&strings) {
        assert!(said.is_translated(), "{said}");
        assert!(!said.is_a_bug(), "{said}");
        assert!(said.unfilled().is_empty(), "{said}");
    }

    let unreadable = NotMeasured::Unreadable {
        at: PathBuf::from("/proc/stat"),
        why: String::new(),
    }
    .said(&strings);
    assert_eq!(
        unreadable.text(),
        "Nie udało się odczytać, co jest uruchomione, z /proc/stat."
    );

    let one = Network {
        namespace: None,
        shared_with: 1,
    }
    .said(&strings);
    assert_eq!(
        one.text(),
        "Ruch sieciowy liczony razem z jednym innym procesem."
    );
    let few = Network {
        namespace: None,
        shared_with: 3,
    }
    .said(&strings);
    assert_eq!(
        few.text(),
        "Ruch sieciowy liczony razem z 3 innymi procesami."
    );
    let many = Network {
        namespace: None,
        shared_with: 22,
    }
    .said(&strings);
    assert_eq!(
        many.text(),
        "Ruch sieciowy liczony razem z 22 innymi procesami."
    );

    let elsewhere = Counted::Elsewhere {
        at: PathBuf::from("one.txt"),
    }
    .said(&strings)
    .expect("a sentence beside a size");
    assert_eq!(
        elsewhere.text(),
        "Policzone pod inną ze swoich nazw, one.txt."
    );
    let a_few_unnamed = a_holding(false, 3)
        .left_unnamed(&strings)
        .expect("a sentence above the count");
    assert_eq!(
        a_few_unnamed.text(),
        "3 rzeczy, których nazw nie można pokazać, nie są policzone."
    );
}

/// **The one refusal worded partly by the file half carries its sentence,
/// not its key.** Against both crates' words it is a sentence a person
/// reads; against this crate's alone the inner half is a key, marked as one,
/// which is what a shell that forgot to collect `alo-files`' words would
/// show — and is not English nobody offered to translate.
#[test]
fn the_one_refusal_worded_by_the_file_half_carries_its_sentence() {
    let not_counted = NotMeasured::NotCounted {
        at: PathBuf::from("Documents"),
        why: alo_files::Failed::Gone {
            path: "Documents".to_owned(),
        },
    };

    let mut both = measuring_words().expect("this crate's own words");
    alo_files::words::declare_into(&mut both).expect("the file half's words");
    let said = not_counted.said(&Strings::of(both));
    assert!(!said.is_a_bug(), "{said}");
    assert!(said.unfilled().is_empty(), "{said}");
    assert!(said.text().contains("Documents"), "{said}");
    assert!(!said.text().contains("files."), "{said}");

    let alone = not_counted.said(&Strings::of(
        measuring_words().expect("this crate's own words"),
    ));
    assert!(alone.is_a_bug(), "{alone}");
}
