//! The printers already set up are listed, one is removed or made the one this
//! machine prints on — each one request and nothing else — and every way the
//! printing service can say no is a refusal rather than a change.
//!
//! What `docs/autonomy/v0-5-the-broker-and-the-disk-plan.md` task 2 needs from
//! this crate: the three changes the privileged broker carries out are
//! `alo-printing`'s own decided operations, and the identity each printer
//! crosses into the broker under is what the printing service reported.
//! Against a printing service on this machine's loopback that speaks the
//! protocol and remembers what it was sent.

#![expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

mod serving;

use alo_printing::ipp::Group;
use alo_printing::{Called, CannotChange, find, make_default, printers_set_up, remove};

use serving::{
    Answer, DELETE_PRINTER, GET_DEVICES, GET_PRINTERS, SET_DEFAULT, a_printing_service,
    nothing_listening, ok, status, with_device, with_printer,
};

/// The queue a Brother on the office network was set up under.
const THE_BROTHER: &str = "alo-brother-hl-l2350dw-series-5b1f0c3e9a7d2468";

/// The queue a Canon on a cable was set up under.
const THE_CANON: &str = "alo-canon-pixma-g3560-0f1e2d3c4b5a6978";

/// What the printing service keeps on an office machine: two printers alo OS
/// set up, the Brother listed twice, a printer made of software, and a queue
/// whose name no printer set up here could have.
fn an_office_machine() -> Answer {
    let mut listed = ok();
    for (queue, uri, info) in [
        (
            THE_BROTHER,
            "dnssd://Brother%20HL-L2350DW._ipp._tcp.local/?uuid=e3248000",
            "Brother HL-L2350DW series",
        ),
        (
            THE_CANON,
            "ippusb://Canon%20G3560/?serial=X",
            "Canon PIXMA G3560",
        ),
        (
            THE_BROTHER,
            "dnssd://Brother%20HL-L2350DW._ipp._tcp.local/?uuid=e3248000",
            "Brother HL-L2350DW series",
        ),
        ("PDF", "cups-pdf:/", "Virtual PDF Printer"),
        (
            "../../etc/passwd",
            "ipp://192.168.1.20/ipp/print",
            "A queue with a path for a name",
        ),
    ] {
        listed = with_printer(listed, queue, uri, info);
    }
    Answer::Chunked(listed)
}

/// **The printers set up are listed, each once, by the name each gave itself**,
/// and each crosses into the broker as what the printing service reported: its
/// queue. A printer made of software and a queue no printer here could have are
/// not on the list, and listing sends one request.
#[test]
fn the_printers_set_up_are_listed_and_reported_as_their_queues() {
    let serving = a_printing_service(|_| an_office_machine());
    let listed = printers_set_up(&serving.service()).expect("the service answered");

    let called: Vec<&Called> = listed.iter().map(|printer| printer.called()).collect();
    assert_eq!(
        called,
        [
            &Called::Named("Brother HL-L2350DW series".to_owned()),
            &Called::Named("Canon PIXMA G3560".to_owned()),
        ]
    );
    let reported: Vec<&[u8]> = listed.iter().map(|printer| printer.as_reported()).collect();
    assert_eq!(reported, [THE_BROTHER.as_bytes(), THE_CANON.as_bytes()]);
    assert_eq!(serving.operations(), [GET_PRINTERS]);
}

/// A machine with no printer at all is an empty list, and a service that is not
/// there is not one.
#[test]
fn no_printer_set_up_is_an_empty_list_and_no_service_is_not() {
    let serving = a_printing_service(|_| Answer::Ipp(status(0x0406)));
    assert_eq!(
        printers_set_up(&serving.service()).map(|l| l.len()).ok(),
        Some(0)
    );
    assert!(printers_set_up(&nothing_listening()).is_err());
}

/// **A found printer is reported as the address it was found at**, and two
/// printers found are two identities.
#[test]
fn a_found_printer_is_reported_as_where_it_was_found() {
    let serving = a_printing_service(|_| {
        Answer::Ipp(with_device(
            with_device(
                ok(),
                "dnssd://Brother%20HL-L2350DW._ipp._tcp.local/?uuid=e3248000",
                "Brother HL-L2350DW series",
            ),
            "ippusb://Canon%20G3560/?serial=X",
            "Canon PIXMA G3560",
        ))
    });
    let found = find(&serving.service()).expect("the service answered");
    let reported: Vec<&[u8]> = found.iter().map(|printer| printer.as_reported()).collect();
    assert_eq!(
        reported,
        [
            "dnssd://Brother%20HL-L2350DW._ipp._tcp.local/?uuid=e3248000".as_bytes(),
            "ippusb://Canon%20G3560/?serial=X".as_bytes(),
        ]
    );
    assert_eq!(serving.operations(), [GET_DEVICES]);
}

/// The printer set up under this queue, found through the listing.
fn the_canon() -> alo_printing::Printer {
    let serving = a_printing_service(|_| an_office_machine());
    printers_set_up(&serving.service())
        .expect("the service answered")
        .into_iter()
        .find(|printer| printer.as_reported() == THE_CANON.as_bytes())
        .expect("the Canon is set up")
}

/// **Removing sends one request, about exactly that printer.**
#[test]
fn removing_a_printer_is_one_request_about_that_printer() {
    let canon = the_canon();
    let serving = a_printing_service(|_| Answer::Ipp(ok()));
    assert_eq!(remove(&serving.service(), &canon), Ok(()));

    assert_eq!(serving.operations(), [DELETE_PRINTER]);
    let heard = serving.heard();
    let asked_about = heard
        .first()
        .and_then(|request| request.attribute(Group::Operation, "printer-uri"))
        .and_then(|uri| uri.text().map(str::to_owned));
    assert_eq!(
        asked_about.as_deref(),
        Some(format!("ipp://localhost/printers/{THE_CANON}").as_str())
    );
}

/// **Making a printer the default sends one request, about exactly that
/// printer**, and removes nothing.
#[test]
fn making_a_printer_the_default_is_one_request_about_that_printer() {
    let canon = the_canon();
    let serving = a_printing_service(|_| Answer::Ipp(ok()));
    assert_eq!(make_default(&serving.service(), &canon), Ok(()));

    assert_eq!(serving.operations(), [SET_DEFAULT]);
    let heard = serving.heard();
    let asked_about = heard
        .first()
        .and_then(|request| request.attribute(Group::Operation, "printer-uri"))
        .and_then(|uri| uri.text().map(str::to_owned));
    assert_eq!(
        asked_about.as_deref(),
        Some(format!("ipp://localhost/printers/{THE_CANON}").as_str())
    );
}

/// **Every way the service says no is a refusal, for both changes**: a printer
/// it no longer keeps, an account it will not let make the change — said in
/// the protocol or by the web server in front of it — and a service that is not
/// there, hangs up, or answers with something else.
#[test]
fn every_no_from_the_printing_service_is_a_refusal() {
    let canon = the_canon();
    let cases: [(fn() -> Answer, CannotChange); 6] = [
        (|| Answer::Ipp(status(0x0406)), CannotChange::NoLongerThere),
        (|| Answer::Ipp(status(0x0403)), CannotChange::NotPermitted),
        (|| Answer::Ipp(status(0x0401)), CannotChange::NotPermitted),
        (|| Answer::Http(403), CannotChange::NotPermitted),
        (|| Answer::HangUp, CannotChange::ServiceNotAnswering),
        (
            || Answer::Ipp(status(0x0500)),
            CannotChange::ServiceNotAnswering,
        ),
    ];
    for (answering, refused) in cases {
        let serving = a_printing_service(move |_| answering());
        assert_eq!(remove(&serving.service(), &canon), Err(refused));
        assert_eq!(make_default(&serving.service(), &canon), Err(refused));
    }
    assert_eq!(
        remove(&nothing_listening(), &canon),
        Err(CannotChange::ServiceNotAnswering)
    );
    assert_eq!(
        make_default(&nothing_listening(), &canon),
        Err(CannotChange::ServiceNotAnswering)
    );
}
