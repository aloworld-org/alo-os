//! The two `stat` files: a process's, and the machine's.
//!
//! `/proc/<pid>/stat` is one line of numbers with the process's name in
//! parentheses as the second field, and the name may hold spaces and
//! parentheses of its own. So the line is read from the **last** closing
//! parenthesis, after which every field is a number and the count is the
//! kernel's documented one.
//!
//! `/proc/stat` begins with a `cpu` line summing every processor: user, nice,
//! system, idle, iowait, irq, softirq, steal, guest and guest_nice, in clock
//! ticks. Their sum is all the processor time there was, which is what a
//! process's ticks are a share of.

/// What this crate reads out of `/proc/<pid>/stat`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ProcessStat {
    /// Field 2, `comm`: what the process calls itself, without the
    /// parentheses the kernel writes round it.
    pub name: String,
    /// Field 14, `utime`: ticks scheduled in user mode.
    pub utime: u64,
    /// Field 15, `stime`: ticks scheduled in kernel mode.
    pub stime: u64,
    /// Field 22, `starttime`: ticks after boot the process started.
    pub starttime: u64,
}

impl ProcessStat {
    /// User and kernel ticks together, which is the process's processor time.
    #[must_use]
    pub(crate) fn ticks(&self) -> u64 {
        self.utime.saturating_add(self.stime)
    }
}

/// A process's `stat` line, read. [`None`] if the line is not one.
#[must_use]
pub(crate) fn process(text: &str) -> Option<ProcessStat> {
    let (_, from_name) = text.split_once('(')?;
    let (name, after_name) = from_name.rsplit_once(')')?;
    // Field 3 is the first after the name; the kernel's numbering is 1-based
    // and the name is field 2, so field N is at index N - 3 here.
    let mut fields = after_name.split_whitespace();
    let utime = fields.nth(14 - 3)?.parse().ok()?;
    let stime = fields.next()?.parse().ok()?;
    let starttime = fields.nth(22 - 15 - 1)?.parse().ok()?;
    Some(ProcessStat {
        name: name.to_owned(),
        utime,
        stime,
        starttime,
    })
}

/// The machine's `stat`, read: the sum of the `cpu` line, in ticks. [`None`]
/// if there is no such line.
#[must_use]
pub(crate) fn machine(text: &str) -> Option<u64> {
    let line = text.lines().find(|line| line.starts_with("cpu "))?;
    let mut total: u64 = 0;
    for field in line.split_whitespace().skip(1) {
        total = total.checked_add(field.parse().ok()?)?;
    }
    Some(total)
}

#[cfg(test)]
#[expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None is the failure being reported"
)]
mod tests {
    use super::{ProcessStat, machine, process};

    /// A real line, from a `cat`.
    const A_LINE: &str = "9006 (cat) R 8987 8987 8987 34819 8987 4194304 514 0 0 0 3 4 0 0 20 0 1 \
                          0 54309 5877760 152 18446744073709551615 1 1 0 0 0 0 0 0 0 0 0 0 17 0 0 \
                          0 0 0 0 0 0 0 0 0 0 0 0";

    /// Fields 14, 15 and 22 are what they are, counted from the kernel's
    /// documented numbering.
    #[test]
    fn a_process_line_is_read_by_field_number() {
        let read = process(A_LINE);
        assert_eq!(
            read,
            Some(ProcessStat {
                name: "cat".to_owned(),
                utime: 3,
                stime: 4,
                starttime: 54309,
            })
        );
        assert_eq!(read.as_ref().map(ProcessStat::ticks), Some(7));
    }

    /// **A name with a space and a parenthesis in it does not shift the
    /// fields.** `(Web Content)` and `(a) b)` are both real.
    #[test]
    fn a_name_with_spaces_and_parentheses_does_not_shift_the_fields() {
        let awkward = A_LINE.replacen("(cat)", "(a (b) c) d)", 1);
        let read = process(&awkward).expect("a stat line");
        assert_eq!(read.name, "a (b) c) d");
        assert_eq!(
            (read.utime, read.stime, read.starttime),
            (3, 4, 54309),
            "the fields after an awkward name are where they were"
        );
    }

    /// A line that is not one is not a reading of zero.
    #[test]
    fn a_line_that_is_not_a_stat_line_is_nothing() {
        assert_eq!(process(""), None);
        assert_eq!(process("9006 (cat) R 1 2"), None);
        assert_eq!(process("9006 (cat) R x y z"), None);
    }

    /// The machine's `cpu` line is summed, and only that line.
    #[test]
    fn the_machine_line_is_the_sum_of_every_state() {
        let text = "cpu  12027 0 24238 179176 682 0 820 0 0 0\ncpu0 1 1 1 1 1 1 1 1 1 1\nintr 5\n";
        assert_eq!(machine(text), Some(12027 + 24238 + 179_176 + 682 + 820));
        assert_eq!(machine("cpu0 1 1 1\n"), None);
        assert_eq!(machine("cpu  1 x\n"), None);
    }
}
