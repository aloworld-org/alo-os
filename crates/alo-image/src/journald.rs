//! What this image tells `systemd-journald` to do, read from its drop-in.
//!
//! One question only: **where a forwarded journal is written.**
//! `crate::kernel_arguments` answers whether forwarding is on, because that
//! is a kernel argument; this answers where it goes, because that is not.
//!
//! # Why the two are separate files
//!
//! They were one, for about an hour, and the thing that separated them is
//! worth keeping. `systemd.journald.tty_path=/dev/ttyS0` was added to
//! `console.toml` as a kernel argument, gated green, built, and booted —
//! and **changed nothing at all**: of 854 lines on `ttyS0`, 854 still
//! carried a kernel timestamp prefix and none came from the journal, the
//! same count as the boot before it.
//!
//! journald accepts exactly ten `systemd.journald.*` kernel arguments, asked
//! of the binary rather than of a manual page, and `tty_path` is not among
//! them. **A kernel argument journald does not know is not an error.** GRUB
//! carries it, the kernel carries it, `/proc/cmdline` shows it, and nothing
//! reads it — so every artefact of the fix looked right while the journal
//! stayed on the screen.
//!
//! So the destination lives where journald reads it, in
//! `usr/lib/systemd/journald.conf.d/`, and this file is what lets the gate
//! tell a configured destination from a plausible one.
//!
//! # Read leniently, like the recipe and the notes
//!
//! No file is an empty answer rather than *this directory is no image*, which
//! is `crate::image`'s convention for everything a check has a sentence
//! about. An image with no drop-in has a fault `crate::checking` names; it is
//! not unreadable.

/// What an image's journald drop-in says, of the one key that matters here.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TheJournalSettings {
    /// The `TTYPath=` the drop-in sets, if it sets one.
    tty_path: Option<String>,
}

impl TheJournalSettings {
    /// Read a journald drop-in.
    ///
    /// **The last assignment wins**, which is systemd's own rule for a key
    /// repeated inside one file, and the reason this does not stop at the
    /// first match.
    ///
    /// Comments are skipped by the `#` the shipped `journald.conf` uses for
    /// its own defaults — `#TTYPath=/dev/console` is a documented default and
    /// not a setting, and a reader that took it for one would report the
    /// screen as configured on every image that ships the stock file.
    #[must_use]
    pub fn read(text: &str) -> Self {
        let tty_path = text
            .lines()
            .map(str::trim)
            .filter(|line| !line.starts_with('#') && !line.starts_with(';'))
            .filter_map(|line| line.split_once('='))
            .filter_map(|(key, value)| {
                key.trim()
                    .eq_ignore_ascii_case("TTYPath")
                    .then(|| value.trim())
                    .filter(|value| !value.is_empty())
                    .map(str::to_owned)
            })
            .next_back();
        Self { tty_path }
    }

    /// The console a forwarded journal is written to, when the drop-in says.
    ///
    /// [`None`] means it does not say, and journald then uses its own default
    /// of `/dev/console` — **the primary console**, which
    /// `usr/lib/bootc/kargs.d/console.toml` deliberately makes `tty0`, the
    /// screen. So [`None`] is not *unknown*; it is *the screen*, and the
    /// check reads it that way.
    #[must_use]
    pub fn where_a_forwarded_journal_goes(&self) -> Option<&str> {
        self.tty_path.as_deref()
    }

    /// Whether that destination is one of these serial lines.
    ///
    /// Takes the lines rather than finding them, because which ports this
    /// image has is `crate::kernel_arguments`' answer and there should not be
    /// a second one. A `/dev/` prefix is accepted on either side: journald
    /// wants a path and a `console=` names a port.
    #[must_use]
    pub fn goes_to_one_of(&self, lines: &[&str]) -> bool {
        self.where_a_forwarded_journal_goes().is_some_and(|path| {
            let named = path.strip_prefix("/dev/").unwrap_or(path);
            lines.contains(&named)
        })
    }
}

#[cfg(test)]
#[path = "journald_tests.rs"]
mod tests;
