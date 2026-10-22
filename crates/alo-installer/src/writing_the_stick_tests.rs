//! What writing a stick refuses, and the one road where it writes.
//!
//! **The refusals are the subject.** `docs/features.md` carries its longest
//! warning about this road — *a stick that is written by mistake is somebody's
//! photographs* — so every way of not writing is tested here, and the happy
//! path is one test among them rather than the first.
//!
//! # What these cannot see, said rather than left to be found
//!
//! `TheMachine::wrote_the_device` is the seam, and a fake stands in for it. So
//! these settle **which device is named, when, and after what consent** — and
//! nothing at all about whether a real device is really written. That needs
//! Windows and a stick somebody can spare, and it is named in the pull request
//! rather than assumed.

#![expect(
    clippy::expect_used,
    clippy::indexing_slicing,
    clippy::panic,
    reason = "in a test, a panic naming what was measured is the failure being reported"
)]

use std::path::{Path, PathBuf};

use alo_strings::{Said, Strings};

use super::{THE_IMAGE, Written, write_the_stick};
use crate::machine::{Ran, TheMachine};
use crate::program::Program;

/// A machine with devices, an image, and a person who types.
struct AMachine {
    /// What `ListingTheDisks` prints.
    disks: String,
    /// What the person types.
    typed: String,
    /// The files it has.
    files: Vec<PathBuf>,
    /// Everything it was told, in order.
    said: Vec<String>,
    /// Every device written, with what was written onto it.
    wrote: Vec<(String, PathBuf)>,
    /// Whether a write refuses.
    refuses: bool,
    /// How big the image is, in bytes.
    image_is: u64,
    /// Whether reading any file refuses, so that a road which reads one is
    /// caught rather than merely slow.
    refuse_reads: bool,
}

impl AMachine {
    /// A machine with these devices, this image, and this answer.
    fn with(disks: &str, typed: &str, has_the_image: bool) -> Self {
        Self {
            disks: disks.to_owned(),
            typed: typed.to_owned(),
            files: if has_the_image {
                vec![Path::new("C:/alo").join(THE_IMAGE)]
            } else {
                Vec::new()
            },
            said: Vec::new(),
            wrote: Vec::new(),
            refuses: false,
            // Smaller than the 32 GB stick in the fixtures, so the ordinary
            // road is not the too-small one.
            image_is: 8_000_000_000,
            refuse_reads: false,
        }
    }

    /// Everything it said, joined, for asking whether a sentence was shown.
    fn everything_said(&self) -> String {
        self.said.join("\n")
    }
}

impl TheMachine for AMachine {
    fn how_big_is(&mut self, file: &Path) -> Option<u64> {
        self.files
            .iter()
            .any(|had| had == file)
            .then_some(self.image_is)
    }

    fn say(&mut self, said: &Said) {
        assert!(
            said.unfilled().is_empty(),
            "a gap reached a person: {}",
            said.text()
        );
        self.said.push(said.text().to_owned());
    }

    fn ask(&mut self, _said: &Said) -> String {
        self.typed.clone()
    }

    fn run(&mut self, program: &Program) -> std::io::Result<Ran> {
        assert!(
            matches!(program, Program::ListingTheDisks),
            "this road runs one program and it ran {program:?}"
        );
        Ok(Ran {
            succeeded: true,
            printed: self.disks.clone(),
        })
    }

    fn downloaded_into(&mut self) -> std::io::Result<PathBuf> {
        Ok(Path::new("C:/alo").to_path_buf())
    }

    fn this_program(&mut self) -> std::io::Result<PathBuf> {
        Ok(Path::new("C:/alo/alo-installer.exe").to_path_buf())
    }

    fn read(&mut self, file: &Path) -> std::io::Result<Vec<u8>> {
        assert!(
            !self.refuse_reads,
            "this road read {file:?}, and the only thing it needs of a file is its size"
        );
        if self.files.iter().any(|had| had == file) {
            return Ok(vec![0_u8; 8]);
        }
        Err(std::io::Error::from(std::io::ErrorKind::NotFound))
    }

    fn write(&mut self, _file: &Path, _bytes: &[u8]) -> std::io::Result<()> {
        panic!("this road writes no file on this computer")
    }

    fn pause(&mut self, _for_as_long_as: std::time::Duration) {}

    fn wrote_the_device(
        &mut self,
        device: &str,
        from: &Path,
        told: &mut dyn FnMut(u64, u64),
    ) -> std::io::Result<()> {
        self.wrote.push((device.to_owned(), from.to_path_buf()));
        told(0, 8);
        if self.refuses {
            return Err(std::io::Error::other("the device stopped answering"));
        }
        told(8, 8);
        Ok(())
    }
}

/// This repository's own words, in English.
fn english() -> Strings {
    Strings::of(alo_saying::everything_this_machine_can_say().expect("the machine's vocabulary"))
}

/// One removable device and one fixed disk, as Windows prints them.
const ONE_STICK_AND_THE_SYSTEM_DISK: &str = r#"[
 {"Number":0,"FriendlyName":"CT120BX500SSD1","SerialNumber":"a","BusType":"SATA",
  "UniqueId":"u0","Size":120034123776,"PartitionStyle":"GPT","IsReadOnly":false,"Partitions":[]},
 {"Number":2,"FriendlyName":"SanDisk Ultra","SerialNumber":"b","BusType":"USB",
  "UniqueId":"u2","Size":32000000000,"PartitionStyle":"MBR","IsReadOnly":false,"Partitions":[]}
]"#;

/// **The system disk is never offered**, which is the promise this road's
/// warning rests on.
///
/// `features.md`: *the devices offered are removable ones and never the disk
/// this computer runs on*. A person who typed the system disk's name must be
/// refused as if they had typed nonsense, because from here those are the same
/// thing: a name that is not on the list.
#[test]
fn the_disk_this_computer_runs_on_is_never_offered() {
    let mut machine = AMachine::with(ONE_STICK_AND_THE_SYSTEM_DISK, "CT120BX500SSD1", true);
    let went = write_the_stick(&mut machine, &english());

    assert_eq!(
        went,
        Written::NotThatName,
        "{:?}",
        machine.everything_said()
    );
    assert!(
        machine.wrote.is_empty(),
        "the system disk was written: {:?}",
        machine.wrote
    );
    assert!(
        !machine.everything_said().contains("CT120BX500SSD1"),
        "the system disk was offered by name:\n{}",
        machine.everything_said()
    );
}

/// **Nothing is written until the device's own name is typed.**
#[test]
fn typing_nothing_writes_nothing() {
    let mut machine = AMachine::with(ONE_STICK_AND_THE_SYSTEM_DISK, "   ", true);
    let went = write_the_stick(&mut machine, &english());

    assert_eq!(went, Written::NotAgreed);
    assert!(machine.wrote.is_empty(), "{:?}", machine.wrote);
}

/// **A name that is nearly right is not right.**
///
/// **Space around the name is not a mistake and is trimmed.** `"SanDisk Ultra "`
/// was in this list until the test ran, and the test was wrong rather than the
/// road: a person who typed a trailing space meant the name,
/// `crate::starting_the_stick` trims the same way, and refusing it would be a
/// different promise — *type exactly these characters* rather than *type its
/// name*. What is not trimmed is anything inside it, which the doubled space
/// below holds.
///
/// No closest match, no case folding. A person who mistyped is the one least
/// able to tell which device they got, and here the wrong answer erases it.
#[test]
fn a_name_that_is_nearly_right_writes_nothing() {
    for nearly in [
        "SanDisk ultra",
        "SanDisk",
        "SanDisk  Ultra",
        "sandisk ultra",
    ] {
        let mut machine = AMachine::with(ONE_STICK_AND_THE_SYSTEM_DISK, nearly, true);
        let went = write_the_stick(&mut machine, &english());
        assert_eq!(went, Written::NotThatName, "{nearly:?} was accepted");
        assert!(
            machine.wrote.is_empty(),
            "{nearly:?} wrote {:?}",
            machine.wrote
        );
    }
}

/// **With no image beside the installer, nobody is asked anything.**
///
/// The consent is spent before it is taken otherwise: a person who chose a
/// device and typed its name, to be told there was nothing to write, reads the
/// next warning less carefully.
#[test]
fn with_no_image_nobody_is_asked_and_nothing_is_written() {
    let mut machine = AMachine::with(ONE_STICK_AND_THE_SYSTEM_DISK, "SanDisk Ultra", false);
    let went = write_the_stick(&mut machine, &english());

    assert_eq!(went, Written::NoImage);
    assert!(machine.wrote.is_empty());
    assert!(
        !machine.everything_said().contains("SanDisk Ultra"),
        "a device was offered when there was nothing to write:\n{}",
        machine.everything_said()
    );
}

/// **A machine with nothing removable plugged in says so.**
#[test]
fn nothing_removable_plugged_in_is_said() {
    const ONLY_THE_SYSTEM_DISK: &str = r#"[
     {"Number":0,"FriendlyName":"CT120BX500SSD1","SerialNumber":"a","BusType":"SATA",
      "UniqueId":"u0","Size":120034123776,"PartitionStyle":"GPT","IsReadOnly":false,"Partitions":[]}
    ]"#;
    let mut machine = AMachine::with(ONLY_THE_SYSTEM_DISK, "anything", true);
    let went = write_the_stick(&mut machine, &english());

    assert_eq!(went, Written::NothingPluggedIn);
    assert!(machine.wrote.is_empty());
}

/// **A device Windows will not write is not offered.**
///
/// A stick with its switch on. Offering it walks a person through the whole
/// consent to a refusal at the last step, which teaches them the warning was
/// noise.
#[test]
fn a_read_only_device_is_not_offered() {
    const A_LOCKED_STICK: &str = r#"[
     {"Number":2,"FriendlyName":"SanDisk Ultra","SerialNumber":"b","BusType":"USB",
      "UniqueId":"u2","Size":32000000000,"PartitionStyle":"MBR","IsReadOnly":true,"Partitions":[]}
    ]"#;
    let mut machine = AMachine::with(A_LOCKED_STICK, "SanDisk Ultra", true);
    let went = write_the_stick(&mut machine, &english());

    assert_eq!(
        went,
        Written::NothingPluggedIn,
        "{:?}",
        machine.everything_said()
    );
    assert!(machine.wrote.is_empty());
}

/// **The device named is the device written**, by its physical-drive path and
/// never by a letter.
///
/// A letter names a volume inside a partition; this image replaces the
/// partition table. Writing through a letter would write inside the partition
/// that is already there and the stick would not start anything.
#[test]
fn the_stick_named_is_the_stick_written_by_its_physical_drive() {
    let mut machine = AMachine::with(ONE_STICK_AND_THE_SYSTEM_DISK, "SanDisk Ultra", true);
    let went = write_the_stick(&mut machine, &english());

    assert_eq!(
        went,
        Written::Written {
            device: "SanDisk Ultra".to_owned()
        },
        "{:?}",
        machine.everything_said()
    );
    assert_eq!(machine.wrote.len(), 1, "{:?}", machine.wrote);
    let (device, from) = &machine.wrote[0];
    assert_eq!(device, r"\\.\PhysicalDrive2", "a letter or the wrong disk");
    assert!(
        from.ends_with(THE_IMAGE),
        "something other than the image was written: {from:?}"
    );
}

/// **A device that refuses part-way is said to have refused, and the person is
/// told the stick is now unusable.**
#[test]
fn a_device_that_refuses_is_said_and_the_stick_is_not_claimed_ready() {
    let mut machine = AMachine::with(ONE_STICK_AND_THE_SYSTEM_DISK, "SanDisk Ultra", true);
    machine.refuses = true;
    let went = write_the_stick(&mut machine, &english());

    assert!(
        matches!(went, Written::NotWritten { .. }),
        "{went:?}: {}",
        machine.everything_said()
    );
    assert_eq!(machine.wrote.len(), 1, "it still named the right device");
}

/// **Every warning is shown before the question, not after it.**
///
/// The order is the promise. A person reads the consent once, and a warning
/// that arrived after they had typed would be a warning about something they
/// had already agreed to.
#[test]
fn the_warnings_come_before_the_question() {
    let mut machine = AMachine::with(ONE_STICK_AND_THE_SYSTEM_DISK, "SanDisk Ultra", true);
    let _ = write_the_stick(&mut machine, &english());
    let said = machine.everything_said();

    let erased = said
        .find("Everything on the stick you choose will be erased")
        .expect("the erasing warning is shown");
    let only_removable = said
        .find("Only removable devices are shown")
        .expect("the promise about which devices is shown");
    let device = said.find("SanDisk Ultra").expect("the device is offered");

    assert!(
        only_removable < device && erased < device,
        "a warning came after the device it is about:\nremovable at {only_removable}, \
         erased at {erased}, device at {device}\n{said}"
    );
}

/// **Space around the name is trimmed, and that is deliberate.**
///
/// The sibling road does it, and *type its name* is not *type exactly these
/// characters*. Held as its own test so that the trimming is a decision with a
/// reason rather than something the other test happens to allow.
#[test]
fn space_around_the_name_is_not_a_mistake() {
    let mut machine = AMachine::with(ONE_STICK_AND_THE_SYSTEM_DISK, "  SanDisk Ultra  ", true);
    let went = write_the_stick(&mut machine, &english());
    assert_eq!(
        went,
        Written::Written {
            device: "SanDisk Ultra".to_owned()
        },
        "{:?}",
        machine.everything_said()
    );
}

/// **A person can reach this road by typing a word, and the word is dispatched.**
///
/// The fault this is written for was in this change for the length of one
/// review: `write_the_stick` was declared, exported, and covered by the ten
/// tests above, and **no word a person could type arrived at it.** Found by the
/// desktop lane reading `main.rs`, not by any of these tests — because a test
/// that calls the function directly cannot see that nothing else does.
///
/// So this reads `main.rs` off the disk. That is a blunt instrument and it is
/// the right one here: the question is not *does the function work*, which the
/// ten above answer, but *is there a road from a person to it*, which only the
/// dispatch can answer.
#[test]
fn a_person_can_reach_this_road_by_typing_a_word() {
    let main = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("src")
            .join("main.rs"),
    )
    .expect("this crate has a main.rs");

    // The control. A read that came back empty would make both assertions below
    // pass against nothing, which is this repository's commonest false green.
    assert!(
        main.len() > 1000,
        "read {} bytes of main.rs, so this test is broken rather than the dispatch",
        main.len()
    );
    // **The dispatch, not the names.** This asserted `contains("THE_WRITINGS_WORD")`
    // and `contains("write_the_stick")` until it was mutated — and **both are
    // satisfied by the `use` statement at the top of the file**, so deleting the
    // whole match arm left the test green. That is *a mention is not a call*,
    // which this repository has a note named after, written by me into the guard
    // whose only job was to catch it.
    //
    // So: the shape of the arm and the shape of the call, neither of which an
    // import can contain.
    assert!(
        main.contains("one == THE_WRITINGS_WORD"),
        "no word reaches the writer: `main.rs` has no arm matching THE_WRITINGS_WORD. \
         Importing the name is not dispatching it."
    );
    assert!(
        main.contains("write_the_stick(&mut machine"),
        "the word is matched and the road is not taken: `main.rs` matches \
         THE_WRITINGS_WORD and never calls write_the_stick with the machine"
    );
}

/// **A device the image does not fit on is refused before it is erased.**
///
/// Writing as far as it goes would leave a person a wiped stick that starts
/// nothing, having been told it was ready — and the device they named is the one
/// thing this road destroys.
///
/// Reachable only because the presence check now asks the image's **size**
/// rather than reading the whole file; the fault that replaced read the entire
/// image into memory to answer *is it there*.
#[test]
fn a_device_the_image_does_not_fit_on_is_refused_before_it_is_erased() {
    let mut machine = AMachine::with(ONE_STICK_AND_THE_SYSTEM_DISK, "SanDisk Ultra", true);
    // Bigger than the 32 GB stick in the fixture.
    machine.image_is = 64_000_000_000;
    let went = write_the_stick(&mut machine, &english());

    assert!(
        matches!(went, Written::TooSmall { .. }),
        "{went:?}: {}",
        machine.everything_said()
    );
    assert!(
        machine.wrote.is_empty(),
        "a stick too small for the image was written anyway: {:?}",
        machine.wrote
    );
}

/// **The image is never read to find out whether it is there.**
///
/// The fault the desktop lane found: `TheMachine::read` is *a whole file*, so
/// asking *is the image there* by calling it allocated **gigabytes** and threw
/// them away — an out-of-memory failure on a laptop, before the first warning
/// was drawn.
///
/// No test could see it from the outside, because a fake's `read` returns a
/// small `Vec` and is correct and fast about nothing. So this holds the fake to
/// refusing: if this road ever reads the image again, `AMachine::read` is never
/// asked for it and the one that is asked is `how_big_is`.
#[test]
fn the_image_is_asked_about_and_never_read() {
    let mut machine = AMachine::with(ONE_STICK_AND_THE_SYSTEM_DISK, "SanDisk Ultra", true);
    machine.refuse_reads = true;
    let went = write_the_stick(&mut machine, &english());

    assert_eq!(
        went,
        Written::Written {
            device: "SanDisk Ultra".to_owned()
        },
        "the road read a file it only needed the size of: {}",
        machine.everything_said()
    );
}
