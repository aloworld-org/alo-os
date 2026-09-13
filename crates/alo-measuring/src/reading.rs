//! One reading of the machine, and two readings made into an answer.
//!
//! A [`Reading`] is every total the kernel keeps at one moment. It is not yet
//! the answer to *what is it using?* — a total since a process began says
//! nothing about now — and [`Reading::since`] is what turns two of them into
//! one: rates over the interval the caller says passed, a share of the
//! processor, and the processes that ended in between listed as gone.
//!
//! # A pid is not a process
//!
//! The kernel reuses pids. A process that ended and another that took its
//! number between two readings would, matched by pid alone, look like one
//! process whose counters ran backwards. They are told apart by when they
//! started, which `/proc/<pid>/stat` records in ticks since boot and which no
//! two processes with one pid can share.

use std::collections::HashMap;
use std::path::Path;
use std::time::Duration;

use crate::kernel::Kernel;
use crate::rating;
use crate::refusing::NotMeasured;
use crate::running::{Gone, Network, Process, Running};
use crate::sampled::{Machine, Sampled};
use crate::source::Number;

/// Everything the kernel says at one moment.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Reading {
    /// The machine.
    machine: Machine,
    /// Every process, in pid order.
    processes: Vec<Sampled>,
}

impl Reading {
    /// The kernel, read now.
    ///
    /// # Errors
    ///
    /// [`NotMeasured::NotOnThisHost`] anywhere that is not Linux, and
    /// [`NotMeasured::Unreadable`] when `/proc` or `/proc/stat` cannot be
    /// read. A process that cannot be read is left out rather than refused;
    /// a number the kernel would not give is a [`Number`] saying so.
    pub fn now() -> Result<Self, NotMeasured> {
        #[cfg(target_os = "linux")]
        {
            Self::of_kernel(&crate::kernel::Disk, std::path::Path::new("/proc"))
        }
        #[cfg(not(target_os = "linux"))]
        {
            Err(NotMeasured::NotOnThisHost)
        }
    }

    /// A kernel, read through the three things this crate asks of one.
    ///
    /// [`Self::now`] is this with the running kernel and `/proc`. It is public
    /// so that a [`Kernel`] a test wrote out — a directory of files, one of
    /// them unreadable, one process gone between the listing and its reads —
    /// can be measured on any host by exactly the code that measures the
    /// real one. What a reading holds is still only what a kernel said:
    /// there is no way to write a process into one except through a
    /// [`Kernel`], and choosing which kernel is not editing its facts.
    ///
    /// # Errors
    ///
    /// [`NotMeasured::Unreadable`] when `proc` cannot be listed or its `stat`
    /// cannot be read.
    pub fn of_kernel(kernel: &dyn Kernel, proc: &Path) -> Result<Self, NotMeasured> {
        crate::sampling::everything(kernel, proc)
    }

    /// A reading, from what was sampled. Made by `crate::sampling` and by
    /// nothing else: a reading is what the kernel said, not what a caller
    /// wrote.
    pub(crate) fn of(machine: Machine, processes: Vec<Sampled>) -> Self {
        Self { machine, processes }
    }

    /// The machine.
    #[must_use]
    pub fn machine(&self) -> &Machine {
        &self.machine
    }

    /// Every process, in pid order.
    #[must_use]
    pub fn processes(&self) -> &[Sampled] {
        &self.processes
    }

    /// One process, by pid.
    #[must_use]
    pub fn process(&self, pid: u32) -> Option<&Sampled> {
        self.processes.iter().find(|process| process.pid == pid)
    }

    /// This reading against an earlier one, as rates over `interval`.
    ///
    /// The interval is the caller's: whoever took the two readings knows how
    /// far apart they were, and this crate does not read a clock. A process
    /// in both readings gets rates; one only in this reading gets
    /// [`Number::NotYet`] for each rate and its memory as it is; one only in
    /// the earlier is [`Gone`]. A pid in both with a different start is two
    /// processes, one gone and one new.
    ///
    /// # Errors
    ///
    /// [`NotMeasured::NoInterval`] for an interval of nought, and
    /// [`NotMeasured::SameMoment`] when the machine's own processor count did
    /// not move between the readings — the same reading twice, or two within
    /// one tick. Neither is a rate.
    pub fn since(&self, earlier: &Self, interval: Duration) -> Result<Running, NotMeasured> {
        if interval.is_zero() {
            return Err(NotMeasured::NoInterval);
        }
        let machine_ticks = self
            .machine
            .ticks
            .value
            .saturating_sub(earlier.machine.ticks.value);
        if machine_ticks == 0 {
            return Err(NotMeasured::SameMoment);
        }

        let mut in_namespace: HashMap<u64, usize> = HashMap::new();
        for process in &self.processes {
            if let Some(namespace) = process.namespace {
                *in_namespace.entry(namespace).or_default() += 1;
            }
        }

        let processes = self
            .processes
            .iter()
            .map(|now| {
                let then = earlier
                    .process(now.pid)
                    .filter(|then| then.started == now.started);
                let shared_with = now
                    .namespace
                    .and_then(|namespace| in_namespace.get(&namespace))
                    .map_or(0, |count| count.saturating_sub(1));
                let rate = |later: &Number, field: fn(&Sampled) -> &Number| match then {
                    Some(then) => rating::per_second(later, field(then), interval),
                    None => Number::NotYet {
                        from: later.from().clone(),
                    },
                };
                Process {
                    pid: now.pid,
                    name: now.name.clone(),
                    memory: now.memory.clone(),
                    processor: match then {
                        Some(then) => rating::share(&now.ticks, &then.ticks, machine_ticks),
                        None => Number::NotYet {
                            from: now.ticks.from().clone(),
                        },
                    },
                    read: rate(&now.read, |sampled| &sampled.read),
                    written: rate(&now.written, |sampled| &sampled.written),
                    received: rate(&now.received, |sampled| &sampled.received),
                    sent: rate(&now.sent, |sampled| &sampled.sent),
                    network: Network {
                        namespace: now.namespace,
                        shared_with,
                    },
                }
            })
            .collect();

        let gone = earlier
            .processes
            .iter()
            .filter(|then| {
                self.process(then.pid)
                    .is_none_or(|now| now.started != then.started)
            })
            .map(|then| Gone {
                pid: then.pid,
                name: then.name.clone(),
            })
            .collect();

        Ok(Running::of(
            interval,
            processes,
            gone,
            self.machine.memory_total.clone(),
            self.machine.memory_available.clone(),
        ))
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::sampling::everything;
    use crate::sampling::tests::{Totals, Written, proc};
    use crate::source::Source;

    /// A reading of a written kernel.
    fn read(kernel: &Written) -> Reading {
        everything(kernel, proc()).unwrap()
    }

    /// The totals a process has after doing some work: every counter up by
    /// a round number.
    fn later_totals() -> Totals {
        Totals {
            ticks: 100 + 50,
            memory_kb: 2000 + 1000,
            read: 3000 + 2000,
            written: 4000 + 8000,
            received: 5000 + 500,
            sent: 6000 + 250,
            namespace: 4_026_531_840,
        }
    }

    /// **Asking twice gives a rate over the interval passed in**, and the
    /// numbers are exactly the differences over that interval — including
    /// when the interval given is not the one that really passed, because
    /// this crate has no clock to say otherwise.
    #[test]
    fn two_readings_give_rates_over_the_interval_passed_in() {
        let mut earlier = Written::a_machine(1000);
        earlier.process(42, "cat", 500, Totals::round());
        let mut later = Written::a_machine(1000 + 400);
        later.process(42, "cat", 500, later_totals());
        let (earlier, later) = (read(&earlier), read(&later));

        let running = later.since(&earlier, Duration::from_secs(2)).unwrap();
        assert_eq!(running.interval(), Duration::from_secs(2));
        let process = running.process(42).unwrap();
        assert_eq!(process.name, "cat");
        assert_eq!(process.memory.value(), Some(3000 * 1024), "memory is now");
        assert_eq!(process.processor.value(), Some(50 * 1000 / 400));
        assert_eq!(process.read.value(), Some(2000 / 2));
        assert_eq!(process.written.value(), Some(8000 / 2));
        assert_eq!(process.received.value(), Some(500 / 2));
        assert_eq!(process.sent.value(), Some(250 / 2));
        assert_eq!(process.written.from(), &Source::of("/proc/42/io", "wchar"));
        assert!(running.gone().is_empty());

        let ten = later.since(&earlier, Duration::from_secs(10)).unwrap();
        assert_eq!(ten.process(42).unwrap().written.value(), Some(8000 / 10));
    }

    /// **A process that exited between two readings is gone, not zero**: in
    /// the second list by name, and not in the first at all.
    #[test]
    fn a_process_that_exited_between_readings_is_gone_rather_than_zero() {
        let mut earlier = Written::a_machine(1000);
        earlier.process(42, "stays", 500, Totals::round());
        earlier.process(43, "leaves", 501, Totals::round());
        let mut later = Written::a_machine(1100);
        later.process(42, "stays", 500, later_totals());
        let running = read(&later)
            .since(&read(&earlier), Duration::from_secs(1))
            .unwrap();

        assert_eq!(
            running.gone(),
            [Gone {
                pid: 43,
                name: "leaves".to_owned()
            }]
        );
        assert!(running.process(43).is_none());
        assert_eq!(running.processes().len(), 1);
    }

    /// **A process that began between two readings has no rate yet**, and
    /// says so rather than showing a rate over a beginning it did not have;
    /// its memory, which is a size now and not a rate, is shown.
    #[test]
    fn a_process_that_began_between_readings_has_no_rate_yet() {
        let earlier = Written::a_machine(1000);
        let mut later = Written::a_machine(1100);
        later.process(44, "new", 1050, Totals::round());
        let running = read(&later)
            .since(&read(&earlier), Duration::from_secs(1))
            .unwrap();
        let process = running.process(44).unwrap();
        assert_eq!(process.memory.value(), Some(2000 * 1024));
        for rate in [
            &process.processor,
            &process.read,
            &process.written,
            &process.received,
            &process.sent,
        ] {
            assert!(matches!(rate, Number::NotYet { .. }), "{rate:?}");
            assert_eq!(rate.value(), None);
        }
        assert_eq!(
            process.written.from(),
            &Source::of("/proc/44/io", "wchar"),
            "even a rate that is not there yet names its file"
        );
    }

    /// **A pid the kernel reused is two processes.** Matched by pid alone the
    /// counters would run backwards; matched by start as well, the first is
    /// gone and the second has no rate yet.
    #[test]
    fn a_reused_pid_is_one_process_gone_and_another_not_yet() {
        let mut earlier = Written::a_machine(1000);
        earlier.process(42, "first", 500, later_totals());
        let mut later = Written::a_machine(1100);
        later.process(42, "second", 1050, Totals::round());
        let running = read(&later)
            .since(&read(&earlier), Duration::from_secs(1))
            .unwrap();
        assert_eq!(
            running.gone(),
            [Gone {
                pid: 42,
                name: "first".to_owned()
            }]
        );
        let second = running.process(42).unwrap();
        assert_eq!(second.name, "second");
        assert!(matches!(second.written, Number::NotYet { .. }));
    }

    /// **No interval is refused, and so is no time having passed.** Neither
    /// is a rate, and neither is a list of zeros.
    #[test]
    fn no_interval_and_the_same_moment_are_both_refused() {
        let mut kernel = Written::a_machine(1000);
        kernel.process(42, "cat", 500, Totals::round());
        let reading = read(&kernel);
        assert_eq!(
            reading.since(&reading, Duration::ZERO).unwrap_err(),
            NotMeasured::NoInterval
        );
        assert_eq!(
            reading.since(&reading, Duration::from_secs(1)).unwrap_err(),
            NotMeasured::SameMoment
        );
    }

    /// **Whose network a count is, is said beside it.** Two processes in one
    /// namespace each share with one other; one in a namespace of its own
    /// shares with nobody.
    #[test]
    fn a_shared_network_count_says_how_many_share_it() {
        let alone = Totals {
            namespace: 7,
            ..Totals::round()
        };
        let mut earlier = Written::a_machine(1000);
        earlier.process(1, "a", 1, Totals::round());
        earlier.process(2, "b", 2, Totals::round());
        earlier.process(3, "c", 3, alone);
        let mut later = Written::a_machine(1100);
        later.process(1, "a", 1, Totals::round());
        later.process(2, "b", 2, Totals::round());
        later.process(3, "c", 3, alone);
        let running = read(&later)
            .since(&read(&earlier), Duration::from_secs(1))
            .unwrap();
        assert_eq!(
            running.process(1).unwrap().network,
            Network {
                namespace: Some(4_026_531_840),
                shared_with: 1
            }
        );
        assert_eq!(running.process(2).unwrap().network.shared_with, 1);
        assert_eq!(
            running.process(3).unwrap().network,
            Network {
                namespace: Some(7),
                shared_with: 0
            }
        );
    }

    /// The machine's memory travels into the answer as it was read.
    #[test]
    fn the_machines_memory_is_in_the_answer() {
        let mut earlier = Written::a_machine(1000);
        earlier.process(42, "cat", 500, Totals::round());
        let mut later = Written::a_machine(1100);
        later.process(42, "cat", 500, Totals::round());
        let running = read(&later)
            .since(&read(&earlier), Duration::from_secs(1))
            .unwrap();
        assert_eq!(running.memory_total().value(), Some(16_000_000 * 1024));
        assert_eq!(running.memory_available().value(), Some(12_000_000 * 1024));
    }

    /// **On this host, the kernel is read.** This process is in its own list.
    #[cfg(target_os = "linux")]
    #[test]
    fn on_linux_the_kernel_is_read_and_this_process_is_in_the_list() {
        let reading = Reading::now().unwrap();
        let me = reading.process(std::process::id()).unwrap();
        assert!(me.memory.value().is_some_and(|bytes| bytes > 0));
    }

    /// **On any other host, nothing is measured** — the refusal, not a
    /// reading of nothing.
    #[cfg(not(target_os = "linux"))]
    #[test]
    fn on_any_other_host_nothing_is_measured() {
        assert_eq!(Reading::now().unwrap_err(), NotMeasured::NotOnThisHost);
    }
}
