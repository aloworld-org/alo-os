//! The lines the kernel writes as `Name:   1234 kB`.
//!
//! `/proc/meminfo` and `/proc/<pid>/status` share this shape, and the number
//! is in kilobytes whatever the file. It is turned into bytes here, once, so
//! that every number this crate answers with is in one unit.

/// The number on the line named `name`, in bytes, or [`None`] if the file has
/// no such line.
///
/// A line that is there but does not say a number of kilobytes is treated as
/// not there: the kernel writes these lines itself and does not get them
/// wrong, so anything else is not the line asked for.
#[must_use]
pub(crate) fn bytes_on_line(text: &str, name: &str) -> Option<u64> {
    text.lines().find_map(|line| {
        let (label, rest) = line.split_once(':')?;
        if label.trim() != name {
            return None;
        }
        let mut words = rest.split_whitespace();
        let kilobytes: u64 = words.next()?.parse().ok()?;
        if words.next() != Some("kB") {
            return None;
        }
        kilobytes.checked_mul(1024)
    })
}

#[cfg(test)]
mod tests {
    use super::bytes_on_line;

    /// A slice of a real `/proc/meminfo`.
    const MEMINFO: &str = "MemTotal:       16274708 kB
MemFree:         7908084 kB
MemAvailable:   13116524 kB
Buffers:          243700 kB
";

    /// A slice of a real `/proc/<pid>/status`, tab-separated as the kernel
    /// writes it.
    const STATUS: &str = "Name:\tcat
Umask:\t0022
VmPeak:\t    5876 kB
VmRSS:\t    2408 kB
Threads:\t1
";

    /// The line named is found, and kilobytes become bytes.
    #[test]
    fn the_named_line_is_read_in_bytes() {
        assert_eq!(bytes_on_line(MEMINFO, "MemTotal"), Some(16_274_708 * 1024));
        assert_eq!(
            bytes_on_line(MEMINFO, "MemAvailable"),
            Some(13_116_524 * 1024)
        );
        assert_eq!(bytes_on_line(STATUS, "VmRSS"), Some(2408 * 1024));
    }

    /// A line that is not there is [`None`], and so is a line that is there
    /// without a number of kilobytes on it — `Threads:\t1` is not memory.
    #[test]
    fn a_line_that_is_not_there_is_not_a_zero() {
        assert_eq!(bytes_on_line(STATUS, "VmSwap"), None);
        assert_eq!(bytes_on_line(STATUS, "Threads"), None);
        assert_eq!(bytes_on_line("", "MemTotal"), None);
        // A name that is a prefix of another is not that other.
        assert_eq!(bytes_on_line(MEMINFO, "Mem"), None);
    }
}
