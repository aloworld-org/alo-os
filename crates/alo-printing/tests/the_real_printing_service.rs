//! Actual CUPS administration without Linux capabilities, and refusal of a
//! non-root process asking to authenticate as root. Uses private queues and
//! upstream's IPP Everywhere emulator; this is not physical-printer certification.

#![cfg(target_os = "linux")]
#![expect(
    clippy::unwrap_used,
    clippy::panic,
    reason = "unexpected runtime behavior fails acceptance"
)]

mod cups_runtime;
mod serving;

use alo_printing::{
    CannotChange, PrintingService, find, make_default, printers_set_up, remove, set_up,
    this_machines_printer,
};
use std::os::unix::fs::MetadataExt;

const TEST: &str = "cups_accepts_the_capability_free_broker_and_refuses_another_user";

/// Real CUPS verifies the socket peer, carries the selected operations, and
/// rejects the same claimed identity from a different UID.
#[test]
#[ignore = "requires root, setpriv, CUPS and ippeveprinter; starts only private services"]
fn cups_accepts_the_capability_free_broker_and_refuses_another_user() {
    if let Ok(phase) = std::env::var("ALO_CUPS_RUNTIME_PHASE") {
        exercise(&phase);
        return;
    }
    assert_eq!(
        std::fs::metadata("/proc/self").unwrap().uid(),
        0,
        "requires root"
    );
    let runtime = cups_runtime::Running::start();
    runtime.check("configure", false, TEST);
    runtime.check("refuse", true, TEST);
    runtime.check("remove", false, TEST);
    println!("real CUPS: selected setup/default/removal succeeded; wrong UID changed nothing");
}

fn exercise(phase: &str) {
    let status = std::fs::read_to_string("/proc/self/status").unwrap();
    for field in ["CapInh:", "CapPrm:", "CapEff:", "CapBnd:", "CapAmb:"] {
        let value = status
            .lines()
            .find_map(|line| line.strip_prefix(field))
            .unwrap()
            .trim();
        assert_eq!(u64::from_str_radix(value, 16).unwrap(), 0, "{field}");
    }
    let socket = std::env::var("ALO_CUPS_RUNTIME_SOCKET").unwrap();
    let service = PrintingService::for_the_broker(&socket);
    match phase {
        "configure" => configure(&service, &socket),
        "refuse" => {
            assert_ne!(std::fs::metadata("/proc/self").unwrap().uid(), 0);
            // Listing is a read. A false root claim gives this UID no right to change.
            let listed = printers_set_up(&service).unwrap();
            let printer = listed.first().unwrap();
            assert_eq!(remove(&service, printer), Err(CannotChange::NotPermitted));
            assert_eq!(
                make_default(&service, printer),
                Err(CannotChange::NotPermitted)
            );
            assert_eq!(printers_set_up(&service).unwrap(), listed);
        }
        "remove" => {
            let listed = printers_set_up(&service).unwrap();
            assert_eq!(listed.len(), 1, "the refused caller left the queue intact");
            remove(&service, listed.first().unwrap()).unwrap();
            assert!(printers_set_up(&service).unwrap().is_empty());
        }
        _ => panic!("unknown phase"),
    }
}

fn configure(service: &PrintingService, socket: &str) {
    assert_eq!(std::fs::metadata("/proc/self").unwrap().uid(), 0);
    // Discovery input is the existing protocol fixture. Setup is sent to real
    // CUPS, which talks to the unmodified upstream IPP Everywhere emulator.
    let uri = std::env::var("ALO_CUPS_RUNTIME_PRINTER").unwrap();
    let found = |name: &'static str| {
        let uri = uri.clone();
        let discovery = serving::a_printing_service(move |_| {
            serving::Answer::Ipp(serving::with_device(serving::ok(), &uri, name))
        });
        find(&discovery.service())
            .unwrap()
            .into_iter()
            .next()
            .unwrap()
    };
    let first = set_up(service, &found("First runtime printer")).unwrap();
    let second = set_up(service, &found("Second runtime printer")).unwrap();
    assert_eq!(printers_set_up(service).unwrap().len(), 2);
    assert_eq!(this_machines_printer(service).unwrap(), second.clone());
    make_default(service, &first).unwrap();
    assert_eq!(this_machines_printer(service).unwrap(), first.clone());
    // The ordinary socket client has no administrative authentication.
    assert_eq!(
        remove(&PrintingService::at_its_socket(socket), &first),
        Err(CannotChange::NotPermitted)
    );
    remove(service, &first).unwrap();
    assert_eq!(
        printers_set_up(service).unwrap().as_slice(),
        std::slice::from_ref(&second)
    );
    make_default(service, &second).unwrap();
}
