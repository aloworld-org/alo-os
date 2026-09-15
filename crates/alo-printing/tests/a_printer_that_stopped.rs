//! When a printer stops, which of a closed set of things is wrong — each said
//! with what a person does about it, and anything else said as *not answering*
//! with what was tried, never as a code.
//!
//! One clause of the acceptance for *A printer is found, set up, and says what
//! is wrong with it* in `docs/autonomy/v0-5-documents-and-paper-plan.md`,
//! against a printing service on this machine's loopback reporting what real
//! printers report.

#![expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

mod serving;

use alo_printing::{
    Condition, Printer, PrintingService, Stopped, Tried, how_is, this_machines_printer,
};
use alo_strings::Strings;

use serving::{
    Answer, GET_DEFAULT, GET_PRINTER_ATTRIBUTES, a_printer_that_is, a_printing_service,
    nothing_listening, status, the_default,
};

/// The machine's one vocabulary, as a shell holds it.
fn in_english() -> Strings {
    Strings::of(alo_saying::everything_this_machine_can_say().expect("the machine's words"))
}

/// The printer this machine prints on, as the service describes it.
fn the_printer(service: &PrintingService) -> Printer {
    this_machines_printer(service).expect("a printer is set up")
}

/// How a printer reporting this is, asked through a service that describes it.
fn a_printer_reporting(state: i32, reasons: &'static [&'static str], accepting: bool) -> Condition {
    let serving = a_printing_service(move |request| match request.code() {
        GET_DEFAULT => Answer::Ipp(the_default(
            "alo-brother-0123456789abcdef",
            "ipp://192.168.1.20/ipp/print",
            "Brother HL-L2350DW series",
        )),
        _ => Answer::Ipp(a_printer_that_is(state, reasons, accepting)),
    });
    let service = serving.service();
    let condition = how_is(&service, &the_printer(&service));
    assert_eq!(serving.operations(), [GET_DEFAULT, GET_PRINTER_ATTRIBUTES]);
    condition
}

/// **Each of the closed set is said, with what a person does about it.**
/// Out of paper, jammed, out of ink and refused, from the words real printers
/// report them in — and none of those words reaches the person.
#[test]
fn each_thing_that_is_wrong_is_said_with_what_to_do_about_it() {
    let strings = in_english();
    let cases: [(i32, &'static [&'static str], bool, Stopped, &str); 4] = [
        (
            5,
            &["media-empty-error"],
            true,
            Stopped::OutOfPaper,
            "Put paper in its tray",
        ),
        (
            5,
            &["media-jam-error"],
            true,
            Stopped::Jammed,
            "take the stuck paper out",
        ),
        (
            5,
            &["marker-supply-empty-error"],
            true,
            Stopped::OutOfInk,
            "Put in a new cartridge",
        ),
        (
            3,
            &["none"],
            false,
            Stopped::RefusedTheJob,
            "Sending it again will not change that",
        ),
    ];
    for (state, reasons, accepting, expected, what_to_do) in cases {
        let condition = a_printer_reporting(state, reasons, accepting);
        assert_eq!(condition, Condition::Stopped(expected), "{reasons:?}");
        let said = condition.said(&strings);
        assert!(!said.is_a_bug(), "{said}");
        assert!(said.text().contains(what_to_do), "{said}");
        for reason in reasons {
            assert!(!said.text().contains(reason), "{said}");
        }
    }
}

/// **A state that is none of them is *not answering*, with what was tried** —
/// a door open, a word nobody defined, or a printer that is stopped and says
/// nothing — and the words the printer sent are never repeated.
#[test]
fn a_state_that_is_none_of_them_is_not_answering_with_what_was_tried_never_a_code() {
    let strings = in_english();
    let cases: [(i32, &'static [&'static str]); 4] = [
        (5, &["door-open-error"]),
        (5, &["com.example-fuser-over-temp-error"]),
        (5, &["none"]),
        (4, &["offline", "connecting-to-device"]),
    ];
    for (state, reasons) in cases {
        let condition = a_printer_reporting(state, reasons, true);
        assert_eq!(
            condition,
            Condition::Stopped(Stopped::NotAnswering {
                tried: Tried::AskingThePrinter
            }),
            "{reasons:?}"
        );
        let said = condition.said(&strings).into_text();
        assert!(
            said.starts_with("The printer is not answering: this machine asked how it is"),
            "{said}"
        );
        assert!(said.contains("switched on and connected"), "{said}");
        for reason in reasons {
            assert!(!said.contains(reason), "{said}");
        }
        assert!(
            !said.chars().any(|letter| letter.is_ascii_digit()),
            "{said}"
        );
    }
}

/// **Asking that fails is itself *not answering*, saying what was asked**: the
/// machine's own printing service, or a printer removed since it was set up.
#[test]
fn asking_that_fails_is_not_answering_saying_what_was_asked() {
    let strings = in_english();

    let serving = a_printing_service(|request| match request.code() {
        GET_DEFAULT => Answer::Ipp(the_default(
            "alo-brother-0123456789abcdef",
            "ipp://192.168.1.20/ipp/print",
            "Brother",
        )),
        _ => Answer::HangUp,
    });
    let service = serving.service();
    let printer = the_printer(&service);
    let condition = how_is(&service, &printer);
    assert_eq!(
        condition,
        Condition::Stopped(Stopped::NotAnswering {
            tried: Tried::AskingThisMachinesPrinting
        })
    );
    assert!(
        condition
            .said(&strings)
            .text()
            .contains("the part of this machine that prints did not respond"),
    );
    assert_eq!(
        how_is(&nothing_listening(), &printer),
        Condition::Stopped(Stopped::NotAnswering {
            tried: Tried::AskingThisMachinesPrinting
        })
    );

    let serving = a_printing_service(|request| match request.code() {
        GET_DEFAULT => Answer::Ipp(the_default(
            "alo-brother-0123456789abcdef",
            "ipp://192.168.1.20/ipp/print",
            "Brother",
        )),
        _ => Answer::Ipp(status(0x0406)),
    });
    let service = serving.service();
    let condition = how_is(&service, &the_printer(&service));
    assert_eq!(
        condition,
        Condition::Stopped(Stopped::NotAnswering {
            tried: Tried::LookingForItsSetUp
        })
    );
    assert!(condition.said(&strings).text().contains("Set it up again"));
}

/// **A warning is not a stop.** A printer low on toner, reporting it, is ready.
#[test]
fn a_printer_that_only_warns_is_ready() {
    let condition = a_printer_reporting(3, &["toner-low-report", "media-low-warning"], true);
    assert_eq!(condition, Condition::Ready);
    assert_eq!(condition.said(&in_english()).text(), "The printer is ready");
}

/// **Every stop is a sentence from the machine's own vocabulary**, and every one
/// translates: a German machine reads German, with nothing left in English.
#[test]
fn every_stop_is_in_the_machines_vocabulary_and_translates() {
    let strings = in_english();
    for stopped in Stopped::EVERY {
        let said = stopped.said(&strings);
        assert!(!said.is_a_bug(), "{said}");
        assert!(!said.text().is_empty());
    }

    let vocabulary = alo_saying::everything_this_machine_can_say().expect("the machine's words");
    let german = alo_strings::Language::written("de").expect("German");
    let speaking = vocabulary
        .check(alo_strings::Translation::into_language(german.clone()).says(
            alo_printing::words::JAMMED.key(),
            "Im Drucker steckt Papier fest. Öffnen Sie ihn dort, wohin sein eigenes Display oder \
             seine Lampen zeigen, ziehen Sie das Papier vorsichtig heraus und schließen Sie ihn wieder",
        ))
        .expect("a translation that keeps every gap");
    let mut strings = Strings::of(vocabulary);
    strings.speaks(speaking).expect("German");
    strings.prefers(&[german]);
    let said = Stopped::Jammed.said(&strings);
    assert!(said.is_translated(), "{said}");
    assert!(said.text().starts_with("Im Drucker"), "{said}");
}
