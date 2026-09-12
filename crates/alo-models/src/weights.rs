//! Weights somebody brought themselves — a model alo OS never catalogued,
//! already on the machine, pointed at rather than fetched.
//!
//! `docs/features.md` at v0.5: *★ **Run a model we never catalogued.** Point
//! alo OS at weights you already have and it runs them. **The catalogue
//! recommends; it does not gate** — its job is stating licences and honest
//! costs so somebody can choose well, never deciding what they may run on
//! hardware they own. A machine where the only models are the ones we approved
//! is a walled garden with a sovereign label on it.*
//!
//! [`crate::Model`] is the other kind, and the two are not the same type on
//! purpose. A catalogue entry answers three questions — where the weights come
//! from, what they cost, and whether they drive the verbs — and every one of
//! those answers is **ours**, because we are the ones offering it. A set of
//! weights somebody already has answers to nobody here, and the differences are
//! what this file is for.
//!
//! # There is no licence field, and the absence is the design
//!
//! `docs/features.md`: *what you bring is yours, including its licence. We
//! state the licence of everything we offer and gate our own catalogue on it.
//! Weights somebody brings themselves come with their own terms and their own
//! responsibility, and **alo OS does not pretend to have checked them**.*
//!
//! A `Licence` here — even one saying *unknown* — would be a field somebody
//! downstream reads as an answer, and a machine that showed *licence: unknown*
//! beside a model would be implying it went and looked. So there is nothing to
//! read: this type cannot express a licence, cannot be filtered by one, and has
//! no `safe_default_for_business` for a settings panel to call. What it has
//! instead is [`words::LICENCE_IS_YOURS`], said to the person at the moment
//! they point alo OS at their own weights, and [`Weights::lines`] is the only
//! road to a sentence about the cost — so that line cannot be shown without it.
//!
//! # Three answers still, and only one of them gates
//!
//! **Where it is** is the id the model runtime on this machine answers to,
//! and — since [`Weights::at`] — the file on this disk where a person pointed,
//! where they pointed at one. There is no `upstream`, because nothing fetches
//! these: the catalogue's licence gate lives on `ModelRuntime::fetch` and
//! nothing here goes near it. `ModelRuntime::answers` has never been gated, and
//! says why — *a model already on somebody's own disk was either fetched
//! through that gate or put there by the person whose machine it is*.
//!
//! # A file is measured, not described
//!
//! [`Weights::at`] is the road for *point alo OS at weights you already have*.
//! What it writes down about the file is what the disk says — its size, read
//! off the file at that moment — and nothing else. Nothing here opens the file,
//! parses a header, guesses a quantisation or a parameter count from a name, or
//! looks the name up in the catalogue: the catalogue's grades and licences are
//! for the entries the catalogue makes, and a file somebody brought has neither
//! until somebody measures it. The id is the file's own name, exactly, because
//! a name alo OS invented would be a name nothing else on the machine answers
//! to. The three ways a path fails to be a file are refused in words naming
//! the path, and nothing is added in any of them.
//!
//! **What it costs** is [`crate::Cost`], which warns and refuses nothing.
//!
//! **Whether it drives the verbs** is [`Driving`], the same grade the catalogue
//! carries, from the same measurement — `alo-driving` puts its ten requests to
//! any model, so somebody can measure their own and write down what it earned.
//! [`Driving::NotMeasured`] until they do, and that bar is kept rather than
//! waived: it is not a judgement about what they may run, it is whether alo OS
//! will hand an agent turn to it, and a model that cannot emit a verb call is
//! useless as an agent on anybody's hardware.

use std::path::{Path, PathBuf};

use alo_strings::{Filling, Said, Strings};
use serde::{Deserialize, Serialize};

use crate::costing::Cost;
use crate::driving::Driving;
use crate::runtime::Installed;
use crate::words;

/// Why weights could not be taken as brought.
///
/// **No `Display`, and therefore not a `std::error::Error`** (item 9f). Both of
/// these are read by somebody who has just pointed alo OS at a model of their
/// own, so the only road to words is [`WeightsError::said`] and it takes the
/// strings that person reads.
///
/// None of them is about size. That is the whole shape of this subject:
/// what a model costs is [`crate::Cost`], it is a value rather than an error,
/// and it does not appear in this enum because there is no way for it to stop
/// anything. The three about a file are about a path that is not a file at
/// all — there is nothing there, it is a folder, or the disk would not say —
/// and each names the path, because the path is what a person acts on.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WeightsError {
    /// Nothing to ask the runtime for.
    Unnamed,
    /// The same weights are already on this machine's list, so *answered by X*
    /// could not say which — `Providers::add`'s reasoning, one list over.
    AlreadyBrought(String),
    /// Nothing is at the path a person pointed at.
    NoFileThere(PathBuf),
    /// Something is there and it is not a file — a folder, most often.
    NotAFile(PathBuf),
    /// The disk would not say what is at the path: a permission, a device
    /// that is not answering. `why` is the machine's own account, kept for
    /// whoever is fixing the machine and never put in the sentence.
    FileNotRead {
        /// The path a person pointed at.
        at: PathBuf,
        /// What the machine said about it.
        why: String,
    },
}

impl WeightsError {
    /// The string this crate declares for this refusal.
    #[must_use]
    pub fn word(&self) -> words::Word {
        match self {
            Self::Unnamed => words::WEIGHTS_UNNAMED,
            Self::AlreadyBrought(_) => words::WEIGHTS_ALREADY_BROUGHT,
            Self::NoFileThere(_) => words::WEIGHTS_NO_FILE_THERE,
            Self::NotAFile(_) => words::WEIGHTS_NOT_A_FILE,
            Self::FileNotRead { .. } => words::WEIGHTS_FILE_NOT_READ,
        }
    }

    /// What this says, in the language the person reads.
    ///
    /// Never fails and never panics: a `Strings` that was never given
    /// [`crate::model_words`] answers with the key, marked.
    #[must_use]
    pub fn said(&self, strings: &Strings) -> Said {
        let filling = match self {
            Self::AlreadyBrought(id) => Filling::of("name", id.clone()),
            // A path is data and is never translated: it is quoted back as
            // it is on the disk, because it is the thing the person acts on.
            Self::NoFileThere(at) | Self::NotAFile(at) | Self::FileNotRead { at, .. } => {
                Filling::of("path", at.to_string_lossy().into_owned())
            }
            Self::Unnamed => Filling::nothing(),
        };
        strings.say(&self.word().key(), &filling)
    }
}

/// One set of weights somebody brought to this machine.
///
/// `Serialize` for the reason [`crate::Provider`] is: this is what a settings
/// file holds. Where that file is and who writes it is the daemon's, and is not
/// decided yet — the same sentence item 1 wrote about `Grants`.
///
/// [`drives_verbs`](Self::drives_verbs) has no serde default, which is the
/// catalogue's rule applied here: an entry that says nothing about the
/// measurement fails to read rather than reading as *probably fine*.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Weights {
    /// What the model runtime on this machine answers to. **Matched exactly**,
    /// as `alo-capability` matches every other identity: this is a name a
    /// runtime knows rather than a word a person chose, and two ids differing
    /// in case are two models to it.
    pub id: String,
    /// What the weights take on this machine's disk, as the runtime reports it.
    /// The floor under what they will need in memory — see [`crate::Cost`].
    pub bytes_on_disk: u64,
    /// The quantisation the runtime reports, where it says. Kept for the
    /// catalogue's own reason: *"it worked for me" is not a useful bug report
    /// without it*.
    #[serde(default)]
    pub quantisation: Option<String>,
    /// What a measurement of these weights earned, and
    /// [`Driving::NotMeasured`] until somebody runs one.
    pub drives_verbs: Driving,
    /// The file on this machine a person pointed at, where they pointed at
    /// one rather than picking from what a runtime reports.
    ///
    /// [`None`] for weights a runtime already knows by their id, which is
    /// every entry made by [`Weights::found`] and [`Weights::checked`]. Absent
    /// in a file written before this field existed, and read as absent.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub file: Option<PathBuf>,
}

impl Weights {
    /// Weights named the way the runtime knows them.
    ///
    /// # Errors
    /// [`WeightsError::Unnamed`] for a name that is blank — there would be
    /// nothing to ask the runtime for. Size is deliberately not checked: a
    /// refusal about size is the thing this whole subject exists to not do.
    pub fn checked(id: &str, bytes_on_disk: u64) -> Result<Self, WeightsError> {
        let id = id.trim();
        if id.is_empty() {
            return Err(WeightsError::Unnamed);
        }
        Ok(Self {
            id: id.to_owned(),
            bytes_on_disk,
            quantisation: None,
            drives_verbs: Driving::NotMeasured,
            file: None,
        })
    }

    /// A weights file on this machine, taken as brought — *point alo OS at
    /// weights you already have*.
    ///
    /// **The size is measured off the file, never stated.** It is what the
    /// disk reports for the file at this moment, which is the floor
    /// [`crate::Cost`] compares against; nobody is asked to type a number and
    /// no number is guessed from the name. Nothing else is read: not a header,
    /// not a quantisation, not a parameter count, and not the catalogue. The
    /// id is the file's own name, exactly as the disk spells it, and
    /// [`drives_verbs`](Self::drives_verbs) is [`Driving::NotMeasured`] because
    /// nobody has measured it — [`Weights::lines`] says so in words.
    ///
    /// Nothing here downloads anything, and nothing here decides whether the
    /// file is a model: that is the runtime's to find out when it is asked,
    /// and a file that turns out not to be one is a question that fails to be
    /// answered, said once where it happened.
    ///
    /// # Errors
    /// [`WeightsError::NoFileThere`] when nothing is at the path,
    /// [`WeightsError::NotAFile`] when what is there is not a file, and
    /// [`WeightsError::FileNotRead`] when the disk would not say — each naming
    /// the path. [`WeightsError::Unnamed`] for a file whose name cannot be
    /// spelled as text, because the runtime could not be asked for it.
    pub fn at(path: &Path) -> Result<Self, WeightsError> {
        let measured = match std::fs::metadata(path) {
            Ok(measured) => measured,
            Err(why) if why.kind() == std::io::ErrorKind::NotFound => {
                return Err(WeightsError::NoFileThere(path.to_owned()));
            }
            Err(why) => {
                return Err(WeightsError::FileNotRead {
                    at: path.to_owned(),
                    why: why.to_string(),
                });
            }
        };
        if !measured.is_file() {
            return Err(WeightsError::NotAFile(path.to_owned()));
        }
        let name = path
            .file_name()
            .and_then(|name| name.to_str())
            .ok_or(WeightsError::Unnamed)?;
        let mut weights = Self::checked(name, measured.len())?;
        weights.file = Some(path.to_owned());
        Ok(weights)
    }

    /// Weights the runtime already reports, taken as brought.
    ///
    /// The ordinary road: a person picks from what is on their machine rather
    /// than typing a name, so the id, the size and the quantisation are the
    /// runtime's own answers instead of somebody's recollection.
    ///
    /// # Errors
    /// [`WeightsError::Unnamed`] if the runtime reported something with no name
    /// at all, which nothing could then be asked of.
    pub fn found(installed: &Installed) -> Result<Self, WeightsError> {
        let mut weights = Self::checked(&installed.id, installed.bytes_on_disk)?;
        weights.quantisation = installed.quantisation.clone();
        Ok(weights)
    }

    /// The same weights, with what a measurement of them earned written down.
    ///
    /// Takes `self` rather than `&mut self` because a grade is a fact about one
    /// run against one set of weights: a method that could be called twice on a
    /// borrowed value would invite two grades from two runs to be written over
    /// each other with nothing saying which survived.
    #[must_use]
    pub fn measured(mut self, grade: Driving) -> Self {
        self.drives_verbs = grade;
        self
    }

    /// Whether alo OS will give these weights an agent turn.
    ///
    /// **One question, where [`crate::Model::can_be_the_agent`] is the third of
    /// three.** The catalogue asks whether a model runs here and whether its
    /// licence lets an organisation rely on it, and neither is ours to ask
    /// about weights somebody already has: the memory question warns rather
    /// than refuses, and the licence question has no field to read.
    ///
    /// What is kept is the measurement, because it is not a permission. A model
    /// that produces sentences and loses structure is a bad agent on the
    /// machine of whoever owns it, and handing it somebody's files would be
    /// alo OS being useless rather than alo OS being sovereign.
    #[must_use]
    pub fn can_be_the_agent(&self) -> bool {
        self.drives_verbs.clears_the_bar()
    }

    /// What these weights cost on a machine with this much memory.
    #[must_use]
    pub fn costs_on(&self, machine_gb: f32) -> Cost {
        Cost::of(self.bytes_on_disk, machine_gb)
    }

    /// **What a person is shown when they point alo OS at their own weights:
    /// what it will cost here, whose licence terms these are, and whether
    /// anybody has measured them.**
    ///
    /// Three lines, and there is no method that gives you only the first — the
    /// shape [`crate::NoAgentHere::lines`] has, for the same kind of reason. A
    /// panel that showed the cost alone would be alo OS appearing to have
    /// assessed a model it never looked at; the licence line is the one
    /// sentence saying it did not, and the measurement line is the one saying
    /// the catalogue's grade was not guessed for a file either.
    ///
    /// Said **once**, where the weights are added, which is what
    /// `docs/features.md`'s *said so plainly, once* asks for: nothing on the
    /// way to an answer asks this again, because by then the person has decided
    /// and a machine that repeated itself at every question would be arguing
    /// with them about their own hardware. `alo-telling` is what holds *once*
    /// for the size across a session.
    #[must_use]
    pub fn lines(&self, strings: &Strings, machine_gb: f32) -> [Said; 3] {
        [
            self.costs_on(machine_gb).said(strings),
            strings.say(&words::LICENCE_IS_YOURS.key(), &Filling::nothing()),
            self.measurement(strings),
        ]
    }

    /// Whether anybody has measured these weights driving the verbs, in
    /// words.
    ///
    /// *Not measured* is said as not measured — never as a grade, never as
    /// *probably fine*, and never as a reason to pick something else. The
    /// grade itself, where there is one, is a value beside the sentence rather
    /// than inside it, which is this crate's rule about numbers applied to a
    /// word a language may have no one word for.
    #[must_use]
    pub fn measurement(&self, strings: &Strings) -> Said {
        let word = if self.drives_verbs.has_been_measured() {
            words::WEIGHTS_MEASURED
        } else {
            words::WEIGHTS_NOT_MEASURED
        };
        strings.say(&word.key(), &Filling::nothing())
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::costing::GIGABYTE;
    use crate::testing::{in_english, translated};

    fn installed(id: &str, bytes: u64) -> Installed {
        Installed {
            id: id.to_owned(),
            bytes_on_disk: bytes,
            quantisation: Some("Q4_K_M".to_owned()),
        }
    }

    /// **The item's own sentence, as a test.** Weights far larger than the
    /// machine are brought, and what happens is that somebody is told — a door
    /// that returned `Err` here would be alo OS refusing to try on hardware
    /// its owner paid for.
    #[test]
    fn weights_larger_than_this_machine_are_still_brought() {
        let huge = Weights::found(&installed("their-own-70b", 40 * GIGABYTE)).unwrap();
        assert_eq!(huge.bytes_on_disk, 40 * GIGABYTE);

        let cost = huge.costs_on(16.0);
        assert!(cost.larger_than_memory());
        assert!(
            cost.said(&in_english())
                .text()
                .contains("will still run them"),
            "{}",
            cost.said(&in_english())
        );
    }

    /// **Nothing here answers a licence question, and nothing can.** The
    /// destructure names every field this type has, so whoever adds a
    /// `licence` is stopped by the compiler and asked to read this file's
    /// header rather than told the answer.
    #[test]
    fn nothing_about_these_weights_claims_a_licence_was_read() {
        let weights = Weights::found(&installed("their-own", GIGABYTE)).unwrap();
        let Weights {
            id,
            bytes_on_disk,
            quantisation,
            drives_verbs,
            file,
        } = weights.clone();
        assert_eq!(id, "their-own");
        assert_eq!(bytes_on_disk, GIGABYTE);
        assert_eq!(quantisation, Some("Q4_K_M".to_owned()));
        assert_eq!(drives_verbs, Driving::NotMeasured);
        assert_eq!(file, None);

        // And no rendering of it says anything about a licence either.
        let rendered = format!("{weights:?}") + &serde_json::to_string(&weights).unwrap();
        assert!(!rendered.to_lowercase().contains("licen"), "{rendered}");
    }

    /// The one thing that does gate, and the two that do not. A measurement is
    /// about whether an agent works; a licence and a memory figure are about
    /// what somebody is allowed to do with their own machine.
    #[test]
    fn the_measurement_is_the_only_thing_that_decides_the_agent() {
        let unmeasured = Weights::checked("theirs", 400 * GIGABYTE).unwrap();
        assert!(!unmeasured.can_be_the_agent());

        // Measured and enormous: the memory warning does not take the agent
        // away, because it is a warning.
        let measured = unmeasured.clone().measured(Driving::Reliably);
        assert!(measured.can_be_the_agent());
        assert!(measured.costs_on(16.0).larger_than_memory());

        for grade in [Driving::Sometimes, Driving::Rarely, Driving::NotMeasured] {
            assert!(
                !unmeasured.clone().measured(grade).can_be_the_agent(),
                "{grade:?}"
            );
        }
    }

    /// **The cost cannot be shown without whose licence it is, nor without
    /// whether anybody measured it.** Three lines, one call, and no method
    /// that hands over the first alone.
    #[test]
    fn what_a_person_is_shown_is_the_cost_whose_licence_and_whether_it_was_measured() {
        let strings = in_english();
        let weights = Weights::checked("theirs", 4 * GIGABYTE).unwrap();
        let [cost, licence, measured] = weights.lines(&strings, 16.0);
        assert_eq!(cost.text(), weights.costs_on(16.0).said(&strings).text());
        assert!(licence.text().contains("yours"), "{licence}");
        assert!(licence.text().contains("has not read"), "{licence}");
        assert!(
            measured.text().contains("nobody has measured"),
            "{measured}"
        );

        // The same second and third lines under the other answer, so it is
        // never the warning that carries them.
        let [_, also, and] = Weights::checked("theirs", 40 * GIGABYTE)
            .unwrap()
            .lines(&strings, 16.0);
        assert_eq!(also.text(), licence.text());
        assert_eq!(and.text(), measured.text());
    }

    /// **Not measured is said as not measured**, and a measurement that was
    /// run is said as one — neither as a grade inside the sentence, and
    /// neither as a reason to choose differently.
    #[test]
    fn the_measurement_line_says_whether_a_measurement_was_run_and_no_more() {
        let strings = in_english();
        let unmeasured = Weights::checked("theirs", GIGABYTE).unwrap();
        let said = unmeasured.measurement(&strings);
        assert!(said.text().contains("nobody has measured"), "{said}");
        assert!(!said.text().contains("catalogue"), "{said}");

        for grade in [Driving::Reliably, Driving::Sometimes, Driving::Rarely] {
            let measured = unmeasured.clone().measured(grade).measurement(&strings);
            assert!(
                measured.text().contains("have been measured"),
                "{grade:?}: {measured}"
            );
            assert!(!measured.text().contains("nobody"), "{grade:?}: {measured}");
            assert!(
                !measured.text().to_lowercase().contains("reliab"),
                "{grade:?}: the grade is beside the sentence, not inside it: {measured}"
            );
        }
    }

    /// Both lines are read in the reader's own language, and a machine that
    /// translated one and not the other says so rather than looking finished.
    #[test]
    fn what_a_person_is_shown_is_in_the_language_they_read() {
        let strings = translated(&[(
            words::LICENCE_IS_YOURS,
            "diese Gewichte gehören Ihnen, und ihre Lizenzbedingungen auch",
        )]);
        let [cost, licence, measured] = Weights::checked("theirs", 4 * GIGABYTE)
            .unwrap()
            .lines(&strings, 16.0);
        assert!(licence.is_translated());
        assert!(licence.text().contains("gehören Ihnen"), "{licence}");
        assert!(!cost.is_translated());
        assert!(!measured.is_translated());
    }

    /// A folder of this test's own, empty, under the system's temporary
    /// directory.
    fn a_folder(what: &str) -> PathBuf {
        let folder = std::env::temp_dir()
            .join("alo-models-weights-at")
            .join(format!("{what}-{}", std::process::id()));
        drop(std::fs::remove_dir_all(&folder));
        std::fs::create_dir_all(&folder).unwrap();
        folder
    }

    /// **A file on this machine is taken as brought, and its size is what the
    /// disk says.** The id is the file's own name, nothing is guessed about
    /// it, and the file is remembered so the runtime can be pointed at it.
    #[test]
    fn a_file_on_this_machine_is_brought_and_measured_off_the_disk() {
        let folder = a_folder("measured");
        let at = folder.join("my-finetune.Q4_K_M.gguf");
        std::fs::write(&at, vec![0u8; 12_345]).unwrap();

        let brought = Weights::at(&at).unwrap();

        assert_eq!(brought.id, "my-finetune.Q4_K_M.gguf");
        assert_eq!(brought.bytes_on_disk, 12_345);
        assert_eq!(brought.file.as_deref(), Some(at.as_path()));
        // Nothing was read off the name: no quantisation, no grade.
        assert_eq!(brought.quantisation, None);
        assert_eq!(brought.drives_verbs, Driving::NotMeasured);
        assert!(!brought.can_be_the_agent());
    }

    /// **A path with nothing at it is refused, naming the path**, and so is a
    /// folder — nothing is brought in either, and the sentence is one a person
    /// can act on.
    #[test]
    fn a_path_that_is_not_a_file_is_refused_naming_the_path() {
        let folder = a_folder("refused");
        let strings = in_english();

        let nowhere = folder.join("not-here.gguf");
        let refused = Weights::at(&nowhere).unwrap_err();
        assert_eq!(refused, WeightsError::NoFileThere(nowhere.clone()));
        let said = refused.said(&strings);
        assert!(!said.is_a_bug(), "{said}");
        assert!(said.text().contains("not-here.gguf"), "{said}");
        assert!(said.text().contains("nothing has been added"), "{said}");

        let refused = Weights::at(&folder).unwrap_err();
        assert_eq!(refused, WeightsError::NotAFile(folder.clone()));
        let said = refused.said(&strings);
        assert!(!said.is_a_bug(), "{said}");
        assert!(said.text().contains("nothing has been added"), "{said}");
    }

    /// **A disk that will not say is refused as that**, with what the machine
    /// said kept beside the refusal for whoever fixes it and out of the
    /// sentence the person reads.
    #[test]
    fn a_file_the_disk_will_not_speak_about_is_refused_and_the_reason_is_kept_aside() {
        let refused = WeightsError::FileNotRead {
            at: PathBuf::from("/mnt/weights/theirs.gguf"),
            why: "Permission denied (os error 13)".to_owned(),
        };
        let said = refused.said(&in_english());
        assert!(!said.is_a_bug(), "{said}");
        assert!(said.text().contains("/mnt/weights/theirs.gguf"), "{said}");
        assert!(!said.text().contains("os error"), "{said}");
    }

    /// **The file is written down and read back**, and an entry from before
    /// the field existed reads as weights with no file — which is what it is.
    #[test]
    fn where_the_file_is_survives_being_written_down_and_an_older_entry_has_none() {
        let folder = a_folder("written");
        let at = folder.join("theirs.gguf");
        std::fs::write(&at, b"weights").unwrap();
        let brought = Weights::at(&at).unwrap();
        let written = serde_json::to_string(&brought).unwrap();
        assert_eq!(serde_json::from_str::<Weights>(&written).unwrap(), brought);

        let older = r#"{"id":"theirs","bytes_on_disk":1,"drives_verbs":"not-measured"}"#;
        let read: Weights = serde_json::from_str(older).unwrap();
        assert_eq!(read.file, None);
        // And weights with no file write no `file` key at all.
        let written = serde_json::to_string(&read).unwrap();
        assert!(!written.contains("file"), "{written}");
    }

    /// Weights with no name could not be asked of anything, so that is the one
    /// door refusal — and it says what to do about it.
    #[test]
    fn weights_with_no_name_cannot_be_asked_for() {
        let refused = Weights::checked("   ", GIGABYTE).unwrap_err();
        assert_eq!(refused, WeightsError::Unnamed);
        assert!(
            refused.said(&in_english()).text().contains("name"),
            "{refused:?}"
        );
        assert_eq!(
            Weights::found(&installed("", GIGABYTE)).unwrap_err(),
            WeightsError::Unnamed
        );
    }

    /// A name is what the runtime answers to, so surrounding space is not part
    /// of it — and what the runtime reported is what is written down.
    #[test]
    fn what_the_runtime_reported_is_what_was_brought() {
        assert_eq!(Weights::checked("  theirs  ", 1).unwrap().id, "theirs");

        let found = Weights::found(&installed("theirs:7b", 3 * GIGABYTE)).unwrap();
        assert_eq!(found.id, "theirs:7b");
        assert_eq!(found.bytes_on_disk, 3 * GIGABYTE);
        assert_eq!(found.quantisation, Some("Q4_K_M".to_owned()));
    }

    /// **A stored entry states its grade or fails to read**, which is the
    /// catalogue's rule (`drives_verbs` has no serde default) applied to the
    /// list a person owns. An entry reading as *probably fine* is exactly what
    /// `Driving::NotMeasured` exists to prevent.
    #[test]
    fn a_stored_entry_that_says_nothing_about_the_measurement_does_not_read() {
        let weights = Weights::checked("theirs", GIGABYTE)
            .unwrap()
            .measured(Driving::Reliably);
        let written = serde_json::to_string(&weights).unwrap();
        assert_eq!(
            serde_json::from_str::<Weights>(&written).unwrap(),
            weights,
            "{written}"
        );

        let silent = r#"{"id":"theirs","bytes_on_disk":1}"#;
        assert!(serde_json::from_str::<Weights>(silent).is_err());
        let invented = r#"{"id":"theirs","bytes_on_disk":1,"drives_verbs":"probably"}"#;
        assert!(serde_json::from_str::<Weights>(invented).is_err());
    }
}
