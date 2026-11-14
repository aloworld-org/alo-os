//! **The machine a walk boots**, as qemu's command line.
//!
//! The same machine the walks by hand booted (`boot17.sh` on the lane
//! machine): q35, UEFI from OVMF with its own fresh variables, the disk on
//! virtio, `virtio-vga`, user networking — so first boot can reach Flathub —
//! and **the q35 default PS/2 keyboard and mouse, with no tablet**, because a
//! tablet is different hardware from what a shipped machine has. Two
//! differences, both about the instrument: the processor is the host's under
//! KVM, and the monitor is QMP on a local port rather than the human monitor.

use std::path::PathBuf;

/// Everything needed to boot the walk machine.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Booting {
    /// The written disk, raw.
    pub disk: PathBuf,
    /// OVMF's code, read-only.
    pub firmware: PathBuf,
    /// A fresh copy of OVMF's variables for this boot.
    pub variables: PathBuf,
    /// Where the serial port is written.
    pub serial: PathBuf,
    /// The local port QMP listens on.
    pub qmp_port: u16,
    /// Whether the processor is the host's under KVM; without, it is emulated.
    pub kvm: bool,
    /// Memory, in MiB.
    pub memory: u32,
    /// Processors.
    pub processors: u32,
}

impl Booting {
    /// qemu's arguments for this machine, in order.
    #[must_use]
    pub fn arguments(&self) -> Vec<String> {
        let path = |it: &PathBuf| it.display().to_string();
        let mut arguments: Vec<String> = vec!["-machine".to_owned(), "q35".to_owned()];
        if self.kvm {
            arguments.extend([
                "-enable-kvm".to_owned(),
                "-cpu".to_owned(),
                "host".to_owned(),
            ]);
        } else {
            arguments.extend(["-accel".to_owned(), "tcg".to_owned()]);
        }
        arguments.extend([
            "-m".to_owned(),
            self.memory.to_string(),
            "-smp".to_owned(),
            self.processors.to_string(),
            "-drive".to_owned(),
            format!(
                "if=pflash,format=raw,readonly=on,file={}",
                path(&self.firmware)
            ),
            "-drive".to_owned(),
            format!("if=pflash,format=raw,file={}", path(&self.variables)),
            "-drive".to_owned(),
            format!("file={},format=raw,if=virtio", path(&self.disk)),
            "-device".to_owned(),
            "virtio-vga".to_owned(),
            "-netdev".to_owned(),
            "user,id=net0".to_owned(),
            "-device".to_owned(),
            "virtio-net-pci,netdev=net0".to_owned(),
            "-display".to_owned(),
            "none".to_owned(),
            "-serial".to_owned(),
            format!("file:{}", path(&self.serial)),
            "-qmp".to_owned(),
            format!("tcp:127.0.0.1:{},server=on,wait=off", self.qmp_port),
            "-no-reboot".to_owned(),
        ]);
        arguments
    }
}

#[cfg(test)]
#[path = "booting_tests.rs"]
mod tests;
