//! *What is running, and what it is using* — read from the kernel that is
//! actually running, and checked against it.
//!
//! The unit tests take the pieces apart against a kernel they wrote out.
//! This file opens the real `/proc` and holds the plan's sentences to the
//! machine the tests are on:
//!
//! | The acceptance | The test |
//! |---|---|
//! | every number is read from `/proc` and named for its file, so a test can open that file and compare | [`every_number_is_named_for_the_file_it_came_from_and_agrees_with_it`] |
//! | asking twice gives a rate, with the interval passed in rather than read from a clock | [`asking_twice_gives_a_rate_over_the_interval_passed_in`] |
//! | a process that exited between two readings is gone rather than zero | [`a_process_that_ended_between_readings_is_gone_rather_than_zero`] |
//! | a process that began between two readings has no rate yet | [`a_process_that_began_between_readings_has_no_rate_yet`] |
//! | a line the kernel does not keep for a process is not said, not zero | [`a_number_the_kernel_does_not_keep_is_not_said_and_agrees_with_the_file`] |
//!
//! Every comparison here is against the file the number names, re-read by
//! this test with its own few lines of parsing. That is deliberate: the
//! promise is that a person with `cat` sees the same number, and the test is
//! that person.

#![cfg(target_os = "linux")]
#![expect(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::io::Write;
use std::path::Path;
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

use alo_measuring::{Number, Reading, Source};

/// A process that will sit still for as long as this test needs.
fn a_sleeping_child() -> Child {
    let child = Command::new("sleep")
        .arg("60")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("a sleep to spawn");
    // Until it has replaced itself with `sleep`, its files are the parent's
    // image; a moment is enough.
    std::thread::sleep(Duration::from_millis(200));
    child
}

/// The file a number names, read now.
fn the_file(number: &Number) -> String {
    let at = number.from().file();
    std::fs::read_to_string(at)
        .unwrap_or_else(|why| panic!("{} could not be read: {why}", at.display()))
}

/// `Name:   N kB` on the line named, in bytes — the test's own reading of
/// the file, not the crate's.
fn kilobytes_on(text: &str, name: &str) -> Option<u64> {
    let line = text
        .lines()
        .find(|line| line.starts_with(name) && line.as_bytes().get(name.len()) == Some(&b':'))?;
    let mut words = line.split_whitespace().skip(1);
    let kb: u64 = words.next()?.parse().ok()?;
    assert_eq!(words.next(), Some("kB"));
    Some(kb * 1024)
}

/// `name: N` on the line named.
fn number_on(text: &str, name: &str) -> u64 {
    let line = text
        .lines()
        .find(|line| line.starts_with(name) && line.as_bytes().get(name.len()) == Some(&b':'))
        .unwrap_or_else(|| panic!("no {name} line in {text}"));
    line.split_whitespace().nth(1).unwrap().parse().unwrap()
}

/// `utime + stime` of a process's `stat` line: fields 14 and 15, counted
/// from after the last `)`.
fn ticks_of(stat: &str) -> u64 {
    let (_, after) = stat.rsplit_once(')').unwrap();
    let fields: Vec<u64> = after
        .split_whitespace()
        .skip(11)
        .take(2)
        .map(|field| field.parse().unwrap())
        .collect();
    fields.iter().sum()
}

/// The sum of the machine's `cpu` line.
fn machine_ticks(stat: &str) -> u64 {
    let line = stat.lines().find(|line| line.starts_with("cpu ")).unwrap();
    line.split_whitespace()
        .skip(1)
        .map(|field| field.parse::<u64>().unwrap())
        .sum()
}

/// Bytes received and sent on every interface but `lo`, from `net/dev`.
fn crossed(dev: &str) -> (u64, u64) {
    let mut received = 0;
    let mut sent = 0;
    for line in dev.lines() {
        let Some((name, rest)) = line.split_once(':') else {
            continue;
        };
        if name.trim() == "lo" {
            continue;
        }
        let fields: Vec<u64> = rest
            .split_whitespace()
            .map(|field| field.parse().unwrap())
            .collect();
        received += fields.first().unwrap();
        sent += fields.get(8).unwrap();
    }
    (received, sent)
}

/// A number's file, read before and after, holds the number between them.
///
/// A counter the kernel keeps only ever grows, so a reading taken between two
/// reads of its file lies between them; a size that did not change is equal
/// to both. Either way the number came out of that file.
fn between(before: u64, number: &Number, after: u64, what: &str) {
    let value = number
        .value()
        .unwrap_or_else(|| panic!("{what} was not a number: {number:?}"));
    assert!(
        before <= value && value <= after,
        "{what}: the file said {before} then {after}, and the number is {value}"
    );
}

/// **Every number a process has is read from the file it is named for.**
///
/// A sleeping child is read six ways, and each number is between what the
/// named file said just before the reading and just after it — which for a
/// process doing nothing is equality, and for a counter the whole machine
/// shares is the only honest comparison. The file each number names is under
/// `/proc/<pid>`, by the kernel's own field name.
#[test]
fn every_number_is_named_for_the_file_it_came_from_and_agrees_with_it() {
    let mut child = a_sleeping_child();
    let pid = child.id();
    let dir = Path::new("/proc").join(pid.to_string());
    let read_all = || {
        (
            std::fs::read_to_string(dir.join("status")).unwrap(),
            std::fs::read_to_string(dir.join("stat")).unwrap(),
            std::fs::read_to_string(dir.join("io")).unwrap(),
            std::fs::read_to_string(dir.join("net/dev")).unwrap(),
            std::fs::read_to_string("/proc/stat").unwrap(),
        )
    };

    let before = read_all();
    let reading = Reading::now().unwrap();
    let after = read_all();
    let process = reading.process(pid).expect("the child is in the list");
    assert_eq!(process.name, "sleep", "the name is the kernel's, from stat");

    // Each number names its file and its field, and the file agrees.
    let named = [
        (&process.memory, "status", "VmRSS"),
        (&process.ticks, "stat", "utime+stime"),
        (&process.read, "io", "rchar"),
        (&process.written, "io", "wchar"),
        (&process.received, "net/dev", "bytes received"),
        (&process.sent, "net/dev", "bytes sent"),
    ];
    for (number, file, field) in named {
        assert_eq!(number.from(), &Source::of(dir.join(file), field));
        assert_eq!(number.from().file(), dir.join(file));
    }
    between(
        kilobytes_on(&before.0, "VmRSS").unwrap(),
        &process.memory,
        kilobytes_on(&after.0, "VmRSS").unwrap(),
        "VmRSS",
    );
    between(
        ticks_of(&before.1),
        &process.ticks,
        ticks_of(&after.1),
        "utime+stime",
    );
    between(
        number_on(&before.2, "rchar"),
        &process.read,
        number_on(&after.2, "rchar"),
        "rchar",
    );
    between(
        number_on(&before.2, "wchar"),
        &process.written,
        number_on(&after.2, "wchar"),
        "wchar",
    );
    between(
        crossed(&before.3).0,
        &process.received,
        crossed(&after.3).0,
        "bytes received",
    );
    between(
        crossed(&before.3).1,
        &process.sent,
        crossed(&after.3).1,
        "bytes sent",
    );

    // And the machine's own line, which a share is a share of.
    assert_eq!(
        reading.machine().ticks.from,
        Source::of("/proc/stat", "cpu")
    );
    between(
        machine_ticks(&before.4),
        &Number::Known(reading.machine().ticks.clone()),
        machine_ticks(&after.4),
        "cpu",
    );
    assert_eq!(
        reading.machine().memory_total.from(),
        &Source::of("/proc/meminfo", "MemTotal")
    );
    assert_eq!(
        reading.machine().memory_total.value(),
        kilobytes_on(&the_file(&reading.machine().memory_total), "MemTotal"),
        "MemTotal does not change while a machine is up"
    );

    // The namespace the two network counts belong to is the one the kernel
    // links the child to.
    let link = std::fs::read_link(dir.join("ns/net")).unwrap();
    let namespace: u64 = link
        .to_string_lossy()
        .trim_start_matches("net:[")
        .trim_end_matches(']')
        .parse()
        .unwrap();
    assert_eq!(process.namespace, Some(namespace));

    child.kill().unwrap();
    child.wait().unwrap();
}

/// **Asking twice gives a rate, over the interval passed in.**
///
/// Between two readings this process writes a known number of bytes and
/// keeps one thread busy. The rates are exactly the differences between the
/// two readings' totals over the interval the test says — and saying a
/// different interval gives a different rate from the same two readings,
/// because nothing in the crate consulted a clock.
#[test]
fn asking_twice_gives_a_rate_over_the_interval_passed_in() {
    let me = std::process::id();
    let scratch = std::env::temp_dir().join(format!("alo-measuring-written-{me}"));
    let earlier = Reading::now().unwrap();

    // Work: four mebibytes through a file, and a thread busy for a while.
    const BYTES: u64 = 4 * 1024 * 1024;
    let mut file = std::fs::File::create(&scratch).unwrap();
    file.write_all(&vec![7u8; usize::try_from(BYTES).unwrap()])
        .unwrap();
    file.sync_all().unwrap();
    drop(file);
    let began = Instant::now();
    let mut spun: u64 = 0;
    while began.elapsed() < Duration::from_millis(400) {
        spun = spun.wrapping_mul(6_364_136_223_846_793_005).wrapping_add(1);
    }
    assert_ne!(spun, 1, "the loop ran");
    drop(std::fs::remove_file(&scratch));

    let later = Reading::now().unwrap();
    let (then, now) = (earlier.process(me).unwrap(), later.process(me).unwrap());
    let written_delta = now.written.value().unwrap() - then.written.value().unwrap();
    assert!(
        written_delta >= BYTES,
        "{written_delta} bytes were written, at least {BYTES} expected"
    );
    let ticks_delta = now.ticks.value().unwrap() - then.ticks.value().unwrap();
    assert!(ticks_delta > 0, "the busy thread cost some processor time");
    let machine_delta = later.machine().ticks.value - earlier.machine().ticks.value;

    for seconds in [1_u64, 4, 60] {
        let interval = Duration::from_secs(seconds);
        let running = later.since(&earlier, interval).unwrap();
        assert_eq!(running.interval(), interval);
        let mine = running.process(me).unwrap();
        assert_eq!(
            mine.written.value(),
            Some(written_delta / seconds),
            "over {seconds}s"
        );
        assert_eq!(mine.written.from(), now.written.from());
        assert_eq!(
            mine.processor.value(),
            Some(ticks_delta * 1000 / machine_delta)
        );
        assert!(
            mine.processor.value().unwrap() > 0,
            "a share of the processor"
        );
        assert_eq!(
            mine.memory.value(),
            now.memory.value(),
            "memory is a size now, not a rate"
        );
    }
}

/// **A process that ended between two readings is gone, not zero.**
#[test]
fn a_process_that_ended_between_readings_is_gone_rather_than_zero() {
    let mut child = a_sleeping_child();
    let pid = child.id();
    let earlier = Reading::now().unwrap();
    assert!(earlier.process(pid).is_some(), "the child was running");

    child.kill().unwrap();
    child.wait().unwrap();
    std::thread::sleep(Duration::from_millis(50));
    let later = Reading::now().unwrap();

    let running = later.since(&earlier, Duration::from_millis(50)).unwrap();
    assert!(running.process(pid).is_none(), "not a row of zeros");
    let gone = running
        .gone()
        .iter()
        .find(|gone| gone.pid == pid)
        .expect("the child is in the list of what ended");
    assert_eq!(gone.name, "sleep");
}

/// **A process that began between two readings has no rate yet**, and says
/// so in place of every rate; its memory, a size now, is shown.
#[test]
fn a_process_that_began_between_readings_has_no_rate_yet() {
    let earlier = Reading::now().unwrap();
    let mut child = a_sleeping_child();
    let later = Reading::now().unwrap();

    let running = later.since(&earlier, Duration::from_millis(200)).unwrap();
    let new = running.process(child.id()).expect("the child is running");
    for rate in [
        &new.processor,
        &new.read,
        &new.written,
        &new.received,
        &new.sent,
    ] {
        assert!(matches!(rate, Number::NotYet { .. }), "{rate:?}");
    }
    assert!(new.memory.value().is_some_and(|bytes| bytes > 0));
    assert!(!running.gone().iter().any(|gone| gone.pid == child.id()));

    child.kill().unwrap();
    child.wait().unwrap();
}

/// **A line the kernel does not keep for a process is *not said*, not zero.**
///
/// Every process in a reading whose `status` has no `VmRSS` — a thread of
/// the kernel's own — is reported as not said, and every one whose file has
/// the line is reported with a number: the reading agrees with the files, row
/// by row, for whatever this machine happens to be running.
#[test]
fn a_number_the_kernel_does_not_keep_is_not_said_and_agrees_with_the_file() {
    let reading = Reading::now().unwrap();
    let mut not_said = 0;
    for process in reading.processes() {
        let Ok(status) = std::fs::read_to_string(process.memory.from().file()) else {
            continue; // ended since the reading; nothing to compare against
        };
        match &process.memory {
            Number::Known(_) => assert!(
                kilobytes_on(&status, "VmRSS").is_some(),
                "pid {}",
                process.pid
            ),
            Number::NotSaid { from } => {
                assert!(
                    kilobytes_on(&status, "VmRSS").is_none(),
                    "pid {}",
                    process.pid
                );
                assert_eq!(from.field, "VmRSS");
                not_said += 1;
            }
            other => panic!("pid {}: {other:?}", process.pid),
        }
    }
    if let Some(kthreadd) = reading
        .process(2)
        .filter(|process| process.name == "kthreadd")
    {
        assert!(
            matches!(kthreadd.memory, Number::NotSaid { .. }),
            "{:?}",
            kthreadd.memory
        );
        assert!(not_said > 0);
    }
}
