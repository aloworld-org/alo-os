//! What the person chose about their own machine, and the bound an
//! organisation set around it.
//!
//! [ADR 0016](../../../docs/decisions/0016-the-organisation-bounds-and-the-person-chooses.md)
//! settles the tension this crate exists in the middle of. ADR 0004 gives
//! `/etc/alo/agentd.toml` to whoever manages a machine; ADR 0008 gives *where a
//! question may be answered* to the person sitting at it. They are two
//! settings, two owners and two files:
//!
//! | | Owner | Where | What |
//! |---|---|---|---|
//! | The bound | the organisation | `/etc/alo/agentd.toml` | `alo_models::SourcePolicy` — which places are permitted at all |
//! | The choice | the person | [`THE_SETTINGS`], under `$XDG_CONFIG_HOME/alo` | which model answers, which weights they brought themselves, and which language they read |
//!
//! **This crate is the person's half**, and it holds the one place the two meet
//! — [`Chosen::asking`], which is the only door in it that produces an
//! `alo_answering::Answering`.
//!
//! # A choice outside the bound is refused out loud
//!
//! Never quietly replaced with a permitted one, which is the comfortable
//! failure ADR 0016 names: a person picks a provider, the rule forbids egress,
//! and the machine answers anyway on its own hardware. Nothing appears broken,
//! and the person believes they know where their question went. So
//! [`Chosen::asking`] hands back the permission for **the place that was
//! chosen** or the rule's own refusal — `alo_models::NotAllowed`, in the rule's
//! own words — and there is no method here that answers with a place somebody
//! did not pick.
//!
//! # Nothing here recommends anything
//!
//! `alo_models::Catalogue::agent_for_cpu` is the other crate's *which model
//! would this machine give an agent*; this one holds what somebody actually
//! chose, and it is deliberately unable to invent one. A settings store that
//! could produce a choice would be ADR 0016's rejected *the organisation sets a
//! default* wearing different clothes: a default is a choice, made by whoever
//! set it. A machine nobody has configured therefore has no answer here at all
//! — [`Settings::untouched`] — and whoever asks it a question is told so.
//!
//! # What a missing file means, and what a broken one does not
//!
//! **No file at all is a person who has not chosen**, not an error:
//! [`Settings::at`] answers [`Settings::untouched`]. That is deliberately the
//! opposite of `alo_keeping::Reading`, where a missing record is refused rather
//! than read as *nothing happened* — a record is evidence alo OS itself writes,
//! and a settings file is a thing somebody may simply never have made.
//!
//! **A file that is there and wrong is refused whole**, in words naming what
//! was wrong with it, and nothing in it is honoured. Half a settings file is
//! the machine choosing the other half.
//!
//! # A choice cannot outrun the list it names
//!
//! Since [ADR 0019](../../../docs/decisions/0019-a-runtime-is-found-not-configured.md)
//! one of the two lists a choice can name lives **here**: the weights somebody
//! brought are the person's, and they go beside the choice rather than in a
//! store of their own. [`Settings::of`] is what that costs — a choice naming
//! the brought list must name an entry on it, refused where the pair is made
//! rather than answered with a `None` further on. [`Settings::weights`]
//! promises the entry because of it.
//!
//! The catalogue is deliberately not checked the same way: it ships with the
//! release rather than living in this file, and a model already on somebody's
//! disk is theirs to ask. The one list this crate can contradict itself about
//! is the one it holds.
//!
//! # What is not here yet
//!
//! **A machine in the next room.** ADR 0008 permits it and ADR 0003 says what
//! pairing is, and this machine keeps no list of paired machines anywhere — so
//! a choice naming one is a file that fails to read rather than a setting that
//! silently does nothing. `alo-asking` refuses it too: *nothing anywhere
//! reaches a machine on this network yet.*
//!
//! **A provider is here since format 2**, which is the second of the three
//! choices `docs/features.md` names, and the third — alo's own service — is the
//! same one, because ADR 0014 makes it one more provider with no special case
//! anywhere in the code. What is *not* here is a credential: the file holds a
//! provider's address and the region whoever added it stated, and the key lives
//! in a keyring under a name derived from the provider's own.
//!
//! **And there is no keyring on this machine yet.** `alo_models::SecretRef`
//! names where a key would live and nothing keeps one, so a provider that needs
//! a credential is chosen, persisted and refused at the moment of asking —
//! never asked without its key, and never answered somewhere else instead.
//!
//! **And an address, which is not coming.** Where a model runtime on this
//! machine is, is `alo_models`' adapter's own knowledge — ADR 0019 — so there
//! is no key for one here and no key for one in the organisation's file
//! either.

mod bound;
mod chosen;
mod place;
mod refusing;
mod settings;
#[cfg(test)]
mod testing;
mod unreadable;
mod words;
mod written;

pub use chosen::{Chosen, NoModel, NoProvider, Picked, Which};
pub use place::{CONFIG_HOME, HOME, THE_FOLDER, THE_SETTINGS, where_it_is};
pub use refusing::NotSet;
pub use settings::{Settings, Unresolved};
pub use unreadable::{At, NotToml};
pub use words::{EVERY_WORD, Word, WordsError, choosing_words, declare_into};
pub use written::{ALSO_READ, THE_FORMAT, is_a_shape_we_read};
