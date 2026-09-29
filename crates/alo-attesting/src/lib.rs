//! What left this machine in a period, as one statement an auditor can check.
//!
//! `docs/features.md` promises this at tier **v1**: *a signed, printable
//! statement of exactly what left this machine in a period. The artifact an
//! auditor asks for and nobody can currently produce.*
//!
//! The tier is written `v1` rather than in the square brackets `docs/features.md`
//! uses, because rustdoc reads those as a link and denies the warning — the same
//! reason `alo-image`'s own documentation spells tiers out.
//!
//! This crate is the **payload** — the statement and its digest. It holds no key
//! and signs nothing, for the reason in *What this does not do* below.
//!
//! # It reads the record and states nothing of its own
//!
//! Every number here comes from [`alo_record::Record`]. That is not tidiness: law
//! 1's record already cannot hold a departure the indicator never showed —
//! `alo-record`'s `departed` module is *the only door from an egress into the
//! record*, and [`Happened::Left`](alo_record::Happened::Left) is reachable only
//! from an [`alo_egress::Departing`]. So the record is the account of what left,
//! and an attestation that counted anything separately would be a second account
//! to keep true.
//!
//! Whether something caused egress is likewise **a variant rather than a
//! calculation** — `Happened::caused_egress` is `Left | LeftOnItsOwn` and nothing
//! else. This crate does not decide what counts as leaving; it reports what the
//! record already settled.
//!
//! # Four things an auditor asks, and the record already separates them
//!
//! | | |
//! |---|---|
//! | What left, under whose authority, to where | [`Happened::Left`](alo_record::Happened::Left) |
//! | What the policy **refused** to let leave | [`Happened::HeldBack`](alo_record::Happened::HeldBack) |
//! | What alo OS reached on its own (★ *no telemetry*) | [`Happened::LeftOnItsOwn`](alo_record::Happened::LeftOnItsOwn) |
//! | What was answered here and never left | [`Happened::AnsweredHere`](alo_record::Happened::AnsweredHere) |
//!
//! **The refusals are half the artifact.** `alo-record` says it of itself: a
//! record keeping only successes cannot answer what a security review actually
//! asks, which is not *what did it do* but *what did it try*. A statement listing
//! only departures would be the same omission wearing a signature.
//!
//! # The measurement law 1 promises to publish
//!
//! > With a local model a working day produces **zero** inference egress,
//! > measured at the network boundary — and we publish the measurement rather
//! > than the promise.
//!
//! That measurement is [`Statement::answered_here`] against the departures whose
//! reason was a question. It is the one number in here that is a **claim about
//! the product** rather than a log of events, and it is why this artifact is
//! worth more than a list: a promise is a sentence, and this is the same sentence
//! with a number behind it that is either true on a machine or is not.
//!
//! # Why a format number, from the first commit
//!
//! [`THE_FORMAT`] is in the rendered statement because **a signature commits to
//! bytes.** The day somebody rewords a sentence in [`rendering`] — and on this
//! repository somebody will, because sentences are reworded here for good reasons
//! constantly — every attestation signed before that change stops verifying
//! against a fresh render of the same record. Not wrong and not tampered:
//! **unverifiable**, which an auditor cannot tell apart from tampered.
//!
//! A version in the payload turns that into *render it the way version 1 rendered
//! it*. It cannot be added afterwards, because the artifacts that would need it
//! are the ones already signed. ADR 0068 is the same problem one layer up and its
//! answer — a published sentence whose meaning changes gets a new key — is the
//! precedent.
//!
//! # Rendered the same way by anybody
//!
//! Two auditors rendering the same period from the same record must get the same
//! bytes, or the digest settles nothing. So:
//!
//! - the period's ends are **named and closed at one end only** — see [`Period`],
//!   which is half-open, because *the same day* rendered two ways by two people
//!   is two digests and no way to tell which is wrong;
//! - the order of entries is **fixed here** rather than inherited from whatever
//!   the record iterates in, for the same reason;
//! - and every count is of entries in the period, decided by the entry's own
//!   `at`, never by when the statement was made.
//!
//! # What this does not do, and why that is not a gap
//!
//! **It holds no key, signs nothing, and cannot.** There is a test that it
//! reaches for no key, in the shape `alo-image` already uses for its own workflow
//! — whose best line is that *the comment saying the owner signs is not signing*.
//!
//! That is not scope avoidance. **Who signs an attestation is a decision this
//! crate must not make**, and the reasoning belongs in an ADR. What is settled
//! enough to write here is the shape of it, because a reader of this crate will
//! otherwise re-derive it:
//!
//! A vendor-signed attestation requires **the machine's record of what left it to
//! travel to the vendor to be signed**. On a product sold on sovereignty that is
//! disqualifying rather than merely awkward: the flagship compliance artifact
//! would be one whose production is itself an egress to us, and law 1's own
//! indicator would fire to make it. *Send only the digest* does not rescue it — a
//! signature over bytes nobody saw attests *somebody handed me these*, not *this
//! machine's account is genuine*, which is notarisation wearing an attestation's
//! clothes, and it still makes a customer's compliance artifact depend on our
//! uptime and their connectivity.
//!
//! So the machine signs its own. The guarantee that makes that safe is **a key
//! generated on the machine in hardware that cannot export it** — a property of
//! the silicon. It is deliberately *not* the argument that an agent has no path to
//! the key because the capability model does not list one: that rests on a policy
//! staying as strict as it is today, and a policy is a thing somebody widens in a
//! change that looks unrelated. Non-exportability survives a future capability
//! model sloppier than this one.
//!
//! **And an attestation says *this machine said this*, never *this is true*.** A
//! compromised machine signs false attestations while it is compromised, and no
//! scheme fixes that — the vendor branch would have us signing a compromised
//! machine's record with a straight face. What hardware buys is that an attacker
//! cannot take the key with them, cannot forge attestations after eviction, and
//! cannot produce one anywhere but on that machine.
//!
//! None of this changes what the payload contains or how it is rendered, which is
//! why the payload is buildable now and blocked on nothing.
//!
//! # What is in it about people, which is a decision and not a field
//!
//! Each departure names the **grantee** it left under — an agent or an
//! application, `org.gnome.Cheese` and not a person's name; see
//! `alo_capability::Grantee`, whose two kinds never answer for each other — named
//! in prose rather than linked, because this crate reads a record and never makes
//! a grant, so `alo-capability` is a dev-dependency here and a rustdoc link would
//! point at something this crate does not depend on.
//!
//! But per-departure attribution with timestamps is a finer-grained document than
//! *what left this machine*, and this artifact is built to be handed to a third
//! party. **An auditor usually needs to know what left and where it went; a
//! security team investigating an incident needs to know which agent.** Those are
//! different questions, and answering the second by default in a document designed
//! to leave the machine is a choice rather than a detail.
//!
//! So [`THE_FORMAT`] 1 is **the detailed form**, deliberately. Whether the form an
//! auditor receives should instead be destinations and counts with no per-event
//! attribution is the ADR's to decide, and a summary is what a format 2 would be
//! for. It is written here so that the decision is made in the open rather than
//! inherited from a field that arrived in an example.
//!
//! # A note on language, left open on purpose
//!
//! `CLAUDE.md` requires user-facing strings to be externalised from day one, and
//! an auditor is a person who may not read English. But a signature covers bytes,
//! so a statement rendered in two languages would be two digests over one set of
//! facts.
//!
//! **The tension dissolves once the payload is seen for what it is: a data format,
//! not prose.** Its field names are identifiers exactly as `format` and every key
//! in `dock.toml` are identifiers, and nobody proposes translating a config key.
//! The document a person reads is a **translated rendering of** this payload, and
//! the payload is what is signed. One artifact, two surfaces: a reader in any of
//! the 24 languages gets a page they can read, a verifier anywhere gets the same
//! bytes, and the strings live where the i18n rule wants them.
//!
//! Two things that rendering must carry, and the second is the one that is easy to
//! miss. It must show **the digest of the payload it was rendered from**, or it is
//! a claim with nothing behind it. And it must say **that it is a rendering and
//! not the record** — because a well-typeset page in an auditor's language
//! carrying a real digest *looks* like evidence, and a maliciously rendered one
//! carrying the same real digest looks exactly as convincing. *Verify the payload;
//! this is a reading of it* belongs in the vocabulary, translated with the rest.
//!
//! This crate renders the payload and no page, so neither is built here. They are
//! written down so that whoever builds the page finds the requirement rather than
//! discovering it from an auditor who trusted a rendering.

#![doc(html_root_url = "https://github.com/aloworld-org/alo-os")]

pub mod held_by;
pub mod period;
pub mod rendering;
pub mod statement;

pub use held_by::HeldBy;
pub use period::Period;
pub use rendering::{THE_FORMAT, digest_of, rendered};
pub use statement::Statement;
