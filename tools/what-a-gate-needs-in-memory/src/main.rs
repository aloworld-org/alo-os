//! How much memory a gate run needs, measured rather than guessed.
//!
//! Run this beside a gate. It samples how much memory is **available to the
//! guest** and reports the low-water mark, which is what a memory precondition
//! in `alo-kernel-loop` would have to be sized against.
//!
//! ```text
//! what-a-gate-needs-in-memory &
//! cd /root && BRANCH=<a branch> ./gate3.sh
//! ```
//!
//! # Why this exists
//!
//! The supervisor refuses to gate on a filesystem with less than 12 GiB free,
//! because *a build started here would not fail as a build — it fails as a
//! linker that cannot open a file, which reads like a broken change and is not
//! one.* **Memory produces that same shape and there is no refusal for it.** A
//! compiler killed by the kernel's out-of-memory reaper leaves a `signal: 9`
//! in the middle of a crate somebody just edited.
//!
//! A floor picked by eye would be the threshold nobody can test. So it is
//! measured, and this is what measures it — kept in the repository rather than
//! on one machine, because a refusal that says *less than a gate has been
//! measured to need* cites a measurement, and a citation nobody can re-run is a
//! dead citation wearing a number's clothes.
//!
//! # Available, not resident
//!
//! It records the low-water mark of `MemAvailable`, **not** the peak resident
//! size of any one process. What kills a build is the total across many
//! concurrent `rustc`, so a floor taken from the largest single child would
//! admit exactly the machine it exists to refuse. This is the third PC's
//! observation and it is the reason the tool is shaped this way.
//!
//! # What it cannot see
//!
//! **The host.** On a machine where Linux is a guest, `MemAvailable` is the
//! guest's view. A host that is swapping the guest's pages makes everything
//! slow and kills nothing, and nothing measurable from in here distinguishes
//! that from an idle host. So this measures the failure that *kills* a build
//! and is blind to the one that merely drags it — which is a limit the
//! precondition built on it will have to state in its own words, or a passing
//! check gets read as *this machine is healthy*.
//!
//! # A warm cache measures almost nothing
//!
//! The first run of this on the third PC never saw `clippy`, `fmt` or `build`
//! in 336 samples: with 41 GiB of warm artefacts those gates had nothing to
//! compile and finished inside a sampling window. A number from such a run is
//! the cost of the **cheapest** run a machine can have. So [`Seen`] reports
//! which gates were observed working, and a run that never saw the compiling
//! ones says so rather than offering its number as if it meant something.

use std::collections::BTreeSet;
use std::fs;
use std::io::Write as _;
use std::path::Path;
use std::thread::sleep;
use std::time::{Duration, Instant};

/// How often to look. Short enough to catch a gate that finishes quickly,
/// long enough that the sampler is not itself a load.
const EVERY: Duration = Duration::from_millis(500);

/// How long to wait for a build to appear before giving up on ever seeing one.
const PATIENCE: Duration = Duration::from_secs(120);

/// What was seen working, so a number from a run that compiled nothing says so.
#[derive(Debug, Default)]
struct Seen {
    /// The cargo subcommands observed running, in whatever order they appeared.
    gates: BTreeSet<String>,
    /// The most `rustc` processes running at once.
    most_rustc: usize,
}

impl Seen {
    /// Whether anything that actually compiles was observed.
    ///
    /// **The guard whose first version ended before its subject began.** The
    /// third PC's first sampler asked *have I seen a build* and was satisfied by
    /// its first sample, because available memory is always a little under
    /// total — so it concluded thirty seconds in, before the gate started. A
    /// measurement that ends before its subject begins.
    ///
    /// So the question is not *has memory moved* but *have I seen a process
    /// whose job is to compile*, which cannot be true before one exists.
    fn saw_something_compile(&self) -> bool {
        self.most_rustc > 0
    }

    /// Whether the gates that do the heavy compiling were seen.
    ///
    /// A run where these are absent had a warm cache and its number is the cost
    /// of the cheapest run, not of a gate.
    fn saw_the_compiling_gates(&self) -> bool {
        ["build", "clippy", "test", "nextest"]
            .iter()
            .any(|which| self.gates.contains(*which))
    }
}

/// One look at the machine.
struct ALook {
    /// Bytes the kernel says are available to this guest.
    available: u64,
    /// Which cargo subcommand is running, if one is.
    gate: Option<String>,
    /// How many `rustc` processes are running.
    rustc: usize,
}

fn main() {
    let total = match meminfo("MemTotal") {
        Some(it) => it,
        None => {
            eprintln!(
                "this machine's /proc/meminfo could not be read for MemTotal, so nothing was \
                 measured. A sampler that reported a number it could not take would be worse \
                 than one that refuses."
            );
            std::process::exit(1);
        }
    };

    println!("watching. Start a gate now; this ends when the gate does.");
    println!("MemTotal {} MiB", total / (1024 * 1024));

    let began = Instant::now();
    let mut lowest = total;
    let mut lowest_during: Option<String> = None;
    let mut seen = Seen::default();
    let mut samples: u64 = 0;
    let mut was_building = false;

    loop {
        let Some(look) = a_look() else {
            eprintln!("this machine stopped answering about its memory; stopping.");
            break;
        };
        samples += 1;

        if let Some(gate) = look.gate.clone() {
            seen.gates.insert(gate);
        }
        seen.most_rustc = seen.most_rustc.max(look.rustc);

        if look.available < lowest {
            lowest = look.available;
            lowest_during.clone_from(&look.gate);
        }

        let building = look.rustc > 0;
        if building {
            was_building = true;
        }

        // Ends when a build has been seen and has finished — never on a
        // threshold, and never before a build has been seen at all.
        if was_building && !building {
            break;
        }
        if !was_building && began.elapsed() > PATIENCE {
            eprintln!(
                "no compiler ran in {} seconds, so nothing was measured. Start a gate while \
                 this is running.",
                PATIENCE.as_secs()
            );
            std::process::exit(1);
        }

        let _ = std::io::stdout().flush();
        sleep(EVERY);
    }

    report(
        total,
        lowest,
        lowest_during.as_deref(),
        &seen,
        samples,
        began,
    );
}

/// Say what was measured, and refuse to offer a number a run did not earn.
fn report(
    total: u64,
    lowest: u64,
    during: Option<&str>,
    seen: &Seen,
    samples: u64,
    began: Instant,
) {
    let mib = |bytes: u64| bytes / (1024 * 1024);

    println!();
    println!("samples          {samples} at {} ms", EVERY.as_millis());
    println!("over             {} s", began.elapsed().as_secs());
    println!("MemTotal         {} MiB", mib(total));
    println!("lowest available {} MiB", mib(lowest));
    println!(
        "so a gate cost   {} MiB at its peak",
        mib(total.saturating_sub(lowest))
    );
    println!("the peak fell in {}", during.unwrap_or("nothing named"));
    println!("most rustc at once  {}", seen.most_rustc);
    println!(
        "gates seen working  {}",
        if seen.gates.is_empty() {
            "none".to_owned()
        } else {
            seen.gates.iter().cloned().collect::<Vec<_>>().join(", ")
        }
    );

    if !seen.saw_something_compile() {
        println!();
        println!(
            "NOT A MEASUREMENT: no compiler was ever seen, so the number above is this \
             machine idling."
        );
        return;
    }
    if !seen.saw_the_compiling_gates() {
        println!();
        println!(
            "WARM CACHE: the gates that compile were never seen working, so this is the cost \
             of the cheapest run this machine can have rather than the cost of a gate. Clear \
             the build directory and measure again before a floor is set from it."
        );
        return;
    }
    println!();
    println!(
        "A floor set from this is a statement about this workspace, at this parallelism, on \
         this date — re-measure when any of the three changes."
    );
}

/// One sample: available memory, the cargo subcommand running, and how many
/// compilers are working.
fn a_look() -> Option<ALook> {
    let available = meminfo("MemAvailable")?;
    let mut gate = None;
    let mut rustc = 0_usize;

    let Ok(entries) = fs::read_dir("/proc") else {
        return Some(ALook {
            available,
            gate,
            rustc,
        });
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if !path.join("cmdline").exists() {
            continue;
        }
        let Ok(raw) = fs::read(path.join("cmdline")) else {
            continue;
        };
        let words: Vec<String> = raw
            .split(|byte| *byte == 0)
            .filter(|part| !part.is_empty())
            .map(|part| String::from_utf8_lossy(part).into_owned())
            .collect();
        let Some(program) = words.first() else {
            continue;
        };
        let name = Path::new(program)
            .file_name()
            .map_or_else(String::new, |it| it.to_string_lossy().into_owned());

        if name == "rustc" {
            rustc += 1;
        }
        // `cargo <subcommand>` names the gate. Taken from the running process
        // rather than from a list here, so it cannot drift from what the
        // supervisor actually runs.
        if name == "cargo"
            && gate.is_none()
            && let Some(subcommand) = words.get(1)
            && !subcommand.starts_with('-')
        {
            gate = Some(subcommand.clone());
        }
    }
    Some(ALook {
        available,
        gate,
        rustc,
    })
}

/// A field of `/proc/meminfo`, in bytes. The file reports kibibytes.
fn meminfo(field: &str) -> Option<u64> {
    let said = fs::read_to_string("/proc/meminfo").ok()?;
    said.lines()
        .find(|line| line.starts_with(field))
        .and_then(|line| line.split_whitespace().nth(1))
        .and_then(|number| number.parse::<u64>().ok())
        .map(|kib| kib.saturating_mul(1024))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **A run that saw no compiler has measured nothing**, whatever the
    /// numbers look like — the guard whose first version was satisfied before
    /// the gate began.
    #[test]
    fn nothing_compiling_is_not_a_measurement() {
        let idle = Seen::default();
        assert!(!idle.saw_something_compile());

        let worked = Seen {
            most_rustc: 1,
            ..Seen::default()
        };
        assert!(worked.saw_something_compile());
    }

    /// **A warm cache is told apart from a real run.** Seeing only `doc` means
    /// the compiling gates had nothing to do, which is the cheapest run rather
    /// than a gate.
    #[test]
    fn a_run_that_only_saw_doc_is_a_warm_cache() {
        let mut warm = Seen {
            most_rustc: 4,
            ..Seen::default()
        };
        warm.gates.insert("doc".to_owned());
        assert!(warm.saw_something_compile());
        assert!(
            !warm.saw_the_compiling_gates(),
            "a run that only documented is not a measurement of a gate"
        );

        warm.gates.insert("clippy".to_owned());
        assert!(warm.saw_the_compiling_gates());
    }

    /// Any of the compiling gates is enough, because which of them a machine
    /// shows depends on what its cache already holds.
    #[test]
    fn any_compiling_gate_counts() {
        for which in ["build", "clippy", "test", "nextest"] {
            let mut seen = Seen {
                most_rustc: 2,
                ..Seen::default()
            };
            seen.gates.insert(which.to_owned());
            assert!(seen.saw_the_compiling_gates(), "{which}");
        }
    }

    /// **The machine can be asked about its own memory**, and the number is a
    /// plausible one rather than a parse of the wrong column.
    #[test]
    fn this_machine_can_say_how_much_memory_it_has() {
        let Some(total) = meminfo("MemTotal") else {
            return; // Not Linux; nothing to assert.
        };
        assert!(total > 256 * 1024 * 1024, "MemTotal read as {total} bytes");
        if let Some(available) = meminfo("MemAvailable") {
            assert!(available <= total, "available above total");
        }
    }

    /// A field that is not there answers nothing rather than guessing.
    #[test]
    fn a_field_that_is_not_there_is_not_invented() {
        assert_eq!(meminfo("ThisIsNotAFieldOfMeminfo"), None);
    }
}
