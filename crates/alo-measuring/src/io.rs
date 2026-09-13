//! `/proc/<pid>/io`: what a process has read and written through its files.
//!
//! `rchar` and `wchar` are the bytes that went through `read` and `write`
//! calls — the process's open files, whatever they are — since it began.
//! `read_bytes` and `write_bytes`, which the same file also has, are the
//! bytes that reached a disk, and are deliberately not what is answered: a
//! person asking what a program is doing with its files means the files, and
//! the kernel's cache is not theirs to reason about.

/// What this crate reads out of `/proc/<pid>/io`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Counted {
    /// `rchar`: bytes read.
    pub read: u64,
    /// `wchar`: bytes written.
    pub written: u64,
}

/// The file, read. [`None`] if either line is missing or not a number.
#[must_use]
pub(crate) fn counted(text: &str) -> Option<Counted> {
    Some(Counted {
        read: number_on_line(text, "rchar")?,
        written: number_on_line(text, "wchar")?,
    })
}

/// The number on the line `name: N`.
fn number_on_line(text: &str, name: &str) -> Option<u64> {
    text.lines().find_map(|line| {
        let (label, rest) = line.split_once(':')?;
        if label.trim() != name {
            return None;
        }
        rest.trim().parse().ok()
    })
}

#[cfg(test)]
mod tests {
    use super::{Counted, counted};

    /// A real file.
    const IO: &str = "rchar: 20067
wchar: 4096
syscr: 25
syscw: 1
read_bytes: 0
write_bytes: 0
cancelled_write_bytes: 0
";

    /// The two lines a person means by *read* and *written*.
    #[test]
    fn the_bytes_through_the_files_are_read() {
        assert_eq!(
            counted(IO),
            Some(Counted {
                read: 20067,
                written: 4096,
            })
        );
    }

    /// Either line missing is no reading, not a zero.
    #[test]
    fn a_file_without_both_lines_is_nothing() {
        assert_eq!(counted("rchar: 1\n"), None);
        assert_eq!(counted("wchar: 1\n"), None);
        assert_eq!(counted("rchar: x\nwchar: 1\n"), None);
        assert_eq!(counted(""), None);
    }
}
