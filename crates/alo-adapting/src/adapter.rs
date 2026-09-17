//! **What a fine-tune produces: an adapter, tied to the grant that produced it.**
//!
//! [ADR 0048](../../../docs/decisions/0048-an-adapter-is-the-learning-and-the-base-weights-are-never-touched.md).
//! The base model is never written to. Everything a fine-tune learned lives in a
//! file beside it, applied when the model answers, and that file knows which
//! granted folder taught it.
//!
//! **So deleting one adapter takes back what one folder taught, and leaves
//! everything else the model learned.** That is a property of this shape rather
//! than a promise made about it: there is nothing to separate afterwards,
//! because the bytes were never mixed.
//!
//! # One format is the truth, and the other is a copy
//!
//! **The safetensors adapter is canonical.** It is what the trainer wrote, what
//! any other tool reads, and what a person takes to another machine. Everything
//! else is **derived**: the pinned runtime cannot read that format for every
//! architecture, so a copy is converted into the shape that runtime wants, and
//! that copy belongs to that runtime and to this machine.
//!
//! **A derived copy carries the documents too.** It is the same learning in
//! another container, so every rule for the adapter is a rule for it: sending
//! it anywhere is sending the person's documents, and **deleting the adapter
//! deletes every copy made from it**. A machine that kept the runtime's copy
//! after a person deleted their adapter would have taken nothing back at all.
//!
//! If a later change ever keeps only the derived copy, the person's learning is
//! locked to one runtime — which is the thing this product sells against.
//!
//! One adapter per granted folder. v0.5 ships a single one and needs no more —
//! the shape is decided now because it cannot be decided later: once a fine-tune
//! merges into base weights, no amount of care afterwards can take one folder
//! back out.

use std::path::{Path, PathBuf};
use std::time::SystemTime;

use serde::{Deserialize, Serialize};

use crate::learned_from::LearnedFrom;

/// **One adapter: what one grant taught, in a file of its own.**
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Adapter {
    /// **The adapter itself**, in the ordinary LoRA safetensors format: the
    /// truth, and what a person can take elsewhere.
    file: PathBuf,
    /// **Copies made from it for a runtime that cannot read the canonical
    /// format** — derived, regenerable, and carrying the same documents. Each
    /// goes when the adapter goes.
    derived: Vec<PathBuf>,
    /// The base model it applies to, as the catalogue names it.
    applies_to: String,
    /// What the base model was when this was trained — the digest that must
    /// still hold afterwards ([`TheBaseIsUntouched`]).
    the_base_was: String,
    /// The folder, the grant and the counts this was trained from.
    learned: LearnedFrom,
    /// When it was made.
    made_at: SystemTime,
}

impl Adapter {
    /// An adapter over one grant's folder.
    #[must_use]
    pub fn of(
        file: &Path,
        applies_to: &str,
        the_base_was: &str,
        learned: LearnedFrom,
        made_at: SystemTime,
    ) -> Self {
        Self {
            file: file.to_owned(),
            derived: Vec::new(),
            applies_to: applies_to.to_owned(),
            the_base_was: the_base_was.to_owned(),
            learned,
            made_at,
        }
    }

    /// The adapter itself — the canonical safetensors file.
    #[must_use]
    pub fn file(&self) -> &Path {
        &self.file
    }

    /// **Note a copy converted for a runtime.**
    ///
    /// It is derived from [`file`](Self::file) and can always be made again, so
    /// nothing reads it as the truth; and it holds the same learning, so it is
    /// deleted with the adapter.
    #[must_use]
    pub fn and_the_copy_made_for_a_runtime(mut self, copy: &Path) -> Self {
        self.derived.push(copy.to_owned());
        self
    }

    /// Every copy made from this adapter for a runtime.
    #[must_use]
    pub fn derived_copies(&self) -> &[PathBuf] {
        &self.derived
    }

    /// The base model it applies to.
    #[must_use]
    pub fn applies_to(&self) -> &str {
        &self.applies_to
    }

    /// What it was trained on, and under which grant.
    #[must_use]
    pub fn learned(&self) -> &LearnedFrom {
        &self.learned
    }

    /// When it was made.
    #[must_use]
    pub fn made_at(&self) -> SystemTime {
        self.made_at
    }

    /// **Whether this adapter is what a given grant taught** — the question a
    /// revocation asks, and the reason an adapter carries its grant at all.
    #[must_use]
    pub fn came_from(&self, folder: &Path, grantee: &str) -> bool {
        self.learned.under_the_grant_to == grantee
            && self.learned.folders.iter().any(|granted| granted == folder)
    }

    /// **Delete it and every copy made from it**, which takes back what that
    /// grant taught and nothing else.
    ///
    /// The derived copies go first. A machine interrupted halfway through would
    /// otherwise be left holding the runtime's copy — the whole of the learning
    /// — with the canonical file gone and nothing left to say where it came
    /// from.
    ///
    /// # Errors
    /// Whatever the filesystem says. A file that is already gone is not an
    /// error: the person asked for it not to be there.
    pub fn delete(&self) -> std::io::Result<()> {
        for copy in &self.derived {
            gone(copy)?;
        }
        gone(&self.file)
    }
}

/// Remove a file, and count a file that was already absent as removed.
fn gone(path: &Path) -> std::io::Result<()> {
    match std::fs::remove_file(path) {
        Err(why) if why.kind() == std::io::ErrorKind::NotFound => Ok(()),
        other => other,
    }
}

/// **What every adapter is composed over: a base model nothing wrote to.**
///
/// Held by digest, taken before a fine-tune and checked after. A stack that
/// merged into the base would change it, and [`TheBaseIsUntouched::still_holds`]
/// is where that is caught — in a test, before anything ships, rather than in a
/// person's machine a year later when they ask for one folder back.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TheBaseIsUntouched {
    /// The model, as the catalogue names it.
    model: String,
    /// What its bytes were before anything trained.
    was: String,
}

impl TheBaseIsUntouched {
    /// Record what the base model is, before a fine-tune runs.
    #[must_use]
    pub fn before(model: &str, digest: &str) -> Self {
        Self {
            model: model.to_owned(),
            was: digest.to_owned(),
        }
    }

    /// The model this is about.
    #[must_use]
    pub fn model(&self) -> &str {
        &self.model
    }

    /// The digest it must still have.
    #[must_use]
    pub fn was(&self) -> &str {
        &self.was
    }

    /// **Whether the base model is still byte-for-byte what it was.**
    ///
    /// # Errors
    /// [`TheBaseMoved`] naming the model, with both digests, when it is not.
    /// Nothing recovers from this: an adapter over a base that changed is an
    /// adapter nobody can reason about, and a fine-tune that rewrote the base
    /// has already taken away what this whole design exists to keep.
    pub fn still_holds(&self, digest_now: &str) -> Result<(), TheBaseMoved> {
        if digest_now == self.was {
            return Ok(());
        }
        Err(TheBaseMoved {
            model: self.model.clone(),
            was: self.was.clone(),
            is_now: digest_now.to_owned(),
        })
    }
}

/// The base model was written to, which nothing here may do.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error(
    "the base model {model} changed while a fine-tune ran: it was {was} and is now {is_now}. \
     What a fine-tune learns belongs in an adapter beside the model (ADR 0048); a stack that \
     merges into the base takes away a person's ability to remove what one folder taught, and \
     that cannot be given back afterwards"
)]
pub struct TheBaseMoved {
    /// Which model.
    pub model: String,
    /// What it was.
    pub was: String,
    /// What it is now.
    pub is_now: String,
}

/// **Every adapter a model answers with**, in the order they are applied.
///
/// v0.5 composes one. The type takes a list because the day a second granted
/// folder is trained on must not be the day this shape is argued about again.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Composed {
    /// The adapters, in order.
    adapters: Vec<Adapter>,
}

impl Composed {
    /// Nothing composed: the model as it shipped.
    #[must_use]
    pub fn nothing() -> Self {
        Self::default()
    }

    /// Answer with this adapter as well.
    #[must_use]
    pub fn and(mut self, adapter: Adapter) -> Self {
        self.adapters.push(adapter);
        self
    }

    /// Every adapter being applied.
    #[must_use]
    pub fn adapters(&self) -> &[Adapter] {
        &self.adapters
    }

    /// **Take back what one grant taught**, leaving every other adapter where it
    /// was — the whole point of one adapter per folder.
    ///
    /// # Errors
    /// Whatever the filesystem says while deleting the files.
    pub fn without_what_that_grant_taught(
        mut self,
        folder: &Path,
        grantee: &str,
    ) -> std::io::Result<Self> {
        for adapter in self
            .adapters
            .iter()
            .filter(|adapter| adapter.came_from(folder, grantee))
        {
            adapter.delete()?;
        }
        self.adapters
            .retain(|adapter| !adapter.came_from(folder, grantee));
        Ok(self)
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;

    /// An adapter over a folder, with a file on the disk to delete.
    fn an_adapter(folder: &str, grantee: &str, at: &Path) -> Adapter {
        let file = at.join(format!("{}-{grantee}.adapter", folder.replace('/', "_")));
        std::fs::write(&file, b"what this folder taught").unwrap();
        Adapter::of(
            &file,
            "qwen2.5-7b-instruct",
            "sha256-2bada8a7",
            LearnedFrom {
                folders: vec![PathBuf::from(folder)],
                under_the_grant_to: grantee.to_owned(),
                taken_at: SystemTime::UNIX_EPOCH,
                learned_from: 3,
                left_out: 0,
            },
            SystemTime::UNIX_EPOCH,
        )
    }

    fn a_folder() -> PathBuf {
        use std::sync::atomic::{AtomicU32, Ordering};
        static NEXT: AtomicU32 = AtomicU32::new(0);
        let path = std::env::temp_dir().join(format!(
            "alo-adapting-adapter-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir_all(&path).unwrap();
        path
    }

    /// **Deleting one adapter takes back what one folder taught, and leaves the
    /// rest** (ADR 0048, decisions 4 and 5).
    #[test]
    fn deleting_one_adapter_takes_back_one_folder_and_leaves_the_others() {
        let at = a_folder();
        let invoices = an_adapter("/home/anna/Invoices", "@adapting", &at);
        let letters = an_adapter("/home/anna/Letters", "@adapting", &at);
        let (invoices_file, letters_file) = (invoices.file().to_owned(), letters.file().to_owned());

        let composed = Composed::nothing().and(invoices).and(letters);
        assert_eq!(composed.adapters().len(), 2);

        let after = composed
            .without_what_that_grant_taught(Path::new("/home/anna/Invoices"), "@adapting")
            .unwrap();

        assert_eq!(after.adapters().len(), 1, "one folder's learning remains");
        assert!(
            !invoices_file.exists(),
            "what the revoked folder taught is still on the disk"
        );
        assert!(
            letters_file.exists(),
            "revoking one folder took away what another folder taught"
        );
        std::fs::remove_dir_all(&at).unwrap();
    }

    /// **Deleting an adapter deletes the runtime's copy of it as well.**
    ///
    /// The converted copy is the same learning in another container. A machine
    /// that kept it would have taken nothing back, and the sentence a person
    /// read when they deleted it would be false.
    #[test]
    fn deleting_an_adapter_deletes_every_copy_made_from_it() {
        let at = a_folder();
        let canonical = an_adapter("/home/anna/Invoices", "@adapting", &at);
        let copy = at.join("anna-lora.gguf");
        std::fs::write(&copy, b"the same learning, converted for one runtime").unwrap();
        let adapter = canonical.and_the_copy_made_for_a_runtime(&copy);
        assert_eq!(adapter.derived_copies(), std::slice::from_ref(&copy));

        adapter.delete().unwrap();

        assert!(!adapter.file().exists(), "the adapter is still there");
        assert!(
            !copy.exists(),
            "the runtime's copy outlived the adapter, so nothing was taken back"
        );
        std::fs::remove_dir_all(&at).unwrap();
    }

    /// **An adapter knows which grant produced it**, which is what a revocation
    /// asks it.
    #[test]
    fn an_adapter_says_which_grant_it_came_from() {
        let at = a_folder();
        let adapter = an_adapter("/home/anna/Invoices", "@adapting", &at);
        assert!(adapter.came_from(Path::new("/home/anna/Invoices"), "@adapting"));
        assert!(!adapter.came_from(Path::new("/home/anna/Letters"), "@adapting"));
        assert!(!adapter.came_from(Path::new("/home/anna/Invoices"), "@somebody-else"));
        assert_eq!(adapter.applies_to(), "qwen2.5-7b-instruct");
        std::fs::remove_dir_all(&at).unwrap();
    }

    /// **The base model is byte-for-byte what it was**, and a stack that merged
    /// into it is caught here rather than in somebody's machine.
    #[test]
    fn a_fine_tune_that_wrote_to_the_base_model_is_refused() {
        let base = TheBaseIsUntouched::before("qwen2.5-7b-instruct", "sha256-2bada8a7");
        assert_eq!(base.still_holds("sha256-2bada8a7"), Ok(()));

        let moved = base.still_holds("sha256-something-else").unwrap_err();
        assert_eq!(moved.was, "sha256-2bada8a7");
        assert_eq!(moved.is_now, "sha256-something-else");
        assert!(
            moved.to_string().contains("adapter beside the model"),
            "the refusal does not say where the learning belongs"
        );
    }

    /// **A model with nothing composed is the model as it shipped.**
    #[test]
    fn a_model_with_no_adapters_is_the_model_as_it_shipped() {
        assert!(Composed::nothing().adapters().is_empty());
    }
}
