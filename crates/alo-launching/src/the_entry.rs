//! What one desktop entry says about **starting** it.
//!
//! `alo-installed` reads the keys that decide whether an application is
//! *listed* — `Type`, `Name`, `NoDisplay`, `Hidden`, `MimeType`, `Icon` — and
//! deliberately not `Exec`. This reads the keys that decide how it is
//! *started*, from the same `[Desktop Entry]` group by the same rules: only
//! that group, unlocalised keys, the first group ends at the next header.
//!
//! **Read at the moment of the press, not at sign-in.** An application
//! installed after the Dock was drawn is started from the entry that is there
//! now, and an entry changed by an update is started as it now says.

/// The keys of one entry that decide how it starts.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TheEntry {
    /// `Type`, where it said one.
    pub kind: Option<String>,
    /// `Name`, the name a person is shown — what `%c` becomes.
    pub name: Option<String>,
    /// `Exec`, exactly as written, escapes and all.
    pub exec: Option<String>,
    /// `DBusActivatable=true`.
    pub bus_activatable: bool,
    /// `Terminal=true`: it expects to be run inside a terminal.
    pub terminal: bool,
    /// `Hidden=true`: the specification's *deleted*.
    pub hidden: bool,
    /// `Path`, the folder it asks to be started in.
    pub folder: Option<String>,
    /// `Icon`, what `%i` hands over.
    pub icon: Option<String>,
}

impl TheEntry {
    /// Read the `[Desktop Entry]` group of this text.
    ///
    /// [`None`] when there is no such group, which is a file that is not a
    /// desktop entry at all.
    #[must_use]
    pub fn read(text: &str) -> Option<Self> {
        let mut entry = Self::default();
        let mut inside = false;
        let mut saw_the_group = false;
        for line in text.lines() {
            let line = line.trim();
            if line.starts_with('[') {
                if inside {
                    break;
                }
                inside = line == "[Desktop Entry]";
                saw_the_group |= inside;
                continue;
            }
            if !inside || line.starts_with('#') {
                continue;
            }
            let Some((key, value)) = line.split_once('=') else {
                continue;
            };
            let value = value.trim();
            let some = || (!value.is_empty()).then(|| value.to_owned());
            match key.trim() {
                "Type" => entry.kind = some(),
                "Name" => entry.name = some(),
                "Exec" => entry.exec = some(),
                "DBusActivatable" => entry.bus_activatable = value == "true",
                "Terminal" => entry.terminal = value == "true",
                "Hidden" => entry.hidden = value == "true",
                "Path" => entry.folder = some(),
                "Icon" => entry.icon = some(),
                _ => {}
            }
        }
        saw_the_group.then_some(entry)
    }
}

#[cfg(test)]
#[path = "the_entry_tests.rs"]
mod tests;
