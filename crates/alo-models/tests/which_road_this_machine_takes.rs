//! **Which part of this machine runs the model, asked for rather than timed.**
//!
//! Task 22 of `docs/autonomy/v0-5-the-models-measured-plan.md`.
//! [ADR 0007](../../../docs/decisions/0007-the-cpu-is-the-default.md) settled
//! three weeks before this was written that *the CPU is the default; a GPU is
//! acceleration* and that *a GPU changes speed, not capability* — and nothing
//! implemented it. `on_the_gpu_bytes` was [`None`] at every site in the
//! repository, because nothing ever looked at a card, so a machine with one ran
//! exactly as slowly as a machine without.
//!
//! The unit tests beside `card.rs` and `road.rs` ask whether one reading or one
//! reason is right. This one asks the question from outside, which is the
//! question a person has: **is this machine using its graphics card, and if
//! not, why not — in words I can read, and without my having to time
//! anything?**
//!
//! # Every machine in it is a folder, except one
//!
//! A kernel's list of what draws is a folder of files, so every shape of
//! machine below is written as one: no card, an unreachable card, a card of
//! another vendor, a card too small, a machine that keeps no such list. That is
//! the only way one machine can be held to the answer it would give on five.
//!
//! The exception is [`this_machines_own_road_is_measured_and_said`], which
//! reads the machine the test is running on and states what it found. On the
//! machine this task was written on — the third PC, `AGAI01`, x86_64 with no
//! discrete graphics — `/sys/class/drm` holds a `version` file and nothing
//! else, so the answer is *no card* and the road is the processor. **The card
//! road is not shown here and is not claimed here**: that is task 23, and it
//! waits on a machine with a card the pinned runtime can use, exactly as
//! [ADR 0056](../../../docs/decisions/0056-a-sealed-disks-promise-is-shown-on-a-machine-with-a-chip.md)
//! left the chip's half of encryption waiting on a machine with a chip.

#![expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::io::{BufRead as _, BufReader, Write as _};
use std::net::TcpListener;
use std::path::{Path, PathBuf};
use std::thread;

use alo_models::card::{Vendor, WhatDrawsHere};
use alo_models::{
    Catalogue, MeasuredOn, ModelRuntime, Ollama, Road, WhichRoad, WhyTheProcessor, declare_into,
    model_words, words,
};
use alo_strings::{Key, Language, Strings, Translation, Vocabulary};

/// What the pinned runtime answers when asked which cards it can use, asked of
/// a real [`Ollama`] rather than written down here a second time.
fn what_the_runtime_can_use() -> &'static [Vendor] {
    Ollama::new(Catalogue::built_in().unwrap()).cards_it_can_use()
}

/// The weights the image pins, by the size the catalogue states for them — the
/// figure a card's memory is held against, and never `min_vram_gb`.
fn what_the_pinned_weights_need() -> u64 {
    Catalogue::built_in()
        .unwrap()
        .get("phi-3-mini-instruct")
        .unwrap()
        .download_bytes
}

/// A kernel's list of what draws, written by this test, removed when it is
/// done with it.
struct AMachine(PathBuf);

impl AMachine {
    /// One with nothing on its bus, which is the fleet this product exists for.
    fn with_no_card() -> Self {
        use std::sync::atomic::{AtomicU32, Ordering};
        static NEXT: AtomicU32 = AtomicU32::new(0);
        let at = std::env::temp_dir().join(format!(
            "alo-models-which-road-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        drop(std::fs::remove_dir_all(&at));
        std::fs::create_dir_all(&at).unwrap();
        std::fs::write(at.join("version"), "drm 1.1.0 20060810\n").unwrap();
        Self(at)
    }

    /// The same machine with a device on its bus, written the way the kernel
    /// writes one.
    fn and_a_card(
        self,
        named: &str,
        vendor: u16,
        driver: Option<&str>,
        memory: Option<u64>,
    ) -> Self {
        let device = self.0.join(named).join("device");
        std::fs::create_dir_all(&device).unwrap();
        std::fs::write(device.join("vendor"), format!("0x{vendor:04x}\n")).unwrap();
        let uevent = match driver {
            Some(driver) => format!("DRIVER={driver}\nPCI_ID=0x{vendor:04x}:0001\n"),
            None => format!("PCI_ID=0x{vendor:04x}:0001\n"),
        };
        std::fs::write(device.join("uevent"), uevent).unwrap();
        if let Some(memory) = memory {
            std::fs::write(device.join("mem_info_vram_total"), format!("{memory}\n")).unwrap();
        }
        self
    }

    /// What this machine has to draw with, read the way a real machine is read.
    fn draws(&self) -> WhatDrawsHere {
        WhatDrawsHere::among(&self.0)
    }

    /// The road this machine takes with nothing loaded — every reason below is
    /// this, with a different bus behind it.
    fn road(&self) -> Road {
        Road::of(
            None,
            &self.draws(),
            what_the_runtime_can_use(),
            what_the_pinned_weights_need(),
        )
    }
}

impl Drop for AMachine {
    fn drop(&mut self) {
        drop(std::fs::remove_dir_all(&self.0));
    }
}

/// A runtime answering one request with what it was given, handing back what
/// it was sent — so a test can assert on the bytes that really went out rather
/// than on the ones that were meant to.
fn a_runtime_answering(reply: &'static str) -> (String, thread::JoinHandle<String>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let address = format!("http://{}", listener.local_addr().unwrap());
    let handle = thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        let mut reader = BufReader::new(stream.try_clone().unwrap());
        let mut head = String::new();
        let mut length = 0usize;
        loop {
            let mut line = String::new();
            reader.read_line(&mut line).unwrap();
            if let Some(said) = line.to_lowercase().strip_prefix("content-length:") {
                length = said.trim().parse().unwrap_or(0);
            }
            let done = line == "\r\n" || line.is_empty();
            head.push_str(&line);
            if done {
                break;
            }
        }
        let mut body = vec![0u8; length];
        std::io::Read::read_exact(&mut reader, &mut body).unwrap();
        head.push_str(&String::from_utf8_lossy(&body));
        write!(
            stream,
            "HTTP/1.1 200 OK\r\ncontent-type: application/json\r\ncontent-length: {}\r\n\r\n{reply}",
            reply.len()
        )
        .unwrap();
        stream.flush().unwrap();
        head
    });
    (address, handle)
}

/// This crate's words with one of them said in German, German preferred.
fn speaking_german(said: &[(alo_strings::Word, &str)]) -> Strings {
    let vocabulary = model_words().unwrap();
    let german = Language::written("de").unwrap();
    let mut translation = Translation::into_language(german.clone());
    for (word, says) in said {
        translation = translation.says(word.key(), *says);
    }
    let speaking = vocabulary.check(translation).unwrap();
    let mut strings = Strings::of(vocabulary);
    strings.speaks(speaking).unwrap();
    strings.prefers(&[german]);
    strings
}

/// **The machine this test is running on says which road it takes, and says
/// why**, measured from its own kernel rather than assumed.
///
/// It asserts nothing about *which* answer, for the reason
/// `against_a_model_on_this_machine.rs` asserts no particular grade: the
/// hardware is the hardware, and a test that expected *no card* would fail on
/// the machine task 23 is waiting for. What it holds is that the machine
/// answers at all, that the answer carries a reason, and that the reason is a
/// sentence a person can read.
#[test]
fn this_machines_own_road_is_measured_and_said() {
    let draws = WhatDrawsHere::on_this_machine();
    let road = Road::of(
        None,
        &draws,
        what_the_runtime_can_use(),
        what_the_pinned_weights_need(),
    );
    let strings = Strings::of(model_words().unwrap());
    let said = road.shown(&strings);

    // Printed so that whoever runs this has the measurement in front of them,
    // which is what a report quotes.
    println!("this machine draws with: {draws:?}");
    for card in draws.cards() {
        println!(
            "  a card made by {}, driver {:?}, memory {:?}",
            card.made_by().named(),
            card.driver(),
            card.memory_bytes(),
        );
    }
    println!("this machine's road: {road:?}\n  said: {said}");

    assert!(!said.trim().is_empty(), "a machine with nothing to say");
    assert!(!said.contains('{'), "a gap nothing filled: {said}");
    // Nothing was loaded, so nothing may claim the card road: the card road is
    // what the runtime reports, never what the bus suggests.
    assert!(
        !road.is_the_card(),
        "no weights were loaded, so no road can be the card's: {road:?}"
    );
    assert_eq!(road.residency(), None);
}

/// **Every way a machine can end up on its processor is a reason, and each is
/// a different sentence.**
///
/// The three the acceptance names — the wrong vendor, no driver, too little
/// memory — and the two that are the state of an ordinary machine: no card at
/// all, and a machine that keeps no list for this to read. None of them is a
/// failure: in all five the model runs, on the processor, and the machine says
/// which and why.
#[test]
fn every_card_the_runtime_cannot_use_is_a_machine_that_runs_and_says_why() {
    let no_card = AMachine::with_no_card();
    let no_driver = AMachine::with_no_card().and_a_card("card0", Vendor::NVIDIA, None, None);
    let another_vendor = AMachine::with_no_card().and_a_card(
        "card0",
        Vendor::INTEL,
        Some("i915"),
        Some(32_000_000_000),
    );
    let too_small = AMachine::with_no_card().and_a_card(
        "card0",
        Vendor::AMD,
        Some("amdgpu"),
        Some(what_the_pinned_weights_need() / 2),
    );

    assert_eq!(
        no_card.road(),
        Road::OnTheProcessor(WhyTheProcessor::NoCard)
    );
    assert_eq!(
        no_driver.road(),
        Road::OnTheProcessor(WhyTheProcessor::NoDriverForTheCard {
            made_by: Vendor::Nvidia
        }),
        "a card with no driver must never read as no card",
    );
    assert_eq!(
        another_vendor.road(),
        Road::OnTheProcessor(WhyTheProcessor::TheRuntimeCannotUseThatCard {
            made_by: Vendor::Intel
        }),
    );
    assert_eq!(
        too_small.road(),
        Road::OnTheProcessor(WhyTheProcessor::NotEnoughOnTheCard {
            the_card_has: what_the_pinned_weights_need() / 2,
            the_weights_need: what_the_pinned_weights_need(),
        }),
    );
    let would_not_say = Road::of(
        None,
        &WhatDrawsHere::among(Path::new("/there-is-no-such-list-on-any-machine")),
        what_the_runtime_can_use(),
        what_the_pinned_weights_need(),
    );
    assert_eq!(
        would_not_say,
        Road::OnTheProcessor(WhyTheProcessor::TheMachineWouldNotSay),
    );

    // Five machines, five sentences, and no two of them the same — a person
    // reading one knows which of the five they are in.
    let strings = Strings::of(model_words().unwrap());
    let mut said: Vec<String> = [
        no_card.road(),
        no_driver.road(),
        another_vendor.road(),
        too_small.road(),
        would_not_say,
    ]
    .iter()
    .map(|road| road.shown(&strings))
    .collect();
    let how_many = said.len();
    said.sort();
    said.dedup();
    assert_eq!(
        said.len(),
        how_many,
        "two machines reading the same: {said:?}"
    );
}

/// **An NVIDIA machine with no NVIDIA driver is told about the driver**, even
/// when there is an integrated processor beside it that the runtime also
/// cannot use.
///
/// The distinction is the task's own, and it is not a detail: the base is
/// rented and unmodified (ADR 0011) and ships no such driver, so this is the
/// likely state of every NVIDIA machine alo OS meets today. A person told *no
/// card* goes looking for hardware they already own.
#[test]
fn a_laptop_with_an_integrated_processor_and_a_driverless_card_is_told_about_the_driver() {
    let machine = AMachine::with_no_card()
        .and_a_card("card0", Vendor::INTEL, Some("i915"), None)
        .and_a_card("card1", Vendor::NVIDIA, None, None);
    assert_eq!(machine.draws().cards().len(), 2);
    assert_eq!(
        machine.road(),
        Road::OnTheProcessor(WhyTheProcessor::NoDriverForTheCard {
            made_by: Vendor::Nvidia
        }),
    );
    let strings = Strings::of(model_words().unwrap());
    let said = machine.road().shown(&strings);
    assert!(said.contains("NVIDIA"), "{said}");
    assert!(said.contains("driver"), "{said}");
    assert!(
        !said.contains("no graphics card"),
        "a machine with a card was told it has none: {said}"
    );
}

/// **What the runtime reports having loaded is what carries into a grade**, so
/// `on_the_gpu_bytes` holds a figure a device reported rather than one
/// somebody typed.
///
/// The residency is read the way a real one is read — over a socket, out of
/// the runtime's own `/api/ps` answer — and travels the whole way into a
/// [`MeasuredOn`] that the catalogue's own rules accept.
#[test]
fn what_a_card_held_is_read_from_the_runtime_and_carried_into_a_grade() {
    let (address, runtime) = a_runtime_answering(
        r#"{"models":[{"name":"phi-3-mini-instruct:latest","size":3100000000,"size_vram":2900000000}]}"#,
    );
    let loaded = Ollama::at(&address, Catalogue::built_in().unwrap())
        .loaded()
        .unwrap();
    runtime.join().unwrap();
    let held = loaded.first().unwrap();

    let machine =
        AMachine::with_no_card().and_a_card("card0", Vendor::NVIDIA, Some("nvidia"), None);
    let road = Road::of(
        Some(held),
        &machine.draws(),
        what_the_runtime_can_use(),
        what_the_pinned_weights_need(),
    );
    assert_eq!(
        road,
        Road::OnTheCard {
            loaded_bytes: 3_100_000_000,
            on_the_gpu_bytes: 2_900_000_000,
        },
    );

    let mut grade = MeasuredOn {
        machine: "a machine with a card, 32 GB".to_owned(),
        date: "2026-09-22".to_owned(),
        runtime: format!("Ollama {}", alo_models::THE_PINNED_RUNTIME),
        drove: Some(19),
        of: Some(20),
        loaded_bytes: None,
        on_the_gpu_bytes: None,
        held_to: None,
        instructions: None,
    };
    road.recorded_in(&mut grade);
    assert_eq!(grade.on_the_gpu_bytes, Some(2_900_000_000));
    assert_eq!(grade.loaded_bytes, Some(3_100_000_000));

    // And the pair the catalogue would refuse is not what this wrote: a grade
    // carrying this residency loads.
    let text = an_entry_stating(&grade);
    let parsed = Catalogue::parse(&text).unwrap();
    let measured = parsed.get("one").unwrap().measured.as_ref().unwrap();
    assert_eq!(measured.on_the_gpu_bytes, Some(2_900_000_000));
}

/// **A machine whose runtime loaded nothing onto the card is the processor
/// road**, however much it loaded in all — and the reason still comes from the
/// machine rather than from the number.
#[test]
fn a_runtime_that_put_nothing_on_the_card_is_the_processor_road() {
    let (address, runtime) = a_runtime_answering(
        r#"{"models":[{"name":"phi-3-mini-instruct:latest","size":3100000000,"size_vram":0}]}"#,
    );
    let loaded = Ollama::at(&address, Catalogue::built_in().unwrap())
        .loaded()
        .unwrap();
    runtime.join().unwrap();

    let machine = AMachine::with_no_card();
    let road = Road::of(
        loaded.first(),
        &machine.draws(),
        what_the_runtime_can_use(),
        what_the_pinned_weights_need(),
    );
    assert_eq!(road, Road::OnTheProcessor(WhyTheProcessor::NoCard));
    assert_eq!(
        road.residency(),
        None,
        "a residency of nought on the card is not a residency to record beside a grade",
    );
}

/// **`min_vram_gb` does not come back as a judge.**
///
/// ADR 0007 took it out of the offering decision — it answers *will this run
/// well on a card* and cannot answer *will this run at all on this laptop* —
/// and this task is exactly the change that could have put it back. What a
/// machine offers is the same list whatever is or is not on its bus, and an
/// entry whose `min_vram_gb` is far above every card here is still offered.
#[test]
fn what_a_machine_offers_does_not_change_with_what_is_on_its_bus() {
    let catalogue = Catalogue::built_in().unwrap();
    let on_sixteen_gigabytes: Vec<&str> = catalogue
        .to_choose_from_on_cpu(16.0)
        .into_iter()
        .map(|model| model.id.as_str())
        .collect();
    assert!(
        !on_sixteen_gigabytes.is_empty(),
        "a laptop with no card must be offered something (ADR 0007)"
    );

    let machines = [
        AMachine::with_no_card(),
        AMachine::with_no_card().and_a_card("card0", Vendor::NVIDIA, None, None),
        AMachine::with_no_card().and_a_card("card0", Vendor::INTEL, Some("i915"), Some(1)),
        AMachine::with_no_card().and_a_card(
            "card0",
            Vendor::AMD,
            Some("amdgpu"),
            Some(24_000_000_000),
        ),
    ];
    for machine in &machines {
        let road = machine.road();
        let offered: Vec<&str> = catalogue
            .to_choose_from_on_cpu(16.0)
            .into_iter()
            .map(|model| model.id.as_str())
            .collect();
        assert_eq!(
            offered, on_sixteen_gigabytes,
            "what this machine offers changed with its bus: {road:?}"
        );
    }

    // And the figure a road holds a card against is the weights' own size, not
    // the catalogue's card figure: the entry below needs four gigabytes of
    // video memory by `min_vram_gb` and 2.4 by its weights, and a card with
    // three is enough for the road.
    let entry = catalogue.get("phi-3-mini-instruct").unwrap();
    assert!(
        f64::from(entry.min_vram_gb) * 1_000_000_000.0 > 3_000_000_000.0,
        "this test needs an entry whose min_vram_gb is above three gigabytes",
    );
    let three_gigabytes = AMachine::with_no_card().and_a_card(
        "card0",
        Vendor::AMD,
        Some("amdgpu"),
        Some(3_000_000_000),
    );
    assert_eq!(
        three_gigabytes.road(),
        Road::OnTheProcessor(WhyTheProcessor::TheRuntimeLeftItThere),
        "min_vram_gb was used to judge a card the weights fit on",
    );
}

/// **Naming a road changes exactly one thing about what goes to the runtime,
/// and naming none changes nothing at all.**
///
/// The second half is the one worth a test. Every question a person's turn
/// asks goes through the door that names no road, and that door must send the
/// request it sent before this field existed — a new field on every request
/// would be a change to the pinned runtime's input that nobody measured
/// (ADR 0006).
#[test]
fn a_question_that_names_no_road_is_the_request_that_was_always_sent() {
    let answer = r#"{"message":{"role":"assistant","content":"{\"format\":1}"}}"#;

    let (address, runtime) = a_runtime_answering(answer);
    Ollama::at(&address, Catalogue::built_in().unwrap())
        .answers_in_the_envelope("what is the next step?", "phi-3-mini-instruct")
        .unwrap();
    let as_it_always_was = compacted(&runtime.join().unwrap());
    assert!(
        !as_it_always_was.contains("options"),
        "a question that named no road carried one: {as_it_always_was}"
    );
    assert!(!as_it_always_was.contains("num_gpu"), "{as_it_always_was}");

    let (address, runtime) = a_runtime_answering(answer);
    Ollama::at(&address, Catalogue::built_in().unwrap())
        .answers_in_the_envelope_on(
            "what is the next step?",
            "phi-3-mini-instruct",
            WhichRoad::TheProcessor,
        )
        .unwrap();
    let on_the_processor = compacted(&runtime.join().unwrap());
    assert!(
        on_the_processor.contains(r#""options":{"num_gpu":0}"#),
        "the processor road was asked for and nothing said so: {on_the_processor}"
    );

    // And the rest of the request is the same one: the road is the only
    // difference between the two, so a comparison of the two roads is a
    // comparison of the roads and not of two different questions.
    assert_eq!(
        on_the_processor.replace(r#","options":{"num_gpu":0}"#, ""),
        as_it_always_was,
    );
}

/// The JSON that went out, with the whitespace taken out of it.
///
/// The head is dropped because its content length and its port differ between
/// two servers for reasons that have nothing to do with a road; the runtime's
/// own client writes the body over several lines, and what this test is about
/// is the fields rather than the formatting.
fn compacted(sent: &str) -> String {
    sent.split("\r\n\r\n")
        .nth(1)
        .unwrap_or_default()
        .chars()
        .filter(|letter| !letter.is_whitespace())
        .collect()
}

/// **A person reads the reason in their own language, and the vendor inside it
/// is left alone** — a vendor is a proper noun, like a filename.
#[test]
fn the_reason_is_read_in_the_language_the_person_reads() {
    let strings = speaking_german(&[(
        words::THE_PROCESSOR_ANOTHER_VENDOR,
        "die Grafikkarte dieses Rechners, hergestellt von {vendor}, ist keine, auf die die \
         Modelllaufzeit Modelle legt, daher läuft das Modell auf dem Prozessor",
    )]);
    let machine = AMachine::with_no_card().and_a_card(
        "card0",
        Vendor::INTEL,
        Some("i915"),
        Some(32_000_000_000),
    );
    let said = machine.road().said(&strings);
    assert!(said.is_translated(), "{said}");
    assert!(said.text().contains("Intel"), "{said}");
    assert!(said.text().contains("Prozessor"), "{said}");

    // And a machine with no translation of this sentence still says it, in the
    // language this repository is written in, rather than saying nothing.
    let untranslated = AMachine::with_no_card().road().said(&strings);
    assert!(!untranslated.is_translated());
    assert!(untranslated.text().contains("processor"), "{untranslated}");
}

/// **Every sentence about a road declares beside everybody else's**, which is
/// the arrangement on a real machine: one vocabulary, one area per crate.
#[test]
fn what_a_machine_says_about_its_road_declares_beside_everybody_elses() {
    let mut vocabulary = Vocabulary::empty();
    vocabulary
        .says(
            alo_strings::Word::saying("desktop.somebody-elses", "somebody else's string")
                .phrase()
                .unwrap(),
        )
        .unwrap();
    declare_into(&mut vocabulary).unwrap();
    for word in words::ABOUT_THE_ROAD {
        assert!(
            vocabulary
                .phrase(&Key::named(word.named()).unwrap())
                .is_some(),
            "{} is not in the machine's vocabulary",
            word.named(),
        );
    }
}

/// An entry stating this grade, as `data/catalogue.toml` states one.
fn an_entry_stating(grade: &MeasuredOn) -> String {
    let loaded = grade.loaded_bytes.unwrap();
    let on_the_gpu = grade.on_the_gpu_bytes.unwrap();
    let drove = grade.drove.unwrap();
    let of = grade.of.unwrap();
    format!(
        "[[model]]\nid = \"one\"\nname = \"One\"\npublisher = \"p\"\nparameters_b = 3.8\n\
         quantisation = \"Q4_K_M\"\nartefact = \"runtime:one-q4_K_M\"\n\
         download_bytes = 2_400_000_000\nmin_vram_gb = 4.0\nmin_ram_gb = 6.0\n\
         on_cpu = \"comfortable\"\ndrives_verbs = \"reliably\"\n\
         upstream = \"https://example.invalid/one\"\n\
         licence = {{ name = \"MIT\", spdx = \"MIT\", commercial_use = \"permitted\" }}\n\
         measured = {{ machine = \"{}\", date = \"{}\", runtime = \"{}\", drove = {drove}, \
         of = {of}, loaded_bytes = {loaded}, on_the_gpu_bytes = {on_the_gpu}, \
         instructions = \"{THE_INSTRUCTIONS}\" }}\n",
        grade.machine, grade.date, grade.runtime,
    )
}

/// The digest the grades in `data/catalogue.toml` name their instructions by,
/// quoted here so the entry above is one the shipped catalogue's own rules
/// accept rather than a shape invented for a test.
const THE_INSTRUCTIONS: &str = "d468e469651d778ae369c53e37816fce62c80f703de729a074bcf8ff44a5adce";
