//! Printers are found on the local network and on a cable, set up without a
//! person choosing a driver, a queue or a protocol, and never added by being
//! found.
//!
//! One clause of the acceptance for *A printer is found, set up, and says what
//! is wrong with it* in `docs/autonomy/v0-5-documents-and-paper-plan.md`,
//! against a printing service on this machine's loopback that speaks the
//! protocol and remembers what it was sent.

#![expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

mod serving;

use alo_printing::ipp::{Group, Value};
use alo_printing::{
    CannotSetUp, NoPrinter, Reached, Speaks, Unanswered, find, set_up, set_up_said,
    this_machines_printer,
};
use alo_strings::Strings;

use serving::{
    ADD_MODIFY_PRINTER, Answer, GET_DEFAULT, GET_DEVICES, SET_DEFAULT, a_printing_service,
    nothing_listening, ok, status, the_default, with_device,
};

/// The machine's one vocabulary, as a shell holds it.
fn in_english() -> Strings {
    Strings::of(alo_saying::everything_this_machine_can_say().expect("the machine's words"))
}

/// What the printing service's backends find in an ordinary office: a
/// driverless printer on the network, the same one twice, a laser on a USB
/// cable that needs its maker's program, a printer reached over IPP-over-USB
/// at this machine's own address, an old printer on a raw port, one with a
/// name that would put a second line on a screen, and a printer made of
/// software.
fn an_office() -> Answer {
    let mut devices = ok();
    for (uri, name) in [
        (
            "dnssd://Brother%20HL-L2350DW._ipp._tcp.local/?uuid=e3248000",
            "Brother HL-L2350DW series",
        ),
        (
            "dnssd://Brother%20HL-L2350DW._ipp._tcp.local/?uuid=e3248000",
            "Brother HL-L2350DW series",
        ),
        ("usb://HP/LaserJet%201020?serial=X", "HP LaserJet 1020"),
        ("ipp://localhost:60000/ipp/print", "Canon PIXMA G3560"),
        ("socket://192.168.1.30:9100", "Unknown"),
        (
            "ipps://192.168.1.44/ipp/print",
            "Printer\nyour files are being uploaded",
        ),
        ("cups-pdf:/", "Virtual PDF Printer"),
    ] {
        devices = with_device(devices, uri, name);
    }
    Answer::Chunked(devices)
}

/// **Printers on the local network and on a cable are found**, each once, each
/// with where it is and whether it sets up by itself — and a printer made of
/// software is not a printer.
#[test]
fn printers_on_the_network_and_on_a_cable_are_found() {
    let serving = a_printing_service(|_| an_office());
    let found = find(&serving.service()).expect("the service answered");
    let strings = in_english();

    let lines: Vec<String> = found
        .iter()
        .map(|printer| printer.said(&strings).into_text())
        .collect();
    assert_eq!(
        lines,
        [
            "Brother HL-L2350DW series, on your network",
            "HP LaserJet 1020, connected to this machine",
            "Canon PIXMA G3560, connected to this machine",
            "a printer that has not given a name this machine can show, on your network",
            "a printer that has not given a name this machine can show, on your network",
        ]
    );
    let how: Vec<(bool, Speaks)> = found
        .iter()
        .map(|printer| (printer.reached().crosses_the_network(), printer.speaks()))
        .collect();
    assert_eq!(
        how,
        [
            (true, Speaks::Driverless),
            (false, Speaks::OnlyThroughItsMakersProgram),
            (false, Speaks::Driverless),
            (true, Speaks::OnlyThroughItsMakersProgram),
            (true, Speaks::Driverless),
        ]
    );
    let first = found.first().expect("the network printer");
    assert_eq!(
        first.reached(),
        &Reached::OnTheNetwork {
            host: "Brother HL-L2350DW._ipp._tcp.local".to_owned()
        }
    );
    assert_eq!(
        first.proposal(&strings).text(),
        "Set up Brother HL-L2350DW series, so this machine can print on it"
    );
}

/// **Finding never adds a printer.** The service was sent exactly one request,
/// and it was the one that lists devices.
#[test]
fn finding_a_printer_never_adds_one() {
    let serving = a_printing_service(|_| an_office());
    let found = find(&serving.service()).expect("the service answered");
    assert_eq!(found.len(), 5);
    assert_eq!(serving.operations(), [GET_DEVICES]);
    let asked = serving.heard();
    let schemes: Vec<&str> = asked
        .first()
        .and_then(|request| request.attribute(Group::Operation, "include-schemes"))
        .map(|schemes| schemes.texts().collect())
        .unwrap_or_default();
    assert!(
        schemes.contains(&"dnssd") && schemes.contains(&"usb"),
        "{schemes:?}"
    );
}

/// **A printer is set up with nobody choosing a driver, a queue or a
/// protocol**: its own description of itself, a queue derived from what it is,
/// and the address it was found at — and it becomes the printer this machine
/// prints on.
#[test]
fn a_printer_is_set_up_without_anybody_choosing_a_driver_a_queue_or_a_protocol() {
    let serving = a_printing_service(|request| match request.code() {
        GET_DEVICES => an_office(),
        _ => Answer::Ipp(ok()),
    });
    let service = serving.service();
    let found = find(&service).expect("the service answered");
    let brother = found.first().expect("the network printer");

    let printer = set_up(&service, brother).expect("a driverless printer sets up");
    assert_eq!(
        set_up_said(&printer, &in_english()).text(),
        "Brother HL-L2350DW series is set up, and this machine prints on it"
    );
    assert_eq!(printer.reached(), brother.reached());

    assert_eq!(
        serving.operations(),
        [GET_DEVICES, ADD_MODIFY_PRINTER, SET_DEFAULT]
    );
    let heard = serving.heard();
    let adding = heard.get(1).expect("the request that added it");
    let text = |name| {
        adding
            .attribute(Group::Printer, name)
            .and_then(|attribute| attribute.text())
            .map(str::to_owned)
    };
    assert_eq!(text("ppd-name").as_deref(), Some("everywhere"));
    assert_eq!(
        text("device-uri").as_deref(),
        Some("dnssd://Brother%20HL-L2350DW._ipp._tcp.local/?uuid=e3248000")
    );
    let queue = adding
        .attribute(Group::Operation, "printer-uri")
        .and_then(|uri| uri.text())
        .expect("the queue it was added under");
    assert!(
        queue.starts_with("ipp://localhost/printers/alo-brother-hl-l2350dw-series-"),
        "{queue}"
    );
    assert_eq!(
        adding
            .attribute(Group::Printer, "printer-is-accepting-jobs")
            .map(|accepting| accepting.values().to_vec()),
        Some(vec![Value::Boolean(true)])
    );
    let defaulting = heard.get(2).expect("the request that made it the default");
    assert_eq!(
        defaulting
            .attribute(Group::Operation, "printer-uri")
            .and_then(|uri| uri.text()),
        Some(queue)
    );
}

/// **A printer that needs its maker's program is refused before anything is
/// sent**, in a sentence that says what kind of printer does set up.
#[test]
fn a_printer_that_needs_its_makers_program_is_refused_before_anything_is_sent() {
    let serving = a_printing_service(|_| an_office());
    let service = serving.service();
    let found = find(&service).expect("the service answered");
    let laser = found.get(1).expect("the laser on a cable");

    let refused = set_up(&service, laser).expect_err("no program from its maker is installed");
    assert_eq!(refused, CannotSetUp::NeedsItsMakersProgram);
    let said = refused.said(laser, &in_english());
    assert!(
        said.text()
            .starts_with("HP LaserJet 1020 can only be set up"),
        "{said}"
    );
    assert!(said.text().contains("sets up here by itself"), "{said}");
    assert!(!said.is_a_bug(), "{said}");
    assert_eq!(serving.operations(), [GET_DEVICES]);
}

/// **Setting up that fails leaves nothing half done**: a printer that did not
/// describe itself is not made the one this machine prints on, an account not
/// allowed to add printers is told so, and a service that is not there is said
/// as the machine's own printing not responding.
#[test]
fn a_set_up_that_fails_says_why_and_leaves_nothing_half_done() {
    let cases = [
        (0x0404_u16, None, CannotSetUp::DidNotAnswer),
        (0x0403, None, CannotSetUp::NotPermitted),
        (0, Some(401_u16), CannotSetUp::NotPermitted),
    ];
    for (code, http, expected) in cases {
        let serving = a_printing_service(move |request| match (request.code(), http) {
            (GET_DEVICES, _) => an_office(),
            (_, Some(http)) => Answer::Http(http),
            _ => Answer::Ipp(status(code)),
        });
        let service = serving.service();
        let found = find(&service).expect("the service answered");
        let brother = found.first().expect("the network printer");
        assert_eq!(set_up(&service, brother), Err(expected));
        assert_eq!(serving.operations(), [GET_DEVICES, ADD_MODIFY_PRINTER]);
        assert!(!expected.said(brother, &in_english()).is_a_bug());
    }

    let serving = a_printing_service(|request| match request.code() {
        GET_DEVICES => an_office(),
        _ => Answer::HangUp,
    });
    let service = serving.service();
    let found = find(&service).expect("the service answered");
    let brother = found.first().expect("the network printer");
    assert_eq!(
        set_up(&service, brother),
        Err(CannotSetUp::ServiceNotAnswering)
    );
}

/// Finding nothing is an answer, and a service that is not there is not.
#[test]
fn finding_nothing_is_an_answer_and_no_service_is_not() {
    let serving = a_printing_service(|_| Answer::Ipp(ok()));
    assert_eq!(find(&serving.service()).expect("an empty answer").len(), 0);
    assert!(matches!(
        find(&nothing_listening()),
        Err(Unanswered::NotRunning)
    ));
}

/// The printer this machine prints on is read back from the printing service,
/// which keeps it — and a machine where none was set up says so in a sentence
/// that says where to go.
#[test]
fn the_printer_this_machine_prints_on_is_the_one_the_service_keeps() {
    let serving = a_printing_service(|_| {
        Answer::Ipp(the_default(
            "alo-brother-hl-l2350dw-series-0123456789abcdef",
            "ipp://192.168.1.20/ipp/print",
            "Brother HL-L2350DW series",
        ))
    });
    let printer = this_machines_printer(&serving.service()).expect("a printer is set up");
    assert!(printer.reached().crosses_the_network());
    assert_eq!(serving.operations(), [GET_DEFAULT]);

    let serving = a_printing_service(|_| Answer::Ipp(status(0x0406)));
    let none = this_machines_printer(&serving.service()).expect_err("nothing is set up");
    assert_eq!(none, NoPrinter::NoneSetUp);
    assert_eq!(
        none.said(&in_english()).text(),
        "No printer is set up on this machine yet. Open Printers in Settings to find one"
    );

    // A default whose name is not a queue this crate could have made is not a
    // printer it prints on.
    let serving = a_printing_service(|_| {
        Answer::Ipp(the_default(
            "../../etc",
            "ipp://192.168.1.20/ipp/print",
            "x",
        ))
    });
    assert_eq!(
        this_machines_printer(&serving.service()),
        Err(NoPrinter::NoneSetUp)
    );
}
