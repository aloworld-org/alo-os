//! **The label the Windows half writes is the label the environment looks
//! for**, held by a test rather than by a sentence in a document.
//!
//! # Why this is worth its own file
//!
//! The boot environment refuses any disk carrying `ALO-INSTALL`
//! (`alo_installing`'s `disks.rs`, `Unsuitable::HoldsThisInstaller`), and that
//! refusal is the whole of a safety rule the desktop lane is building a disk
//! picker on: **never offer the medium you booted from.** A stick that offered
//! itself would be a person erasing the installer mid-install.
//!
//! That rule rests entirely on the label being there. It is read from one
//! constant at both ends — `alo_installing::THIS_INSTALLER`, filled into the
//! Windows half's `Format-Volume` and compared in its own disk reading — so it
//! is **already true by construction**.
//!
//! **What was missing is a test that runs.** The only tests naming that
//! constant are in `the_installer_walked_on_a_real_windows.rs`, which runs on a
//! real Windows walk and not in the gate. So the two ends were held together by
//! a constant nothing checked, and a change to either would have been found by
//! a person booting a stick.
//!
//! *Promised to the desktop lane in those words: the label is a test, not a
//! documented step, because their rule 1 rests on it.*
//!
//! # What this does not cover
//!
//! **Nothing here writes a stick**, because nothing in this product does yet
//! and `docs/features.md` does not promise it — the four promises added in #863
//! are the environment asking, saying what is on each disk, the stick being for
//! trying, and Windows starting it. *Writing* one is `S1`'s second choice in
//! the design and has no line in `docs/features.md`, so it is scope rather than
//! work. This file is the guard that will be waiting when that line exists.

#![expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None is the failure being reported"
)]

use alo_installer::{DiskNumber, Disks, PartitionNumber, Program};

/// The one place a disk's label is written, and the one place it is read.
///
/// `PreparingTheArea`'s command is the only thing in this product that puts a
/// label on a partition the installer makes.
fn preparing_the_area() -> Program {
    Program::PreparingTheArea {
        disk: DiskNumber(0),
        partition: PartitionNumber(4),
        offset: 1_024 * 1_024 * 1_024,
    }
}

/// **The Windows half writes the label the environment looks for, and it
/// survives the encoding.**
///
/// Not *a* label and not a copy of the word: the constant itself, so the two
/// cannot be changed apart.
///
/// **The arguments are decoded first, and that is the point rather than a
/// detail.** `crate::encoded` hands PowerShell the script as Base64 of its
/// UTF-16LE text, so a reader of `arguments()` sees one long opaque word. The
/// first version of this test looked for the label in that word and failed
/// while the label was perfectly present — which is the better failure to have
/// had, because it means this now proves the label reaches PowerShell **through
/// the encoding** rather than merely appearing in a string somewhere upstream
/// of it. A label mangled by encoding is a label no disk carries and a rule
/// that silently stops holding.
#[test]
fn the_windows_half_writes_the_label_the_environment_looks_for() {
    let script = the_script_of(&preparing_the_area());

    // The control, and it comes first: the decoder works. Without this, a
    // decoder that returned an empty string would make every assertion below
    // vacuous and this test would pass for a command with no label at all.
    assert!(
        script.contains("Format-Volume"),
        "the decoder did not produce the script, so nothing below is being tested: {script}"
    );

    assert!(
        script.contains(alo_installing::THIS_INSTALLER),
        "the area is formatted without the label the environment refuses on: {script}"
    );
    // And it is given to `Format-Volume` as the new file system's label rather
    // than appearing somewhere else in the command by accident.
    assert!(
        script.contains(&format!(
            "-NewFileSystemLabel '{}'",
            alo_installing::THIS_INSTALLER
        )),
        "the label is in the script but not as the file system's label: {script}"
    );
}

/// The script a program really hands PowerShell, decoded from
/// `-EncodedCommand`.
///
/// Base64 of UTF-16LE, which is what `crate::encoded` writes. Decoded here
/// rather than re-encoded and compared, because re-encoding would mean
/// rebuilding the script this is meant to be reading.
fn the_script_of(program: &Program) -> String {
    let arguments = program.arguments();
    let encoded = arguments
        .iter()
        .skip_while(|one| *one != "-EncodedCommand")
        .nth(1)
        .expect("a PowerShell program is given its script as -EncodedCommand");
    let bytes = from_base64(encoded);
    let mut wide = Vec::with_capacity(bytes.len() / 2);
    let mut pairs = bytes.iter();
    while let (Some(low), Some(high)) = (pairs.next(), pairs.next()) {
        wide.push(u16::from_le_bytes([*low, *high]));
    }
    String::from_utf16_lossy(&wide)
}

/// Standard padded Base64, back to bytes.
fn from_base64(text: &str) -> Vec<u8> {
    const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = Vec::with_capacity(text.len() / 4 * 3);
    let mut held: u32 = 0;
    let mut have = 0u32;
    for at in text.bytes() {
        if at == b'=' {
            break;
        }
        let Some(value) = ALPHABET.iter().position(|one| *one == at) else {
            continue;
        };
        held = (held << 6) | value as u32;
        have += 6;
        if have >= 8 {
            have -= 8;
            out.push(((held >> have) & 0xFF) as u8);
        }
    }
    out
}

/// **And the Windows half recognises what it wrote.**
///
/// The other end of the same constant: a disk carrying the label is known to
/// hold an installer's area, so an earlier start that did not finish is found
/// rather than written over.
#[test]
fn the_windows_half_recognises_what_it_wrote() {
    let carrying = a_disk_whose_partition_is_labelled(alo_installing::THIS_INSTALLER);
    assert!(
        Disks::read(Some(&carrying))
            .expect("this is a disk listing")
            .hold_an_installer_area(),
        "a disk carrying the label is not recognised as holding an installer's area"
    );

    // The control, and it is the half that matters: a disk carrying some other
    // label is **not** recognised. Without it this test would pass for code
    // that answered `true` for every disk — which would refuse every disk a
    // person owns.
    let otherwise = a_disk_whose_partition_is_labelled("Recovery");
    assert!(
        !Disks::read(Some(&otherwise))
            .expect("this is a disk listing")
            .hold_an_installer_area(),
        "a disk carrying somebody else's label is taken for an installer's area"
    );
}

/// **The label is one word in one place.**
///
/// It is `alo_installing`'s, and this crate reads it rather than keeping its
/// own. A second copy would be two answers to one question, and the one that
/// would be wrong later is the one with a disk picker resting on it.
#[test]
fn the_label_is_one_word_in_one_place() {
    assert_eq!(alo_installing::THIS_INSTALLER, "ALO-INSTALL");
    // A FAT label is eleven characters, upper case, and a longer one is
    // silently truncated by `Format-Volume` — which would leave the environment
    // looking for a label no disk carries.
    assert!(
        alo_installing::THIS_INSTALLER.len() <= 11,
        "a FAT label longer than eleven characters is truncated when it is written"
    );
    assert_eq!(
        alo_installing::THIS_INSTALLER,
        alo_installing::THIS_INSTALLER.to_ascii_uppercase(),
        "a FAT label is upper case, and a lower-case one comes back upper and stops matching"
    );
}

/// A Windows disk listing whose fourth partition carries this label.
fn a_disk_whose_partition_is_labelled(label: &str) -> String {
    format!(
        r#"[{{"Number":0,"FriendlyName":"Samsung SSD 980 1TB","SerialNumber":"S5P2","BusType":"NVMe","UniqueId":"","Size":1000204886016,"PartitionStyle":"GPT","IsReadOnly":false,"Partitions":[{{"PartitionNumber":4,"Label":"{label}","Size":1073741824,"GptType":"{{ebd0a0a2-b9e5-4433-87c0-68b6b72699c7}}"}}]}}]"#
    )
}
