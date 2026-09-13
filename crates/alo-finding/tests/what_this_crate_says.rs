//! What this crate says, against the vocabulary the code actually uses.
//!
//! The unit tests read the list; this reads what a person would see, in a
//! language that is not the one the code is written in. Polish, because the
//! countable sentences here — *the index stopped after N things*, *N things
//! whose names cannot be shown* — have three forms for whole numbers in
//! Polish where English has two, and a sentence assembled out of English
//! pieces would show it at five.

#![expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::path::PathBuf;

use alo_finding::{
    Contents, Covered, Entry, Kind, Moment, NotAsked, NotIndexed, NotSearched, Unread,
    finding_words,
};
use alo_strings::{Form, Key, Language, Said, Strings, Translation};

/// A key, from this crate's list.
fn key(named: &str) -> Key {
    Key::named(named).expect("a key")
}

/// Polish, and this crate's thirty-five strings in it.
fn in_polish() -> Strings {
    let vocabulary = finding_words().expect("this crate's own words");
    let polish = Language::written("pl").expect("a language");
    let not_whole = key("finding.not-whole");
    let unnamed = key("finding.unnamed");
    let no_reader = key("finding.not-searched.no-reader");
    let too_big = key("finding.not-searched.too-big");
    let translation = Translation::into_language(polish.clone())
        .says(
            key("finding.not-asked.nothing"),
            "Nie zapytano o nic, więc nic nie przeszukano: wyszukiwanie potrzebuje nazwy, \
             rodzaju, daty albo kilku słów.",
        )
        .says(
            key("finding.not-asked.more-than-a-sentence"),
            "{words} słów to więcej niż zdanie, a wyszukiwanie przyjmuje najwyżej {most}.",
        )
        .says(
            key("finding.not-asked.longer-than-a-name"),
            "{chars} znaków to więcej niż jakakolwiek nazwa pliku, a wyszukiwanie przyjmuje \
             najwyżej {most}.",
        )
        .says(
            key("finding.not-searched.outside"),
            "Niczego poza {folder} nie przeszukano.",
        )
        .says(
            key("finding.not-searched.folder-unread"),
            "Nie udało się odczytać {below}, więc niczego w nim nie przeszukano: {why}",
        )
        .says(
            key("finding.not-searched.elsewhere"),
            "{below} jest na innym dysku, więc niczego w nim nie przeszukano.",
        )
        .says(
            key("finding.not-searched.not-entered"),
            "Indeks zatrzymał się, zanim skończył {below}, więc nie przeszukano go w całości.",
        )
        .says(
            key("finding.not-searched.file-unread"),
            "Nie udało się odczytać {below}, więc go nie przeszukano: {why}",
        )
        .says(
            no_reader.for_form(Form::One),
            "Jeden plik jest rodzaju, którego słów nie można odczytać, więc nie przeszukano \
             go po słowach.",
        )
        .says(
            no_reader.for_form(Form::Few),
            "{files} pliki są rodzajów, których słów nie można odczytać, więc nie przeszukano \
             ich po słowach.",
        )
        .says(
            no_reader.for_form(Form::Many),
            "{files} plików jest rodzajów, których słów nie można odczytać, więc nie \
             przeszukano ich po słowach.",
        )
        .says(
            too_big.for_form(Form::One),
            "Jeden plik jest większy, niż indeks czyta, więc nie przeszukano go po słowach.",
        )
        .says(
            too_big.for_form(Form::Few),
            "{files} pliki są większe, niż indeks czyta, więc nie przeszukano ich po słowach.",
        )
        .says(
            too_big.for_form(Form::Many),
            "{files} plików jest większych, niż indeks czyta, więc nie przeszukano ich po \
             słowach.",
        )
        .says(
            key("finding.not-absolute"),
            "{at} nie mówi, gdzie jest od korzenia maszyny, więc nie można go zindeksować.",
        )
        .says(
            key("finding.not-walked"),
            "Nie udało się zindeksować {at}. {why}",
        )
        .says(
            key("finding.not-the-same-folder"),
            "Ten indeks dotyczy {indexed}, a nie {asked}.",
        )
        .says(
            key("finding.not-kept"),
            "Nie udało się zapisać indeksu do {at}: {why}",
        )
        .says(
            key("finding.not-opened"),
            "Nie udało się odczytać indeksu z {at}: {why}",
        )
        .says(
            key("finding.not-an-index"),
            "{at} nie jest indeksem, który ta maszyna potrafi odczytać: {why}",
        )
        .says(
            key("finding.nowhere-to-keep-it"),
            "Nie ma katalogu domowego, w którym można by trzymać indeks.",
        )
        .says(
            key("finding.contents.not-text"),
            "Nie jest to rodzaj pliku, którego słowa można odczytać.",
        )
        .says(
            key("finding.contents.not-read"),
            "Nie udało się odczytać jego słów: {why}",
        )
        .says(
            key("finding.contents.too-big"),
            "Większy niż {most} bajtów, które indeks czyta, więc jego słowa nie zostały odczytane.",
        )
        .says(key("finding.kind.text"), "Tekst")
        .says(key("finding.kind.pdf"), "Dokument PDF")
        .says(key("finding.kind.png"), "Obraz PNG")
        .says(key("finding.kind.jpeg"), "Obraz JPEG")
        .says(key("finding.kind.gif"), "Obraz GIF")
        .says(key("finding.kind.zip"), "Archiwum zip")
        .says(key("finding.kind.program"), "Program")
        .says(
            key("finding.kind.bytes"),
            "Plik rodzaju, którego ta maszyna nie zna",
        )
        .says(key("finding.kind.empty"), "Pusty plik")
        .says(
            key("finding.kind.unread"),
            "Plik, którego nie udało się odczytać",
        )
        .says(key("finding.kind.folder"), "Folder")
        .says(key("finding.kind.link"), "Dowiązanie")
        .says(
            key("finding.kind.other"),
            "Coś, co nie jest ani plikiem, ani folderem",
        )
        .says(
            not_whole.for_form(Form::One),
            "Indeks zatrzymał się po jednej rzeczy, więc nie obejmuje całego folderu.",
        )
        .says(
            not_whole.for_form(Form::Few),
            "Indeks zatrzymał się po {most} rzeczach, więc nie obejmuje całego folderu.",
        )
        .says(
            not_whole.for_form(Form::Many),
            "Indeks zatrzymał się po {most} rzeczach, więc nie obejmuje całego folderu.",
        )
        .says(
            unnamed.for_form(Form::One),
            "Jedna rzecz, której nazwy nie można pokazać, nie jest zindeksowana.",
        )
        .says(
            unnamed.for_form(Form::Few),
            "{unnamed} rzeczy, których nazw nie można pokazać, nie są zindeksowane.",
        )
        .says(
            unnamed.for_form(Form::Many),
            "{unnamed} rzeczy, których nazw nie można pokazać, nie są zindeksowane.",
        );
    let speaking = vocabulary
        .check(translation)
        .expect("a translation of thirty-five strings with no gaps in it");
    let mut strings = Strings::of(vocabulary);
    strings.speaks(speaking).expect("a checked translation");
    strings.prefers(&[polish]);
    strings
}

/// What the walk did not reach, for these tests.
fn a_covered(whole: bool, unnamed: usize) -> Covered {
    Covered {
        whole,
        most: 20_000,
        unread: Vec::new(),
        elsewhere: Vec::new(),
        not_entered: Vec::new(),
        unnamed,
    }
}

/// An entry, for what a search says it did not look at.
fn an_entry(below: &str, kind: Kind, contents: Contents) -> Entry {
    Entry {
        below: below.to_owned(),
        kind,
        bytes: 1,
        modified: Moment { secs: 1, nanos: 0 },
        contents,
    }
}

/// What a search did not look at, said in full: one of each, and the two
/// counts at one, a few and many.
fn everything_a_search_did_not_look_at(strings: &Strings) -> Vec<Said> {
    let folder = PathBuf::from("/home/ada/Documents");
    let unread = Unread {
        below: "private".to_owned(),
        why: "permission denied".to_owned(),
    };
    let locked = an_entry(
        "locked.txt",
        Kind::Unread,
        Contents::NotRead {
            why: "permission denied".to_owned(),
        },
    );
    let pdf = an_entry("march.pdf", Kind::Pdf, Contents::NotText);
    let big = an_entry("big.txt", Kind::Text, Contents::TooBig { bytes: 9 });
    let mut said = Vec::new();
    for how_many in [1, 3, 25] {
        let not_searched = NotSearched {
            outside: &folder,
            folders_unread: vec![&unread],
            elsewhere: vec!["mnt"],
            not_entered: vec![""],
            unnamed: 0,
            files_unread: vec![&locked],
            no_reader: vec![&pdf; how_many],
            too_big: vec![&big; how_many],
        };
        let sentences = not_searched.said(strings);
        assert_eq!(sentences.len(), 7, "{sentences:?}");
        said.extend(sentences);
    }
    said
}

/// Every sentence this crate says itself, rendered against these strings.
///
/// Every sentence but one: *could not be indexed* carries `alo-files`' own
/// sentence inside it, and
/// [`the_one_refusal_worded_by_the_file_half_carries_its_sentence`] reads
/// that one against both crates' words.
fn everything_said_here(strings: &Strings) -> Vec<Said> {
    let file = PathBuf::from("/home/ada/.local/share/alo/finding/x.index");
    let mut said = vec![
        NotIndexed::NotAbsolute {
            at: PathBuf::from("Documents"),
        }
        .said(strings),
        NotIndexed::NotTheSame {
            asked: PathBuf::from("/home/ada/Pictures"),
            indexed: PathBuf::from("/home/ada/Documents"),
        }
        .said(strings),
        NotIndexed::NotKept {
            at: file.clone(),
            why: "read-only".to_owned(),
        }
        .said(strings),
        NotIndexed::NotOpened {
            at: file.clone(),
            why: "no such file".to_owned(),
        }
        .said(strings),
        NotIndexed::NotAnIndex {
            at: file,
            why: "the file is empty".to_owned(),
        }
        .said(strings),
        NotIndexed::NowhereToKeepIt.said(strings),
    ];
    for contents in [
        Contents::NotText,
        Contents::NotRead {
            why: "permission denied".to_owned(),
        },
        Contents::TooBig { bytes: 5_000_000 },
    ] {
        said.push(
            contents
                .said(strings)
                .expect("a sentence in place of words"),
        );
    }
    for kind in Kind::EVERY {
        said.push(kind.said(strings));
    }
    for unnamed in [1, 3, 22] {
        let covered = a_covered(false, unnamed);
        said.push(
            covered
                .not_the_whole(strings)
                .expect("a sentence above an index that stopped"),
        );
        said.push(
            covered
                .left_unnamed(strings)
                .expect("a sentence above an index that left names out"),
        );
    }
    for not_asked in [
        NotAsked::Nothing,
        NotAsked::MoreThanASentence {
            words: 40,
            most: 32,
        },
        NotAsked::LongerThanAName {
            chars: 300,
            most: 255,
        },
    ] {
        said.push(not_asked.said(strings));
    }
    said.extend(everything_a_search_did_not_look_at(strings));
    said
}

/// A machine with no translations shows the English, and says so about
/// itself; nothing is a key on somebody's screen and nothing has a gap.
#[test]
fn a_machine_with_no_translations_still_says_everything_in_english() {
    let strings = Strings::of(finding_words().expect("this crate's own words"));
    let said = everything_said_here(&strings);
    assert_eq!(said.len(), 6 + 3 + 13 + 6 + 3 + 3 * 7);
    for said in said {
        assert!(!said.is_a_bug(), "{said}");
        assert!(!said.is_translated(), "{said}");
        assert!(said.unfilled().is_empty(), "{said}");
        assert!(!said.text().starts_with("finding."), "{said}");
    }
    assert!(
        Contents::Read { words: Vec::new() }
            .said(&strings)
            .is_none()
    );
    assert!(Contents::NotAFile.said(&strings).is_none());
    assert!(a_covered(true, 0).not_the_whole(&strings).is_none());
    assert!(a_covered(true, 0).left_unnamed(&strings).is_none());
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

    assert_eq!(Kind::Pdf.said(&strings).text(), "Dokument PDF");
    let too_big = Contents::TooBig { bytes: 5_000_000 }
        .said(&strings)
        .expect("a sentence in place of words");
    assert_eq!(
        too_big.text(),
        "Większy niż 1048576 bajtów, które indeks czyta, więc jego słowa nie zostały odczytane."
    );

    let one = a_covered(false, 1)
        .left_unnamed(&strings)
        .expect("a sentence above the index");
    assert_eq!(
        one.text(),
        "Jedna rzecz, której nazwy nie można pokazać, nie jest zindeksowana."
    );
    let few = a_covered(false, 3)
        .left_unnamed(&strings)
        .expect("a sentence above the index");
    assert_eq!(
        few.text(),
        "3 rzeczy, których nazw nie można pokazać, nie są zindeksowane."
    );
    let many = a_covered(false, 22)
        .not_the_whole(&strings)
        .expect("a sentence above the index");
    assert_eq!(
        many.text(),
        "Indeks zatrzymał się po 20000 rzeczach, więc nie obejmuje całego folderu."
    );

    let not_looked_at = everything_a_search_did_not_look_at(&strings);
    assert_eq!(
        not_looked_at.first().expect("a sentence").text(),
        "Niczego poza /home/ada/Documents nie przeszukano."
    );
    assert_eq!(
        not_looked_at.get(5).expect("a sentence").text(),
        "Jeden plik jest rodzaju, którego słów nie można odczytać, więc nie przeszukano go po \
         słowach."
    );
    assert_eq!(
        not_looked_at.get(7 + 6).expect("a sentence").text(),
        "3 pliki są większe, niż indeks czyta, więc nie przeszukano ich po słowach."
    );
    assert_eq!(
        not_looked_at.get(14 + 5).expect("a sentence").text(),
        "25 plików jest rodzajów, których słów nie można odczytać, więc nie przeszukano ich po \
         słowach."
    );
    assert_eq!(
        NotAsked::MoreThanASentence {
            words: 40,
            most: 32
        }
        .said(&strings)
        .text(),
        "40 słów to więcej niż zdanie, a wyszukiwanie przyjmuje najwyżej 32."
    );
}

/// **The one refusal worded partly by the file half carries its sentence,
/// not its key.** Against both crates' words it is a sentence a person
/// reads; against this crate's alone the inner half is a key, marked as one,
/// which is what a shell that forgot to collect `alo-files`' words would
/// show — and is not English nobody offered to translate.
#[test]
fn the_one_refusal_worded_by_the_file_half_carries_its_sentence() {
    let not_walked = NotIndexed::NotWalked {
        at: PathBuf::from("/home/ada/Documents"),
        why: alo_files::Failed::Gone {
            path: "/home/ada/Documents".to_owned(),
        },
    };

    let mut both = finding_words().expect("this crate's own words");
    alo_files::words::declare_into(&mut both).expect("the file half's words");
    let said = not_walked.said(&Strings::of(both));
    assert!(!said.is_a_bug(), "{said}");
    assert!(said.unfilled().is_empty(), "{said}");
    assert!(
        said.text()
            .starts_with("/home/ada/Documents could not be indexed."),
        "{said}"
    );
    assert!(!said.text().contains("files."), "{said}");

    let alone = not_walked.said(&Strings::of(
        finding_words().expect("this crate's own words"),
    ));
    assert!(alone.is_a_bug(), "{alone}");
}
