//! What a built `alo-installer.exe` asks Windows to load before it starts, and
//! which of those a machine that has never built anything does not have.
//!
//! # The failure this exists for
//!
//! On 2026-10-05 the testing PC ran the published `alo-installer.exe` on a clean
//! Windows 11 Pro, elevated, and the process died in the Windows loader before
//! `main`, printing nothing:
//!
//! ```text
//! exit -1073741515  (0xC0000135, STATUS_DLL_NOT_FOUND)
//! imports: KERNEL32, ntdll, api-ms-win-core-synch-l1-2-0,
//!          VCRUNTIME140.dll, api-ms-win-crt-{runtime,math,stdio,locale,heap}
//! System32: VCRUNTIME140.dll, vcruntime140_1.dll, msvcp140.dll all absent
//! ```
//!
//! The default MSVC toolchain links the Visual C++ runtime as a **separate
//! DLL**, and the zip ships no DLL beside the executable. **Promise 22 —
//! *download, click, reboot* — failed at *click***, not at a check and not at a
//! refusal anybody designed. The program did not start.
//!
//! It had never been seen because the installer had only ever been run on
//! machines that build Rust, and a machine with a toolchain has that runtime as
//! a side effect of having one.
//!
//! # Why a reader here rather than running the program somewhere
//!
//! **A GitHub `windows-2025` runner is not a clean machine.** It builds Rust, so
//! it has the Visual C++ redistributable, and a workflow step that started the
//! executable there would have passed on 2026-10-05 while the bug was live. That
//! is the whole family of fault this repository keeps finding: a check whose
//! inputs are less specific than its question.
//!
//! What a clean machine lacks is knowable **from the file**, without a machine
//! to lack it on: the names in a program's import table are the libraries
//! Windows must find before the first instruction runs. So this reads them.
//!
//! # What it deliberately does not answer
//!
//! That the program then *works*. An executable importing nothing unusual can
//! still refuse, crash or do the wrong thing, and this says nothing about any of
//! it. It answers one question — **will Windows load it on a machine with no
//! toolchain** — and that question had no check at all.
//!
//! The api-set names (`api-ms-win-*`) are not libraries: Windows resolves them
//! internally, and their absence from `System32` is normal. They are read like
//! any other name and are not matched by [`A_REDISTRIBUTABLE_LIBRARY`].

use std::fmt;

/// How a Visual C++ redistributable library is recognised, by the start of its
/// name and without case.
///
/// **A rule rather than a list, and that is a correction.** This was three exact
/// names — `VCRUNTIME140.dll`, `vcruntime140_1.dll`, `msvcp140.dll` — under a
/// comment claiming all three were measured. **Only the first was.** The testing
/// PC checked four names in `System32` because they ship together, found all
/// four absent, and only `VCRUNTIME140.dll` was in the executable's import
/// table and killed the loader. The other two were a precaution wearing a
/// measurement's clothes.
///
/// So the rule says what is actually true: **this executable must import no
/// library from the Visual C++ redistributable.** `VCRUNTIME140` is the one that
/// broke a machine; `MSVCP` is the C++ standard library a C++ dependency would
/// pull in; `CONCRT` and `VCCORLIB` are the rest of the same package. A
/// trailing-number variant like `vcruntime140_1.dll` is caught by the prefix
/// rather than by somebody remembering to add it.
///
/// # What is deliberately not here
///
/// The api-set names, `api-ms-win-*`. The same machine showed those absent from
/// `System32` too, and **that is normal on Windows 10 and later**: the loader
/// resolves them to the Universal CRT in `ucrtbase.dll` rather than to files of
/// those names. It walked past every one of them and stopped at
/// `VCRUNTIME140.dll`. Forbidding them would be stricter than any measurement
/// supports, and would condemn programs that start perfectly well.
pub const A_REDISTRIBUTABLE_LIBRARY: [&str; 4] = ["vcruntime", "msvcp", "concrt", "vccorlib"];

/// The one name that was measured breaking a machine, on 2026-10-05.
///
/// Kept separate from the rule above so that the measured fact and the
/// precaution around it never get confused again, which is how the first
/// version of this file came to claim three measurements and have one.
pub const THE_ONE_THAT_BROKE_A_MACHINE: &str = "VCRUNTIME140.dll";

/// Why a file could not be read as a Windows program.
///
/// Every arm names where the read stopped, because a truncated file and a file
/// that is not a program at all are different problems and the fix differs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NotAProgram {
    /// No `MZ` at the start: not a DOS or Windows executable.
    NoDosHeader,
    /// The DOS header's pointer does not land on a `PE` signature.
    NoPeSignature,
    /// A header or table runs past the end of the file.
    Truncated {
        /// Which structure ran out, for somebody reading the failure.
        reading: &'static str,
    },
    /// The optional header's magic is neither PE32 nor PE32+.
    UnknownFormat {
        /// What was there instead.
        magic: u16,
    },
    /// No import directory, which a program that calls into Windows must have.
    NoImportDirectory,
    /// An import entry's name is not inside any section's bytes.
    NameOutsideTheFile {
        /// The address that could not be resolved.
        at: u32,
    },
}

impl fmt::Display for NotAProgram {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NoDosHeader => write!(
                f,
                "this file does not begin with MZ, so it is not a Windows program"
            ),
            Self::NoPeSignature => write!(f, "the DOS header does not point at a PE signature"),
            Self::Truncated { reading } => {
                write!(f, "the file ends in the middle of its {reading}")
            }
            Self::UnknownFormat { magic } => write!(
                f,
                "the optional header's magic is {magic:#06x}, which is neither PE32 nor PE32+"
            ),
            Self::NoImportDirectory => write!(
                f,
                "this program has no import directory, so it asks Windows for no library at all"
            ),
            Self::NameOutsideTheFile { at } => write!(
                f,
                "an import names a library at address {at:#x}, which is in no section of this file"
            ),
        }
    }
}

impl std::error::Error for NotAProgram {}

/// Where a section's bytes are, and which addresses they answer for.
#[derive(Debug, Clone, Copy)]
struct Section {
    /// The address the loader maps these bytes at.
    at: u32,
    /// How much address space the section covers once loaded.
    covers: u32,
    /// Where the bytes are in the file.
    from: u32,
    /// How many bytes are actually in the file.
    holds: u32,
}

/// Every library a program asks Windows to load before it runs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TheImports {
    /// The names, in the order the import table lists them.
    named: Vec<String>,
}

/// Four bytes, little-endian, or a truncation naming what ran out.
fn u32_at(bytes: &[u8], at: usize, reading: &'static str) -> Result<u32, NotAProgram> {
    bytes
        .get(at..at + 4)
        .and_then(|four| <[u8; 4]>::try_from(four).ok())
        .map(u32::from_le_bytes)
        .ok_or(NotAProgram::Truncated { reading })
}

/// Two bytes, little-endian, or a truncation naming what ran out.
fn u16_at(bytes: &[u8], at: usize, reading: &'static str) -> Result<u16, NotAProgram> {
    bytes
        .get(at..at + 2)
        .and_then(|two| <[u8; 2]>::try_from(two).ok())
        .map(u16::from_le_bytes)
        .ok_or(NotAProgram::Truncated { reading })
}

impl TheImports {
    /// Read a Windows program's import table out of its bytes.
    ///
    /// # Errors
    ///
    /// [`NotAProgram`] where the file is not a Windows executable, ends inside a
    /// structure this has to read, or names a library at an address no section
    /// covers.
    pub fn read(bytes: &[u8]) -> Result<Self, NotAProgram> {
        if bytes.get(..2) != Some(b"MZ") {
            return Err(NotAProgram::NoDosHeader);
        }
        // The DOS header's last field is where the real header begins.
        let pe = u32_at(bytes, 0x3C, "DOS header")? as usize;
        if bytes.get(pe..pe + 4) != Some(b"PE\0\0") {
            return Err(NotAProgram::NoPeSignature);
        }

        // The COFF header, immediately after the signature.
        let coff = pe + 4;
        let sections_named = u16_at(bytes, coff + 2, "COFF header")? as usize;
        let optional_is = u16_at(bytes, coff + 16, "COFF header")? as usize;
        let optional = coff + 20;

        // PE32 and PE32+ differ only in where the data directories start: the
        // 64-bit form carries wider addresses in the fields before them.
        let directories_at = match u16_at(bytes, optional, "optional header")? {
            0x010B => optional + 96,
            0x020B => optional + 112,
            magic => return Err(NotAProgram::UnknownFormat { magic }),
        };

        // Directory one is the imports. Zero means a program that asks for
        // nothing, which is a real answer and not a failure to read.
        let imports_at = u32_at(bytes, directories_at + 8, "data directories")?;
        if imports_at == 0 {
            return Err(NotAProgram::NoImportDirectory);
        }

        let sections = Self::sections(bytes, optional + optional_is, sections_named)?;
        let mut named = Vec::new();
        let mut entry = Self::in_the_file(&sections, imports_at)
            .ok_or(NotAProgram::NameOutsideTheFile { at: imports_at })?;

        // Each descriptor is twenty bytes and the table ends at an all-zero one.
        loop {
            let name_at = u32_at(bytes, entry + 12, "import descriptor")?;
            let first = u32_at(bytes, entry, "import descriptor")?;
            if name_at == 0 && first == 0 {
                break;
            }
            if name_at != 0 {
                let starts = Self::in_the_file(&sections, name_at)
                    .ok_or(NotAProgram::NameOutsideTheFile { at: name_at })?;
                named.push(Self::name_at(bytes, starts)?);
            }
            entry += 20;
        }

        Ok(Self { named })
    }

    /// Every section header, read for its address and its bytes.
    fn sections(bytes: &[u8], at: usize, how_many: usize) -> Result<Vec<Section>, NotAProgram> {
        let mut sections = Vec::with_capacity(how_many);
        for which in 0..how_many {
            let header = at + which * 40;
            sections.push(Section {
                covers: u32_at(bytes, header + 8, "section header")?,
                at: u32_at(bytes, header + 12, "section header")?,
                holds: u32_at(bytes, header + 16, "section header")?,
                from: u32_at(bytes, header + 20, "section header")?,
            });
        }
        Ok(sections)
    }

    /// Where a loaded address lives in the file, if any section holds it.
    ///
    /// **A section can cover more address space than it holds bytes for** —
    /// zero-filled data is counted in `covers` and absent from the file — so an
    /// address inside `covers` but past `holds` is deliberately not answered.
    fn in_the_file(sections: &[Section], address: u32) -> Option<usize> {
        sections.iter().find_map(|section| {
            let past = address.checked_sub(section.at)?;
            (past < section.covers && past < section.holds)
                .then(|| section.from as usize + past as usize)
        })
    }

    /// The NUL-terminated name at this offset.
    fn name_at(bytes: &[u8], at: usize) -> Result<String, NotAProgram> {
        let rest = bytes.get(at..).ok_or(NotAProgram::Truncated {
            reading: "import name",
        })?;
        let ends = rest
            .iter()
            .position(|byte| *byte == 0)
            .ok_or(NotAProgram::Truncated {
                reading: "import name",
            })?;
        // `get` rather than a slice: `position` cannot return an index past the
        // end, so this is a panic the compiler cannot see is impossible — and
        // the one thing this reader must never do is panic on a file somebody
        // handed it.
        let name = rest.get(..ends).ok_or(NotAProgram::Truncated {
            reading: "import name",
        })?;
        Ok(String::from_utf8_lossy(name).into_owned())
    }

    /// Every library this program asks for, in the order its table lists them.
    #[must_use]
    pub fn names(&self) -> &[String] {
        &self.named
    }

    /// Whether this program asks for a library of this name, whatever its case.
    #[must_use]
    pub fn asks_for(&self, library: &str) -> bool {
        self.named
            .iter()
            .any(|named| named.eq_ignore_ascii_case(library))
    }

    /// Every Visual C++ redistributable library this program asks for.
    ///
    /// Empty is the answer that means **this program starts on a clean
    /// Windows**, which is what `tests/the_installer_starts_on_a_clean_windows.rs`
    /// holds the shipped executable to.
    ///
    /// Matched on the start of the name, so `vcruntime140_1.dll` is caught by
    /// the same rule as `VCRUNTIME140.dll` without anybody having to remember
    /// the variant.
    #[must_use]
    pub fn what_a_clean_windows_lacks(&self) -> Vec<&str> {
        self.named
            .iter()
            .filter(|named| {
                let lowered = named.to_ascii_lowercase();
                A_REDISTRIBUTABLE_LIBRARY
                    .iter()
                    .any(|family| lowered.starts_with(family))
            })
            .map(String::as_str)
            .collect()
    }
}

#[cfg(test)]
#[expect(
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;

    /// Where the one section's bytes begin in the file this builds.
    const SECTION_AT: usize = 0x200;
    /// The address the loader maps that section at.
    const SECTION_MAPS_TO: u32 = 0x1000;

    /// A Windows program that imports exactly these libraries.
    ///
    /// **Built rather than committed as a fixture**, because a committed binary
    /// is a thing nobody can read in a review, and because a builder lets a test
    /// ask for the shape it is about — an import table with a name in it, a
    /// truncated one, one whose name points nowhere.
    fn a_program_importing(names: &[&str]) -> Vec<u8> {
        let mut file = vec![0u8; 0x400];
        file[0] = b'M';
        file[1] = b'Z';
        file[0x3C..0x40].copy_from_slice(&0x80u32.to_le_bytes());
        file[0x80..0x84].copy_from_slice(b"PE\0\0");

        // COFF: one section, and an optional header of the 64-bit size.
        file[0x86..0x88].copy_from_slice(&1u16.to_le_bytes());
        file[0x94..0x96].copy_from_slice(&240u16.to_le_bytes());
        // PE32+.
        file[0x98..0x9A].copy_from_slice(&0x020Bu16.to_le_bytes());

        // Descriptors first, then the names they point at.
        let after_descriptors = SECTION_AT + (names.len() + 1) * 20;
        let mut name_at = after_descriptors;
        for (which, name) in names.iter().enumerate() {
            let descriptor = SECTION_AT + which * 20;
            // A non-zero first thunk, so the terminator is the only zero entry.
            file[descriptor..descriptor + 4].copy_from_slice(&1u32.to_le_bytes());
            let rva = SECTION_MAPS_TO + (name_at - SECTION_AT) as u32;
            file[descriptor + 12..descriptor + 16].copy_from_slice(&rva.to_le_bytes());
            file[descriptor + 16..descriptor + 20].copy_from_slice(&1u32.to_le_bytes());
            file[name_at..name_at + name.len()].copy_from_slice(name.as_bytes());
            name_at += name.len() + 1;
        }

        // The one section, covering everything the names reach.
        let header = 0x98 + 240;
        let covers = (name_at - SECTION_AT) as u32;
        file[header..header + 8].copy_from_slice(b".idata\0\0");
        file[header + 8..header + 12].copy_from_slice(&covers.to_le_bytes());
        file[header + 12..header + 16].copy_from_slice(&SECTION_MAPS_TO.to_le_bytes());
        file[header + 16..header + 20].copy_from_slice(&0x200u32.to_le_bytes());
        file[header + 20..header + 24].copy_from_slice(&(SECTION_AT as u32).to_le_bytes());

        // Data directory one: where the descriptors are.
        let directories = 0x98 + 112;
        file[directories + 8..directories + 12].copy_from_slice(&SECTION_MAPS_TO.to_le_bytes());
        file
    }

    /// **The names are read, in the order the table lists them.**
    #[test]
    fn a_programs_imports_are_read_in_order() {
        let program = a_program_importing(&["KERNEL32.dll", "VCRUNTIME140.dll"]);
        let read = TheImports::read(&program).expect("a program this builds is readable");

        assert_eq!(read.names(), ["KERNEL32.dll", "VCRUNTIME140.dll"]);
    }

    /// **The failure this whole module exists for**, stated as the test that
    /// would have caught it: a program importing the Visual C++ runtime names it
    /// as a thing a clean Windows does not have.
    #[test]
    fn the_runtime_a_clean_windows_lacks_is_named() {
        let program = a_program_importing(&["KERNEL32.dll", "VCRUNTIME140.dll"]);
        let read = TheImports::read(&program).expect("a program this builds is readable");

        assert_eq!(read.what_a_clean_windows_lacks(), ["VCRUNTIME140.dll"]);
    }

    /// **A statically linked program asks for nothing a clean Windows lacks**,
    /// which is the passing answer and the one the shipped executable owes.
    #[test]
    fn a_program_that_carries_its_runtime_lacks_nothing() {
        let program = a_program_importing(&["KERNEL32.dll", "ntdll.dll"]);
        let read = TheImports::read(&program).expect("a program this builds is readable");

        assert!(read.what_a_clean_windows_lacks().is_empty());
    }

    /// **The linker's spelling is not the only spelling.** The import table
    /// carries whatever case the linker wrote, and `vcruntime140.dll` is the
    /// same library as `VCRUNTIME140.dll`. A comparison that respected case
    /// would answer *nothing missing* for a program that cannot start.
    #[test]
    fn the_same_library_in_another_case_is_the_same_library() {
        let program = a_program_importing(&["vcruntime140.dll"]);
        let read = TheImports::read(&program).expect("a program this builds is readable");

        assert!(read.asks_for("VCRUNTIME140.dll"));
        // **Reported as the program writes it, not as the rule spells it.** The
        // rule matches without case; what comes back is the name in the import
        // table. The sentence this ends up in is read by somebody looking at an
        // executable, and telling them it imports `VCRUNTIME140.dll` when the
        // table says `vcruntime140.dll` sends them looking for a string that is
        // not there.
        assert_eq!(read.what_a_clean_windows_lacks(), ["vcruntime140.dll"]);
    }

    /// **The api-set names are not libraries and are not reported as missing.**
    /// Windows resolves `api-ms-win-*` internally; the testing PC confirmed they
    /// are absent from `System32` on a working machine, so treating their
    /// absence as a fault would condemn every Windows program there is.
    #[test]
    fn an_api_set_is_read_and_is_not_a_missing_library() {
        let program = a_program_importing(&[
            "api-ms-win-crt-runtime-l1-1-0.dll",
            "api-ms-win-core-synch-l1-2-0.dll",
        ]);
        let read = TheImports::read(&program).expect("a program this builds is readable");

        assert_eq!(read.names().len(), 2);
        assert!(read.what_a_clean_windows_lacks().is_empty());
    }

    /// **A file that is not a program says so**, rather than being read as one
    /// that imports nothing — which would pass this check for every text file.
    #[test]
    fn something_that_is_not_a_program_is_refused() {
        assert_eq!(
            TheImports::read(b"not a program at all"),
            Err(NotAProgram::NoDosHeader)
        );
        let mut nearly = vec![0u8; 0x100];
        nearly[0] = b'M';
        nearly[1] = b'Z';
        nearly[0x3C..0x40].copy_from_slice(&0x80u32.to_le_bytes());
        assert_eq!(TheImports::read(&nearly), Err(NotAProgram::NoPeSignature));
    }

    /// **A file that ends inside a structure says where it ended.** A reader
    /// that returned *no imports* for a truncated file would report a program
    /// that cannot start as one that starts anywhere.
    #[test]
    fn a_file_that_stops_early_names_what_it_stopped_inside() {
        let program = a_program_importing(&["KERNEL32.dll"]);
        let cut = &program[..0x90];

        match TheImports::read(cut) {
            Err(NotAProgram::Truncated { reading }) => {
                assert_eq!(reading, "COFF header");
            }
            other => panic!("a file cut inside its COFF header read as {other:?}"),
        }
    }

    /// **An address no section covers is refused, not guessed at.** A reader
    /// that clamped or skipped it would silently drop a library from the answer,
    /// and a dropped name is exactly the one that would not be there to find.
    #[test]
    fn a_name_outside_every_section_is_refused() {
        let mut program = a_program_importing(&["KERNEL32.dll"]);
        // Point the first descriptor's name far past anything mapped.
        program[SECTION_AT + 12..SECTION_AT + 16].copy_from_slice(&0x9000_0000u32.to_le_bytes());

        assert_eq!(
            TheImports::read(&program),
            Err(NotAProgram::NameOutsideTheFile { at: 0x9000_0000 })
        );
    }

    /// **A program asking for nothing is a readable answer**, and the empty
    /// import directory is told apart from an unreadable file.
    #[test]
    fn a_program_with_no_import_directory_says_so() {
        let mut program = a_program_importing(&["KERNEL32.dll"]);
        let directories = 0x98 + 112;
        program[directories + 8..directories + 12].copy_from_slice(&0u32.to_le_bytes());

        assert_eq!(
            TheImports::read(&program),
            Err(NotAProgram::NoImportDirectory)
        );
    }

    /// **The one name that was measured breaking a machine is caught by the
    /// rule.** Everything else the rule catches is a precaution, and the two are
    /// kept apart on purpose: the first version of this file called three names
    /// *measured* when one was.
    #[test]
    fn the_name_that_broke_a_machine_is_caught_by_the_rule() {
        let program = a_program_importing(&[THE_ONE_THAT_BROKE_A_MACHINE]);
        let read = TheImports::read(&program).expect("a program this builds is readable");

        assert_eq!(
            read.what_a_clean_windows_lacks(),
            [THE_ONE_THAT_BROKE_A_MACHINE]
        );
    }

    /// **A trailing-number variant is caught without anybody adding it.**
    /// `vcruntime140_1.dll` ships with the same redistributable and would fail
    /// the same way; a rule on the start of the name catches it, where a list of
    /// exact spellings catches it only if somebody remembered.
    #[test]
    fn a_variant_nobody_listed_is_still_caught() {
        let program = a_program_importing(&["vcruntime140_1.dll", "msvcp140.dll"]);
        let read = TheImports::read(&program).expect("a program this builds is readable");

        assert_eq!(
            read.what_a_clean_windows_lacks(),
            ["vcruntime140_1.dll", "msvcp140.dll"]
        );
    }

    /// **The rule names families and not versions.** A future toolchain's
    /// `vcruntime150.dll` is the same problem, and a rule written around `140`
    /// would greet it as a library a clean Windows has.
    #[test]
    fn the_rule_is_not_written_around_one_version() {
        let program = a_program_importing(&["VCRUNTIME150.dll"]);
        let read = TheImports::read(&program).expect("a program this builds is readable");

        assert_eq!(read.what_a_clean_windows_lacks(), ["VCRUNTIME150.dll"]);
    }
}
