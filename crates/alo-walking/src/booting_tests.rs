//! The machine a walk boots is the one the walks by hand booted, under KVM.

use std::path::PathBuf;

use super::Booting;

/// A machine with every path named.
fn machine(kvm: bool) -> Booting {
    Booting {
        disk: PathBuf::from("/w/alo.raw"),
        firmware: PathBuf::from("/usr/share/OVMF/OVMF_CODE_4M.fd"),
        variables: PathBuf::from("/w/vars.fd"),
        serial: PathBuf::from("/w/serial.raw.log"),
        qmp_port: 4444,
        kvm,
        memory: 6144,
        processors: 4,
    }
}

/// Under KVM the processor is the host's, and the monitor is QMP on a local
/// port only.
#[test]
fn under_kvm_with_qmp_on_a_local_port() {
    let arguments = machine(true).arguments();
    let line = arguments.join(" ");
    assert!(line.contains("-enable-kvm -cpu host"), "{line}");
    assert!(!line.contains("tcg"), "{line}");
    assert!(
        line.contains("-qmp tcp:127.0.0.1:4444,server=on,wait=off"),
        "{line}"
    );
    assert!(line.contains("-serial file:/w/serial.raw.log"), "{line}");
    assert!(
        line.contains("file=/w/alo.raw,format=raw,if=virtio"),
        "{line}"
    );
}

/// **No tablet and no other pointer**: the walk machine has the keyboard and
/// mouse a shipped machine has, so nothing the walk sees is because of a
/// device a person's machine lacks.
#[test]
fn no_device_a_shipped_machine_lacks() {
    let line = machine(true).arguments().join(" ");
    for absent in ["tablet", "usb-", "-vnc", "-spice"] {
        assert!(!line.contains(absent), "{absent} in {line}");
    }
}

/// Without KVM it says so, by emulating.
#[test]
fn without_kvm_it_emulates() {
    let line = machine(false).arguments().join(" ");
    assert!(line.contains("-accel tcg"), "{line}");
    assert!(!line.contains("kvm"), "{line}");
}
