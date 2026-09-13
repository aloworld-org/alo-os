//! `/proc/<pid>/net/dev` and `/proc/<pid>/ns/net`: the network the process
//! is on, and how much has crossed it.
//!
//! The kernel keeps no per-process count of bytes on the network. What it
//! keeps is one count per interface per **network namespace**, and
//! `/proc/<pid>/net/dev` shows the namespace the process is in. So the two
//! numbers here are honest only beside the namespace they belong to, which is
//! why the namespace's number is read as well: `crate::Running` uses it to say
//! how many other processes the count is shared with.
//!
//! Loopback is left out. Bytes that went from this machine to this machine
//! are not what *network* means to the person asking, and they would count
//! twice, once sent and once received.

/// What this crate reads out of `/proc/<pid>/net/dev`: every interface but
/// loopback, summed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) struct Crossed {
    /// Receive bytes, the first number after the interface name.
    pub received: u64,
    /// Transmit bytes, the ninth.
    pub sent: u64,
}

/// The interface that is left out.
const LOOPBACK: &str = "lo";

/// The file, read. [`None`] if it is not the file — no interface line at all
/// is a valid reading of a namespace with nothing in it, and is `Some` of two
/// zeros; a line that is not a number is not.
#[must_use]
pub(crate) fn crossed(text: &str) -> Option<Crossed> {
    let mut total = Crossed::default();
    for line in text.lines() {
        let Some((name, numbers)) = line.split_once(':') else {
            // The two header lines have no colon after a name; the first has
            // `|` separators and the second begins with ` face |`.
            continue;
        };
        if name.trim() == LOOPBACK {
            continue;
        }
        let mut fields = numbers.split_whitespace();
        let received: u64 = fields.next()?.parse().ok()?;
        let sent: u64 = fields.nth(7)?.parse().ok()?;
        total.received = total.received.checked_add(received)?;
        total.sent = total.sent.checked_add(sent)?;
    }
    Some(total)
}

/// The namespace number in what `/proc/<pid>/ns/net` links to, which the
/// kernel writes as `net:[4026531840]`.
#[must_use]
pub(crate) fn namespace(link: &str) -> Option<u64> {
    link.strip_prefix("net:[")?.strip_suffix(']')?.parse().ok()
}

#[cfg(test)]
mod tests {
    use super::{Crossed, crossed, namespace};

    /// A real file, with a loopback interface carrying traffic that must not
    /// be counted.
    const DEV: &str = "Inter-|   Receive                                                |  Transmit
 face |bytes    packets errs drop fifo frame compressed multicast|bytes    packets errs drop fifo colls carrier compressed
    lo: 9999999 100    0    0    0     0          0         0  9999999 100    0    0    0     0       0          0
  eth0: 1000 10    0    0    0     0          0         0  2000 20    0    0    0     0       0          0
 wlan0: 30 3    0    0    0     0          0         0  40 4    0    0    0     0       0          0
";

    /// Every interface but loopback, summed, receive first and transmit ninth.
    #[test]
    fn every_interface_but_loopback_is_summed() {
        assert_eq!(
            crossed(DEV),
            Some(Crossed {
                received: 1030,
                sent: 2040,
            })
        );
    }

    /// The headers alone are a namespace with nothing on it, and that is a
    /// reading of two zeros rather than no reading; a number that is not one
    /// is no reading.
    #[test]
    fn nothing_on_the_network_is_zero_and_nonsense_is_nothing() {
        let headers: String = DEV
            .lines()
            .take(2)
            .map(|line| format!("{line}\n"))
            .collect();
        assert_eq!(crossed(&headers), Some(Crossed::default()));
        assert_eq!(crossed("  eth0: x 1 2 3 4 5 6 7 8\n"), None);
        assert_eq!(crossed("  eth0: 1 2\n"), None);
    }

    /// The namespace is the number in the brackets and nothing else.
    #[test]
    fn the_namespace_is_the_number_in_the_link() {
        assert_eq!(namespace("net:[4026531840]"), Some(4_026_531_840));
        assert_eq!(namespace("pid:[4026531840]"), None);
        assert_eq!(namespace("net:[x]"), None);
        assert_eq!(namespace(""), None);
    }
}
