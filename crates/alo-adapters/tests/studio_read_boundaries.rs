//! Status reads preserve exact bytes, uncertainty and the granted application.

#![expect(clippy::unwrap_used, reason = "unexpected refusal fails the test")]

use std::cell::Cell;
use std::time::{Duration, SystemTime};

use alo_adapters::{Delivers, Driven, Driving, Message, NotDelivered};
use alo_applications::{Application, Installed};
use alo_capability::{Authorised, Grant, Grantee, Grants, Reach};
use alo_strings::{Strings, Vocabulary, Word};

/// A transport whose reads and changes are counted independently.
struct ReadAnswer {
    /// The exact result supplied by the application's read method.
    answer: Result<String, NotDelivered>,
    /// Number of read calls delivered.
    reads: Cell<usize>,
    /// Number of changing calls delivered.
    changes: Cell<usize>,
}

impl Delivers for ReadAnswer {
    fn deliver(&self, _: &Message) -> Result<(), NotDelivered> {
        self.changes.set(self.changes.get() + 1);
        Ok(())
    }

    fn read_text(&self, _: &Message) -> Result<String, NotDelivered> {
        self.reads.set(self.reads.get() + 1);
        self.answer.clone()
    }
}

/// One granted status read through the normal adapter path.
fn read(to: &dyn Delivers) -> Result<Driven, alo_capability::Refused> {
    let at = SystemTime::UNIX_EPOCH + Duration::from_secs(1_760_000_000);
    let mut grants = Grants::default();
    grants.grant(
        Grant::checked(
            "@alo",
            Reach::Application("org.aloworld.Studio".into()),
            at,
            Duration::from_secs(3600),
        )
        .unwrap(),
    );
    let mut words = Vocabulary::empty();
    alo_capability::declare_into(&mut words).unwrap();
    alo_applications::words::declare_into(&mut words).unwrap();
    alo_adapters::words::declare_into(&mut words).unwrap();
    let strings = Strings::of(words);
    let adapters = alo_adapters::shipped_adapters().unwrap();
    let verbs = alo_adapters::adapter_verbs().unwrap();
    let call = verbs.call("studio.status", &[]).unwrap();
    let authorised = Authorised::read(&call, &Grantee::named("@alo"), &grants, at).unwrap();
    let installed =
        Installed::holding([Application::called("org.aloworld.Studio", "alo Studio").unwrap()]);
    Driving::of(authorised, &adapters, &grants, &installed, &strings)
        .unwrap()
        .deliver(to, &strings)
}

#[test]
fn the_eight_kibibyte_limit_counts_bytes_and_accepts_the_exact_boundary() {
    for length in [0, 8191, 8192, 8193] {
        let text = "x".repeat(length);
        let transport = ReadAnswer {
            answer: Ok(text.clone()),
            reads: Cell::new(0),
            changes: Cell::new(0),
        };
        let result = read(&transport);
        if length <= 8192 {
            let answer = result.unwrap();
            assert_eq!(answer.reply(), Some(text.as_str()));
            assert_eq!(answer.application(), "org.aloworld.Studio");
            assert!(!answer.unanswered());
        } else {
            assert!(result.is_err());
        }
        assert_eq!(transport.reads.get(), 1);
        assert_eq!(transport.changes.get(), 0);
    }
    let transport = ReadAnswer {
        answer: Ok("é".repeat(4097)),
        reads: Cell::new(0),
        changes: Cell::new(0),
    };
    assert!(
        read(&transport).is_err(),
        "the bound is UTF-8 bytes, not characters"
    );
}

#[test]
fn an_unanswered_read_is_recordable_uncertainty_without_an_invented_answer() {
    let transport = ReadAnswer {
        answer: Err(NotDelivered::DidNotAnswer),
        reads: Cell::new(0),
        changes: Cell::new(0),
    };
    let result = read(&transport).unwrap();
    assert!(result.unanswered());
    assert_eq!(result.reply(), None);
    assert_eq!(result.application(), "org.aloworld.Studio");
    assert_eq!(transport.reads.get(), 1);
    assert_eq!(transport.changes.get(), 0);
}

#[test]
fn a_transport_with_only_changes_cannot_fabricate_a_status_read() {
    /// A legacy transport which deliberately has no read implementation.
    struct ChangesOnly(Cell<usize>);
    impl Delivers for ChangesOnly {
        fn deliver(&self, _: &Message) -> Result<(), NotDelivered> {
            self.0.set(self.0.get() + 1);
            Ok(())
        }
    }
    let transport = ChangesOnly(Cell::new(0));
    assert!(read(&transport).is_err());
    assert_eq!(transport.0.get(), 0);
}

#[test]
fn change_descriptions_require_words_declared_by_the_same_adapter() {
    let foreign = Word::saying("test.foreign.change", "A foreign change.");
    let error = alo_adapters::studio::loaded()
        .unwrap()
        .describing_change("enable_microphone", foreign, None)
        .unwrap_err();
    assert!(
        matches!(error, alo_adapters::NotLoaded::WordNotDeclared { key, .. }
        if key == "test.foreign.change")
    );
}
