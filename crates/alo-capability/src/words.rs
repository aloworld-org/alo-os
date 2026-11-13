//! Every string this crate can say, and the English beside each one.
//!
//! `CLAUDE.md` says hardcoded English is a bug. This is the list that stops it
//! being one here: a key, the sentence in the language the code is written in,
//! and the note a translator needs. `alo-strings` does the rest.
//!
//! It is the fourth list of its kind — `alo-files` in item 9b, `alo-shortcuts`
//! in 9c, `alo-appearance` in 9d — and it is deliberately the same shape as
//! those three. What is different is who reads it.
//!
//! # These are the sentences a person reads when they are told no
//!
//! Almost nothing in this crate is a label. A grant that could not be made, an
//! argument that did not survive the boundary, a change that was never put to
//! anybody, a call refused at the moment it would have run: every string here
//! is somebody being told that something did not happen. Two things follow.
//!
//! **They say what to do, not what went wrong.** That was true of this crate's
//! English before it moved here, and the notes say so where a translation could
//! lose it — *invalid argument* is not a translation of *give {argument} as a
//! full path*.
//!
//! **They are read twice.** A refusal is shown to a person and then written
//! into the record (ADR 0001 §7), where a security review reads it afterwards.
//! Both readings come from one rendering of one string, which is what
//! [`crate::Refused`] carrying a refusal rather than a sentence is for.
//!
//! **Two of them are not refusals**, and they are the two at the end:
//! [`HAS_AN_AGENT`] and [`HAS_NO_AGENT`] are what the one place in Settings
//! that offers ADR 0009's fourth choice says about this machine. They are here
//! rather than in a shell because the consequence each of them describes is
//! this crate's — turning the agent off ends every grant on the machine — and
//! a sentence about that written anywhere else would be a second account of
//! what [`crate::Agent::declining`] actually does.
//!
//! # What is not here, and why that is not an oversight
//!
//! [`crate::VerbError`], [`crate::VerbsError`] and [`crate::SentenceError`]
//! keep their English and their `Display`. They are refusals of a **verb
//! declaration**: they are read by whoever is writing an adapter against
//! `docs/contracts/agent-verbs.md` at the moment their declaration fails to
//! compile past its own tests, never by whoever is using the machine. It is
//! `alo-shortcuts`' `DefaultsError` one crate on — a release's own list
//! contradicting itself is read by whoever is fixing it, and translating it
//! would hand that person a sentence in whichever language happened to be
//! loaded.

use alo_strings::{Key, Plural, Vocabulary};

/// One string a crate can say.
///
/// Lifted into `alo-strings` by item 9d. Re-exported here because this crate's
/// own files, and the tests that read this list, name it as
/// `crate::words::Word`.
pub use alo_strings::Word;

/// One string this crate can say about a number of things.
///
/// **The second copy of this struct**, `alo-files`' being the first. Two copies
/// are not a pattern and a third would be, which is the rule [`Word`] itself
/// moved under in item 9d: a third lifts it into `alo-strings` beside `Word`.
/// It is written out rather than borrowed from `alo-files` because this crate
/// does not depend on that one and must not start.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Counted {
    /// What names it.
    named: &'static str,
    /// The gap the number goes in.
    number: &'static str,
    /// What it says about one thing.
    one: &'static str,
    /// What it says about any other number of things.
    other: &'static str,
    /// What a translator needs to know.
    note: &'static str,
}

impl Counted {
    /// What names it.
    #[must_use]
    pub fn key(&self) -> Key {
        Key::unchecked(self.named)
    }
}

// ---------------------------------------------------------------------------
// A grant that could not be made — [`crate::GrantError`].
//
// Read by somebody who has just picked the wrong thing in a file chooser.
// ---------------------------------------------------------------------------

/// A grant naming no agent.
pub const ANONYMOUS: Word = Word::saying(
    "capability.grant.anonymous",
    "say which agent this grant is for — a grant to nobody reaches nothing",
);

/// A grant naming no application.
pub const NO_APPLICATION_NAMED: Word = Word::saying(
    "capability.grant.no-application-named",
    "say which application this grant is for — a grant to no application reaches nothing",
);

/// A grant over nothing at all.
pub const NOTHING_NAMED: Word = Word::saying(
    "capability.grant.nothing-named",
    "choose the folder, file or application this grant is over",
);

/// A grant to `/`, which ADR 0001 §3 does not allow.
pub const THE_WHOLE_MACHINE: Word = Word::saying(
    "capability.grant.the-whole-machine",
    "there is no grant to the whole machine — pick the folder you actually mean",
);

/// A grant over a path that is not a full path.
pub const GRANT_NOT_A_FULL_PATH: Word = Word::saying(
    "capability.grant.not-a-full-path",
    "grant a folder by its full path, so it means the same thing wherever it is asked about",
);

/// A grant over a path with `..` in it.
pub const GRANT_COULD_LEAD_ELSEWHERE: Word = Word::saying(
    "capability.grant.could-lead-elsewhere",
    "a path with .. in it can lead somewhere else — grant the folder you mean by its own path",
)
.noting("\"..\" is how a path says \"the folder above\" and is never translated.");

/// A grant lasting no time.
pub const GRANT_NO_TIME: Word = Word::saying(
    "capability.grant.no-time",
    "say how long the grant should last — a grant for no time reaches nothing",
);

/// A grant with no end this machine can represent.
pub const GRANT_NO_END: Word = Word::saying(
    "capability.grant.no-end",
    "a grant has to end — choose how long this one should last",
);

/// A grant to an agent over something that is not a path (ADR 0040).
pub const NOT_FOR_AN_AGENT: Word = Word::saying(
    "capability.grant.not-for-an-agent",
    "the camera, the microphone, the screen and the like are granted only to applications — an \
     agent is offered what it needs at the moment you ask it",
)
.noting(
    "An agent is an AI assistant, not a person. The second half is a rule of alo OS rather than \
     an apology: what an agent sees is offered to it when a person asks it something, and never \
     watched in the background.",
);

/// A grant to an agent over a terminal, which ADR 0043 does not allow.
pub const PERSONS_OWN: Word = Word::saying(
    "capability.grant.a-persons-own",
    "you have not allowed alo to open a terminal",
)
.noting(
    "An agent is an AI assistant, not a person. A terminal is the window a person types commands \
     into. alo can open one for you if you allow it, and even then nothing lets it type into one \
     — what it opens is a window only you can use. This refusal means nobody has allowed it on \
     this machine.",
);

// ---------------------------------------------------------------------------
// What a grant is over — [`crate::Reach`], and the applications half of
// [`crate::Ask`].
//
// These are the line a person reads in their list of grants, and they are also
// what goes inside a refusal. A refusal and the thing named inside it are in
// one language, which is why these are strings rather than something the caller
// assembles.
// ---------------------------------------------------------------------------

/// A folder, and everything under it.
pub const A_FOLDER: Word = Word::saying("capability.reach.folder", "{path} and everything in it")
    .noting(
        "{path} is a folder on the person's own disk. It is never translated, and the sentence \
         reads better with it at the front in most languages.",
    );

/// Exactly one file.
pub const A_FILE: Word = Word::saying("capability.reach.file", "the file {path}")
    .noting("{path} is a file on the person's own disk and is never translated.");

/// One installed application.
pub const AN_APPLICATION: Word = Word::saying(
    "capability.reach.application",
    "the application {application}",
)
.noting(
    "{application} is the identifier the system knows an application by, like \
     org.blender.Blender. It is never translated.",
);

// ---------------------------------------------------------------------------
// What this machine has that is not a path — [`crate::Facility`], ADR 0040.
//
// Clauses, like the three above: each is what a grant is over in a person's
// list, and what goes inside a refusal. Lowercase, because none is ever read on
// its own.
// ---------------------------------------------------------------------------

/// The camera.
pub const THE_CAMERA: Word = Word::saying("capability.facility.camera", "the camera").noting(
    "What an application may be granted: whichever camera the machine has. A clause that goes \
     inside another sentence, so it stays lowercase.",
);

/// The microphone.
pub const THE_MICROPHONE: Word = Word::saying("capability.facility.microphone", "the microphone");

/// One picture of the screen.
pub const THE_SCREEN_ONCE: Word = Word::saying(
    "capability.facility.screen-once",
    "one picture of the screen",
)
.noting("A screenshot, taken once — a different grant from recording or sharing the screen.");

/// The screen, continuously.
pub const THE_SCREEN_CONTINUOUSLY: Word = Word::saying(
    "capability.facility.screen-continuously",
    "the screen, continuously",
)
.noting("Recording the screen or sharing it in a call — a different grant from one picture of it.");

/// Showing the person notifications.
pub const NOTIFICATIONS: Word = Word::saying(
    "capability.facility.notifications",
    "sending you notifications",
);

/// What the person copied.
pub const THE_CLIPBOARD: Word = Word::saying("capability.facility.clipboard", "what you copied")
    .noting("The clipboard: whatever the person last copied.");

/// The desktop background.
pub const THE_DESKTOP_BACKGROUND: Word = Word::saying(
    "capability.facility.desktop-background",
    "the desktop background",
);

/// The person's appearance settings.
pub const APPEARANCE_SETTINGS: Word = Word::saying(
    "capability.facility.appearance-settings",
    "your appearance settings",
)
.noting("Light or dark, the accent colour and contrast — what an application reads to match them.");

/// Whether the machine sleeps.
pub const SLEEP: Word = Word::saying("capability.facility.sleep", "keeping the machine awake")
    .noting("Stopping the machine from sleeping, for example during a presentation.");

/// The state of the network.
pub const THE_NETWORK_STATE: Word = Word::saying(
    "capability.facility.network-state",
    "the state of the network",
)
.noting("Whether the machine is connected, and how — never what is sent over it.");

/// The power profile.
pub const THE_POWER_PROFILE: Word =
    Word::saying("capability.facility.power-profile", "the power profile")
        .noting("Whether the machine is saving power, balanced, or running at full performance.");

/// The application's own secrets in the person's keyring.
pub const ITS_OWN_SECRETS: Word = Word::saying(
    "capability.facility.secrets",
    "a place for its own passwords in your keyring",
)
.noting(
    "Read after \"has been granted\", with an application as the one granted. The application \
     keeps and reads back only the passwords and keys it stored itself — never anybody else's, \
     and never the machine's own. \"Keyring\" is where a person's passwords are kept, locked \
     with their sign-in.",
);

// ---------------------------------------------------------------------------
// What the grants say when they refuse — [`crate::NotGranted`] and
// [`crate::NotAuthorised`].
//
// The two halves of the grants' answer say different things because they need
// different things from the reader: an expiry is fixed by granting again, and a
// path nobody granted is not.
// ---------------------------------------------------------------------------

/// A grant that covered it and has run out.
pub const HAS_EXPIRED: Word = Word::saying(
    "capability.refused.expired",
    "{agent} had a grant to {reach} and it has expired — grant it again to let {agent} reach \
     {wanted}",
)
.noting(
    "{agent} is the name of an agent, written the way it was granted, and is never translated. \
     {reach} is what the grant was over and {wanted} is what was asked for; both arrive already \
     in the reader's language.",
);

/// Nothing has ever covered it.
pub const NEVER_GRANTED: Word = Word::saying(
    "capability.refused.never-granted",
    "{agent} has not been granted {wanted} — grants are made by picking a folder, never by asking \
     for one",
)
.noting(
    "The second half is the whole capability model in one clause: a person grants by choosing \
     something, and no amount of asking widens what an agent may reach. {agent} is never \
     translated; {wanted} arrives already in the reader's language.",
);

/// An application nothing this agent holds has ever covered.
///
/// Its own sentence because [`NEVER_GRANTED`]'s second half — *grants are made
/// by picking a folder* — is false of an application: no folder grant reaches
/// one. Walked on a booted machine on 2026-11-08, *open the text editor* was
/// refused with that sentence and the person was told to *pick a folder in
/// Settings to grant access*, which would have changed nothing.
///
/// **And it names the fix that is real** (ADR 0149): at full access alo can
/// use any application the person's account can, with no grant. A new key
/// rather than a rewording of `capability.refused.application-not-given`,
/// because the old English said what will not help and this says what will —
/// a different claim, which ADR 0068 gives a key of its own.
pub const APPLICATION_NOT_GIVEN: Word = Word::saying(
    "capability.refused.application-not-given-but-full-access-reaches-it",
    "{agent} has not been granted {wanted}, and picking a folder will not change that — choose \
     full access ('straight away, without asking') in Settings → Running code, and alo can use \
     any application you can",
)
.noting(
    "Said to a person whose agent asked to open, bring forward or close an application. The middle \
     stops them looking in the folders list for a fix that is not there; the end names the fix \
     that is. 'straight away, without asking' and 'Settings → Running code' name a choice and a \
     page as they are shown in Settings; use the same words your translation uses there. alo is \
     the assistant. {agent} is never translated; {wanted} arrives already in the reader's \
     language, as 'the application org.gnome.TextEditor'.",
);

/// A machine where the person declined the agent (ADR 0009).
pub const NO_AGENT: Word = Word::saying(
    "capability.refused.no-agent",
    "{agent} cannot reach {wanted} — this machine has no agent, so there is nothing granted and \
     nothing to grant to",
)
.noting(
    "This is not a refusal a person fixes by granting a folder, and a translation must not make \
     it read like one: they chose to run alo OS without an agent, and there is no grants panel \
     for them to go to. {agent} is never translated; {wanted} arrives already in the reader's \
     language.",
);

/// An application's grant that covered it and has run out (ADR 0040).
pub const APPLICATION_LAPSED: Word = Word::saying(
    "capability.refused.application-expired",
    "{application} was granted {reach} and that has expired — grant it again to let {application} \
     use {wanted}",
)
.noting(
    "{application} is the identifier of an application, like org.gnome.Cheese, and is never \
     translated. It is an application, not an agent and not a person. {reach} and {wanted} arrive \
     already in the reader's language.",
);

/// Nothing granted to an application has ever covered it (ADR 0040).
pub const APPLICATION_NEVER_GRANTED: Word = Word::saying(
    "capability.refused.application-never-granted",
    "{application} has not been granted {wanted} — an application reaches only what you grant it, \
     never what it asks for",
)
.noting(
    "{application} is the identifier of an application, like org.gnome.Cheese, and is never \
     translated. The second half is the rule: asking is not granting. {wanted} arrives already in \
     the reader's language.",
);

/// A change offered where only a read may go.
pub const CHANGE_WAITS: Word = Word::saying(
    "capability.refused.change-waits",
    "{verb} changes something — propose it with the sentence describing it and let one person \
     approve that, rather than running it in the turn",
)
.noting(
    "{verb} is the name of a capability, like move_file, and is never translated. \"In the turn\" \
     means \"while the person is waiting for the answer\".",
);

// ---------------------------------------------------------------------------
// An argument that did not survive the boundary — [`crate::ArgError`].
//
// Every one of them names the argument and says what to send instead. They are
// read by a person looking at a refusal, and by whoever is writing an adapter
// against the contract, and "invalid argument" would tell neither anything.
// ---------------------------------------------------------------------------

/// Text where a number was declared.
pub const WANTED_NUMBER: Word = Word::saying(
    "capability.argument.wanted-number",
    "give {argument} as a number, not as text",
)
.noting("{argument} is the name a verb declares an argument under and is never translated.");

/// A number where text was declared.
pub const WANTED_TEXT: Word = Word::saying(
    "capability.argument.wanted-text",
    "give {argument} as text, not as a number",
);

/// Nothing, or only spaces.
pub const ARGUMENT_EMPTY: Word = Word::saying(
    "capability.argument.empty",
    "say what {argument} is — it cannot be blank",
);

/// A relative path.
pub const ARGUMENT_NOT_A_FULL_PATH: Word = Word::saying(
    "capability.argument.not-a-full-path",
    "give {argument} as a full path, so it means the same thing wherever it is read",
);

/// A path with `..` in it.
pub const ARGUMENT_COULD_LEAD_ELSEWHERE: Word = Word::saying(
    "capability.argument.could-lead-elsewhere",
    "a path with .. in it can lead somewhere else — give {argument} as the path you mean",
)
.noting("\"..\" is how a path says \"the folder above\" and is never translated.");

/// A path where one name was declared.
pub const NOT_ONE_NAME: Word = Word::saying(
    "capability.argument.not-one-name",
    "{argument} is one name, not a path — give the name on its own, without folders in it",
);

/// Something that is not an application identifier.
pub const NOT_AN_IDENTIFIER: Word = Word::saying(
    "capability.argument.not-an-identifier",
    "give {argument} as an application identifier, like org.blender.Blender",
)
.noting("org.blender.Blender is an example identifier and is not translated.");

/// Longer than the verb allows.
///
/// Countable, because a length is a number of characters and how a language
/// counts is that language's business (item 9a).
pub const TOO_LONG: Counted = Counted {
    named: "capability.argument.too-long",
    number: "longest",
    one: "{argument} is longer than one character — shorten it",
    other: "{argument} is longer than {longest} characters — shorten it",
    note: "{longest} is the most characters the argument may be, so the sentence means \"longer \
           than this many\". {argument} is the name a verb declares an argument under and is \
           never translated.",
};

/// A character that cannot be read in a sentence.
pub const NOT_PRINTABLE: Word = Word::saying(
    "capability.argument.not-printable",
    "{argument} contains a character that cannot be shown — retype it in ordinary text",
)
.noting(
    "The character is left out of the sentence on purpose: it is what made the text unreadable, \
     and showing it would do the same to this refusal.",
);

/// A number outside the range the verb declared.
pub const OUT_OF_RANGE: Word = Word::saying(
    "capability.argument.out-of-range",
    "give {argument} as a number between {least} and {most}",
)
.noting("Both ends are included: {least} and {most} are themselves allowed.");

/// Something that is not one of the options.
pub const NOT_ON_THE_LIST: Word = Word::saying(
    "capability.argument.not-on-the-list",
    "{argument} has to be one of: {options}",
)
.noting(
    "{options} is a list of the names a verb offers, separated by commas — left_half, right_half. \
     They are what has to be sent rather than what a person reads, so they are never translated, \
     even though the same options are shown as ordinary words elsewhere; the punctuation before \
     them is yours to place.",
);

// ---------------------------------------------------------------------------
// Something that never became a call at all — [`crate::CallError`].
// ---------------------------------------------------------------------------

/// A verb that is not on the list.
pub const NO_SUCH_VERB: Word = Word::saying(
    "capability.call.no-such-verb",
    "there is no verb called {verb} — the list is closed, so a verb that is not on it does not \
     exist",
)
.noting(
    "{verb} is the name that was asked for, which came from outside and is never translated. \
     \"The list is closed\" is law 2: a capability nobody wrote down does not exist.",
);

/// An argument the verb does not take.
pub const NO_SUCH_ARGUMENT: Word = Word::saying(
    "capability.call.no-such-argument",
    "{verb} does not take {argument}",
)
.noting("Both gaps are names in the contract and are never translated.");

/// An argument the verb takes, that nothing gave.
pub const ARGUMENT_MISSING: Word = Word::saying(
    "capability.call.missing",
    "{verb} needs {argument} — {purpose}",
)
.noting(
    "{purpose} is what the verb says the argument is for. It arrives already in the reader's \
     language, looked up with this same vocabulary, so it is not to be repeated here.",
);

/// The same argument given twice.
pub const SAME_ARGUMENT_TWICE: Word = Word::saying(
    "capability.call.same-argument-twice",
    "{argument} was given twice — a call gives each argument one value",
);

// ---------------------------------------------------------------------------
// A change that was never put to anybody — [`crate::ProposalError`].
// ---------------------------------------------------------------------------

/// A read offered for approval.
pub const READ_DOES_NOT_WAIT: Word = Word::saying(
    "capability.proposal.read-does-not-wait",
    "{verb} answers a question rather than changing anything — run it in the turn instead of \
     asking about it",
)
.noting(
    "\"In the turn\" means \"while the person is waiting for the answer\". Asking somebody to \
     approve a question is how approving becomes a reflex, which is what this refusal exists to \
     prevent.",
);

/// A question that stands for no time at all.
pub const PROPOSAL_NO_TIME: Word = Word::saying(
    "capability.proposal.no-time",
    "say how long the question stands — one that lapses at once cannot be answered",
);

/// A question standing longer than this machine can represent.
pub const PROPOSAL_NO_END: Word = Word::saying(
    "capability.proposal.no-end",
    "a question has to lapse — choose how long this one stands for",
);

// ---------------------------------------------------------------------------
// A question that could not be answered — [`crate::AnswerError`].
// ---------------------------------------------------------------------------

/// A number that is not waiting for an answer.
pub const NOTHING_WAITING: Word = Word::saying(
    "capability.answer.nothing-waiting",
    "nothing is waiting to be approved under number {number} — it has been answered already, or \
     it was never asked",
)
.noting("{number} is the number a person answers a proposal by, as it is shown to them.");

/// A question that stood too long.
pub const LAPSED: Word = Word::saying(
    "capability.answer.lapsed",
    "\"{sentence}\" was proposed too long ago to answer — ask again if it is still wanted",
)
.noting(
    "The quotation marks are part of the sentence: use the ones your language writes. {sentence} \
     is the change that was proposed, and it arrives already in the reader's language, looked up \
     with this same vocabulary.",
);

// ---------------------------------------------------------------------------
// Which of the two kinds of machine this is — [`crate::Agent`], ADR 0009.
//
// The only two strings in this file that are not somebody being told no. They
// are the one place in Settings that offers the fourth choice, and each says
// what the act would **do** rather than naming a state, because the person
// reading one is deciding whether to perform it.
// ---------------------------------------------------------------------------

/// A machine with an agent.
pub const HAS_AN_AGENT: Word = Word::saying(
    "capability.agent.on",
    "this machine has an agent — turn it off whenever you like, and everything it has been \
     granted ends at once",
)
.noting(
    "Read by somebody deciding whether to turn the agent off, so the second half is a \
     consequence rather than a reassurance: the grants really do end, and turning the agent on \
     again does not bring them back.",
);

/// A machine where the person declined one.
pub const HAS_NO_AGENT: Word = Word::saying(
    "capability.agent.off",
    "this machine has no agent — turn one on whenever you like, and nothing is reinstalled",
)
.noting(
    "ADR 0009: declining the agent is a setting rather than a different edition of alo OS, and \
     the second half is what somebody who declined at setup needs to know. It is a statement of \
     fact and not an invitation; a translation that made it enthusiastic would be advertising to \
     somebody who has already said no once.",
);

/// The sealed box, which is how a fresh machine runs the agent's code.
pub const RUNS_IN_A_SEALED_BOX: Word = Word::saying(
    "capability.running.sealed-box",
    "in a sealed box that sees only this project",
)
.noting(
    "One of three choices for how code the agent writes is allowed to run, and the one a fresh \
     machine arrives with. A box the agent cannot reach out of: it sees the project and nothing \
     else, reaches no network except what the person approved, and is thrown away after each \
     run.",
);

/// Asking the person before each run.
pub const RUNS_AFTER_ASKING: Word = Word::saying(
    "capability.running.asking-each-time",
    "after showing you the command and waiting for you",
)
.noting(
    "One of three choices for how code the agent writes is allowed to run. The agent shows the \
     exact command and it runs only once the person approves that command. \"Waiting for you\" \
     is addressed to the person reading it.",
);

/// Running directly, without asking.
pub const RUNS_AT_FULL_TRUST: Word = Word::saying(
    "capability.running.full-trust",
    "straight away, without asking",
)
.noting(
    "One of three choices for how code the agent writes is allowed to run, and the one the \
     person switches on themselves. It must not be translated as a recommendation or as a \
     warning: the warning is a separate sentence beside it.",
);

/// What a person is told when they switch full trust on.
///
/// **ADR 0064's own words**, kept as they were written. It informs and does
/// not forbid.
pub const WHAT_FULL_TRUST_COSTS: Word = Word::saying(
    "capability.running.what-full-trust-costs",
    "text in a web page, an email or a downloaded project can steer the agent, and at this \
     level whatever it steers the agent to run, runs",
)
.noting(
    "Shown beside the choice to let the agent run code without asking, so that the person \
     switching it on knows what it costs. It is a plain statement and not a refusal, a scare or \
     a recommendation -- the person is deciding about their own machine and this is the fact \
     they need. Keep it one sentence.",
);

/// The record, which no level turns off.
pub const ALWAYS_THE_RECORD: Word = Word::saying(
    "capability.running.always.record",
    "every command and what it printed is written down",
)
.noting(
    "One of three things that stay on however the person lets code run. It restricts nothing -- \
     it is a record the person themselves can read -- which is why it is not a setting.",
);

/// The snapshot before each run, which no level turns off.
pub const ALWAYS_THE_SNAPSHOT: Word = Word::saying(
    "capability.running.always.snapshot",
    "your files are copied before each run, so it can be undone",
)
.noting(
    "One of three things that stay on however the person lets code run, including when they \
     have allowed the agent to run anything without asking. It takes no choice away: it is what \
     makes undoing possible.",
);

/// The egress indicator, which no level turns off.
pub const ALWAYS_THE_INDICATOR: Word = Word::saying(
    "capability.running.always.indicator",
    "anything that leaves this machine is shown as it happens",
)
.noting(
    "One of three things that stay on however the person lets code run. \"Leaves this machine\" \
     means sent over a network. It restricts nothing and it is what the person is owed, which is \
     why it is not a setting.",
);

/// Every string this crate can say, in the order this file declares them.
///
/// The array is what a test reads down and what [`declare_into`] walks, so a
/// word declared above and left out here is a string nothing can look up.
/// Nothing ran because what was asked for was not a runnable command.
pub const NOTHING_RAN_NOT_A_COMMAND: Word = Word::saying(
    "capability.running.not-a-command",
    "nothing ran, because what was asked for was not a command this machine could run",
)
.noting(
    "Read in a record, and shown to a person, when something asked to run a command and nothing \
     ran — the person's agent, or the person themselves, who may put the word \"try\" before a \
     command to run it in a box. The request itself was the problem rather than the machine: no full path to a program, \
     or no such program on this machine, or nowhere named that the command would be allowed to \
     reach. Nothing was attempted.",
);

/// Nothing ran because the box could not be built.
pub const NOTHING_RAN_NO_BOX: Word = Word::saying(
    "capability.running.no-box",
    "nothing ran, because this machine could not build the box it runs commands in",
)
.noting(
    "Read in a record, and shown to a person, when something asked to run a command and nothing \
     ran — the person's agent, or the person themselves, who may put the word \"try\" before a \
     command. The \"box\" is a confinement the operating system itself holds, so that a command can \
     reach only what the person chose. The machine could not build one and would not run the \
     command without it. It is a fault in the machine, not a refusal by any rule, and nothing \
     was attempted.",
);

/// Nothing ran because the command could not be started.
pub const NOTHING_RAN_WOULD_NOT_START: Word = Word::saying(
    "capability.running.would-not-start",
    "nothing ran, because the command could not be started",
)
.noting(
    "Read in a record, and shown to a person, when the box was built and the command still did \
     not start — or started and the machine then lost track of it. A fault in the machine rather \
     than a refusal by any rule.",
);

/// Nothing ran as administrator: the person closed the password window
/// (ADR 0150).
pub const NOTHING_RAN_THEY_CANCELLED: Word = Word::saying(
    "capability.administrator.they-cancelled",
    "nothing ran, because you closed the window that asked for your password",
)
.noting(
    "Read in a record, and shown to a person, after alo asked to run a command as administrator \
     (with the rights to change the whole computer) and the person closed the password window \
     instead of typing their password.",
);

/// Nothing ran as administrator: the password was not accepted.
pub const NOTHING_RAN_NOT_ACCEPTED: Word = Word::saying(
    "capability.administrator.not-accepted",
    "nothing ran, because the password was not accepted",
)
.noting(
    "Read in a record, and shown to a person, after the computer asked for their password to run \
     a command as administrator and did not accept what was given.",
);

/// Nothing ran as administrator: nothing could ask for the password.
pub const NOTHING_RAN_NOTHING_CAN_ASK: Word = Word::saying(
    "capability.administrator.nothing-can-ask",
    "nothing ran, because nothing on this computer could ask for your password: you need to be \
     signed in at its screen",
)
.noting(
    "Read in a record, and shown to a person, when alo wanted to run a command as administrator \
     and there was no desktop signed in where the password window could appear. It is not saying \
     a password was wrong.",
);

/// Nothing ran as administrator: the account is not an administrator.
pub const NOTHING_RAN_NOT_AN_ADMINISTRATOR: Word = Word::saying(
    "capability.administrator.not-an-administrator",
    "nothing ran, because your account is not an administrator on this computer",
)
.noting(
    "Read in a record, and shown to a person, when alo wanted to run a command as administrator \
     (with the rights to change the whole computer) and the person's account does not have those \
     rights here.",
);

/// Nothing ran as administrator: nobody answered the password window.
pub const NOTHING_RAN_NO_ANSWER_IN_TIME: Word = Word::saying(
    "capability.administrator.no-answer-in-time",
    "nothing ran, because the password was not given within five minutes, so the question was \
     taken back",
)
.noting(
    "Read in a record, and shown to a person, when the password window was left open for five \
     minutes without an answer and closed itself.",
);

/// Nothing ran as administrator: the part that runs it did not answer.
pub const NOTHING_RAN_AS_ADMINISTRATOR_COULD_NOT_ASK: Word = Word::saying(
    "capability.administrator.could-not-ask",
    "nothing ran, because this computer could not ask for administrator rights just now",
)
.noting(
    "Read in a record, and shown to a person, when the part of the computer that runs a command \
     as administrator, or the part that asks for the password, did not answer. A fault in the \
     computer, not a refusal by the person.",
);

/// The sealed box was joined and refused to start the program, on a machine
/// whose system layer it cannot read yet — the owner's *say so plainly, keep
/// going* of 2026-11-06 (`crate::the_sealed_box_here`).
pub const THE_SEALED_BOX_CANNOT_RUN_PROGRAMS_HERE_YET: Word = Word::saying(
    "capability.running.sealed-box-not-here-yet",
    "{program} was not run. The sealed box can't run programs on this computer yet. Choose \
     'ask each time' in Settings → Running code to run commands after you approve them.",
)
.noting(
    "Shown when a person runs a command in the sealed box and the box cannot start it, on a \
     computer where the sealed box cannot run programs yet. {program} is the program's path. \
     'ask each time' and 'Settings → Running code' name a choice and a page as they are shown \
     in Settings; use the same words your translation uses there.",
);

/// Under the sealed box in Settings, on a computer where it cannot run
/// programs yet. The person's choice is not changed by it.
pub const THE_SEALED_BOX_IS_NOT_HERE_YET: Word = Word::saying(
    "capability.running.sealed-box-not-available-here",
    "The sealed box isn't available on this computer yet.",
)
.noting(
    "A plain line under the sealed box choice in Settings → Running code, on a computer where the \
     sealed box cannot run programs yet. The choice is still there and can still be chosen.",
);

/// Nothing ran because this is not how the person said commands may run.
pub const NOTHING_RAN_NOT_THAT_WAY: Word = Word::saying(
    "capability.running.not-that-way",
    "nothing ran, because running a command this way is not what you chose",
)
.noting(
    "Read in a record, and shown to a person, when something asked for a command to be run in a \
     way that does not match how the person said commands may run on this machine. Their choice \
     stands and nothing was attempted. It is not a warning they need to act on; it is the \
     machine saying it did what they asked.",
);

/// What a person is told beside the switch that lets alo act on what they ask
/// for in the terminal without the OK.
///
/// **The owner's own words**, from `docs/features.md`, kept as they were
/// written. It informs and does not forbid.
pub const WHAT_ACTING_WITHOUT_ASKING_COSTS: Word = Word::saying(
    "capability.terminal.what-acting-without-asking-costs",
    "alo will send, change and delete what you ask for without showing you first",
)
.noting(
    "Shown beside the switch that lets alo do what the person asks for in the terminal without \
     waiting for their OK, so that the person switching it on knows what it costs. A plain \
     statement, not a refusal, a scare or a recommendation. alo is the assistant. Keep it one \
     sentence.",
);

/// Where alo works in an application, at the default: the row's name in
/// Settings (`crate::AloInApplications::InItsOwnCopy`).
pub const WORKS_IN_ITS_OWN_COPY: Word = Word::saying(
    "capability.in-applications.in-its-own-copy",
    "alo works in its own copy of an application",
)
.noting(
    "The name of a choice in Settings. alo is the assistant built into alo OS. When alo works in an application for the person — a web browser or a code editor — it starts a separate copy of that application for itself, with its own profile, so the person's own windows are left alone and only what alo's copy sends over the network is counted.",
);

/// The line under [`WORKS_IN_ITS_OWN_COPY`].
pub const WORKS_IN_ITS_OWN_COPY_IS: Word = Word::saying(
    "capability.in-applications.in-its-own-copy-is",
    "What alo's copy sends is counted and shown while it works. An application that cannot open a separate copy is refused",
)
.noting(
    "Under the choice \"alo works in its own copy of an application\" in Settings. \"Refused\" means alo will say it cannot work in that application at this setting.",
);

/// Where alo works in an application, turned up: the row's name in Settings
/// (`crate::AloInApplications::InTheirWindow`). [`NO_COPY_OF_ITS_OWN`]
/// quotes it exactly.
pub const MAY_WORK_IN_YOUR_WINDOW: Word = Word::saying(
    "capability.in-applications.in-your-window",
    "alo may work in your own window",
)
.noting(
    "The name of a choice in Settings. alo is the assistant built into alo OS. Chosen, alo may work in the window of an application the person already has open, rather than in a separate copy of its own. The sentence that tells a person alo cannot work in an application quotes this name exactly, so translate both the same way.",
);

/// The line under [`MAY_WORK_IN_YOUR_WINDOW`].
pub const MAY_WORK_IN_YOUR_WINDOW_IS: Word = Word::saying(
    "capability.in-applications.in-your-window-is",
    "Any application can be used. While alo works, the indicator says the application may be sending",
)
.noting(
    "Under the choice \"alo may work in your own window\" in Settings. The indicator is the mark at the edge of the screen that shows when something leaves this computer.",
);

/// What letting alo work in the person's own window costs: **ADR 0143's own
/// words**, from the table of settings, kept as they were written.
pub const WHAT_WORKING_IN_YOUR_WINDOW_COSTS: Word = Word::saying(
    "capability.in-applications.what-your-window-costs",
    "alo will show that the application is sending, but cannot count what it sends, and works where your other tabs are",
)
.noting(
    "Shown beside the choice that lets alo work in the person's own application window, so the person choosing it knows what it costs: alo's work happens beside their own open tabs and documents, and the computer can say that something is being sent but not how much. A plain statement, not a refusal, a scare or a recommendation. Keep it one sentence.",
);

/// **Refusal A** (the owner, 2026-11-10): at the default, an application that
/// cannot open a separate copy of its own for alo — and the Settings choice
/// that lets alo work in the person's own window instead, by its exact name and
/// page.
pub const NO_COPY_OF_ITS_OWN: Word = Word::saying(
    "capability.in-applications.no-copy-of-its-own",
    "alo cannot work in {application} safely at this setting, because it cannot open a separate copy of its own for alo. To let alo work in your own window instead, choose “alo may work in your own window” in Settings → Folders alo can use",
)
.noting(
    "Told to the person when they asked alo to do something inside an application. {application} is the application's name, such as Text Editor. alo works in a separate copy of an application so that what it sends can be counted; this one cannot start such a copy, so alo did nothing in it. The words in quotation marks are the name of a choice in Settings, and \"Folders alo can use\" is the name of the Settings page it is on: use exactly the translations those have there.",
);

/// Printed in the terminal after alo did what was asked without waiting for
/// the OK, because the person chose that.
pub const DONE_WITHOUT_ASKING: Word = Word::saying(
    "capability.terminal.done-without-asking",
    "Done without asking you first, as you chose in Settings → Terminal.",
)
.noting(
    "Printed in the terminal under what alo just did, when the person has let alo act there \
     without their OK. 'Settings → Terminal' names a page as Settings shows it; use the same \
     words your translation uses there.",
);

/// The same, for a change the machine could not keep a copy from before.
pub const DONE_WITHOUT_ASKING_AND_IT_CANNOT_BE_PUT_BACK: Word = Word::saying(
    "capability.terminal.done-without-asking-cannot-be-put-back",
    "Done without asking you first, as you chose in Settings → Terminal. This one can't be put \
     back: a copy of your files from before it could not be kept.",
)
.noting(
    "Like the sentence above, for a change the computer carried out without being able to keep \
     a copy of the person's files from before it, so Put back cannot undo it.",
);

/// What a person reads about a change alo carried out at once, because they
/// chose full access (ADR 0149).
pub const DONE_AT_FULL_ACCESS: Word = Word::saying(
    "capability.full-access.done",
    "Done without asking you first, because you chose full access in Settings → Running code: \
     {sentence}.",
)
.noting(
    "Shown under what alo did, once for each change it carried out without the person's OK, \
     because the person chose full access (shown in Settings as 'straight away, without \
     asking'). {sentence} is what was done, as the person would have been asked about it, for \
     example 'rename /home/anna/march.pdf to march-final.pdf'. 'Settings → Running code' names a \
     page as Settings shows it; use the same words your translation uses there.",
);

/// The same, for a change the machine could not keep a copy from before.
pub const DONE_AT_FULL_ACCESS_AND_IT_CANNOT_BE_PUT_BACK: Word = Word::saying(
    "capability.full-access.done-cannot-be-put-back",
    "Done without asking you first, because you chose full access in Settings → Running code: \
     {sentence}. This one can't be put back: a copy of your files from before it could not be \
     kept.",
)
.noting(
    "Like the sentence above, for a change the computer carried out without being able to keep \
     a copy of the person's files from before it, so Put back cannot undo it.",
);

/// What a person reads about a command alo ran at once, because they chose
/// full access, **that did not work** (ADR 0149). Never *Done*: a run that
/// ended with a failure is not a change that was made.
pub const DID_NOT_WORK_AT_FULL_ACCESS: Word = Word::saying(
    "capability.full-access.did-not-work",
    "Tried without asking you first, because you chose full access in Settings → Running code: \
     {sentence}. It did not work: {why}.",
)
.noting(
    "Shown under what alo did, for a command it ran without the person's OK, because the person \
     chose full access, when the command failed. {sentence} is what was run, for example 'run this \
     command: mkdir /home/anna/Documents/demo'. {why} is one of the clauses that say why, such as \
     'the command ended with exit status 1'. 'Settings → Running code' names a page as Settings \
     shows it; use the same words your translation uses there.",
);

/// The same, for a run the machine could not keep a copy from before.
pub const DID_NOT_WORK_AT_FULL_ACCESS_AND_IT_CANNOT_BE_PUT_BACK: Word = Word::saying(
    "capability.full-access.did-not-work-cannot-be-put-back",
    "Tried without asking you first, because you chose full access in Settings → Running code: \
     {sentence}. It did not work: {why}. If it changed anything, that can't be put back: a copy of \
     your files from before it could not be kept.",
)
.noting(
    "Like the sentence above, for a failed command the computer ran without being able to keep a \
     copy of the person's files from before it. A command can fail part of the way through, so the \
     person is told that whatever it did change cannot be undone with Put back.",
);

/// The person said yes in the terminal, the command ran, and it did not work.
pub const DID_NOT_WORK_AFTER_YOUR_OK: Word = Word::saying(
    "capability.terminal.did-not-work",
    "It did not work: {why}.",
)
.noting(
    "Printed in the terminal after the person answered yes to a command alo proposed, when the \
     command ran and failed. {why} is one of the clauses that say why, such as 'the command ended \
     with exit status 1'.",
);

/// The same, for a run the machine could not keep a copy from before.
pub const DID_NOT_WORK_AFTER_YOUR_OK_AND_IT_CANNOT_BE_PUT_BACK: Word = Word::saying(
    "capability.terminal.did-not-work-cannot-be-put-back",
    "It did not work: {why}. If it changed anything, that can't be put back: a copy of your files \
     from before it could not be kept.",
)
.noting(
    "Like the sentence above, when the computer could not keep a copy of the person's files from \
     before the command, so Put back cannot undo whatever it changed before it failed.",
);

/// Why a command did not work: it ended by itself with a failure.
pub const A_COMMAND_ENDED_WITH_A_STATUS: Word = Word::saying(
    "capability.running.ended-with-a-status",
    "the command ended with exit status {status}",
)
.noting(
    "Follows 'It did not work:' in a sentence about a command that ran. {status} is a number the \
     program gave as it ended, and any number but 0 means it failed; keep the number as it is. \
     'exit status' is the usual name for that number in a terminal.",
);

/// Why a command did not work: something stopped it before it ended by itself.
pub const A_COMMAND_WAS_STOPPED: Word = Word::saying(
    "capability.running.was-stopped",
    "the command was stopped before it finished",
)
.noting(
    "Follows 'It did not work:' for a command that something outside it ended, such as the \
     computer or another program, so it never said whether it succeeded.",
);

/// Why a command did not work: it ran longer than one is let run.
pub const A_COMMAND_RAN_TOO_LONG: Word = Word::saying(
    "capability.running.ran-too-long",
    "the command ran longer than a command may, and was stopped",
)
.noting(
    "Follows 'It did not work:' for a command the computer stopped because it had run for longer \
     than alo lets one command run.",
);

/// Why a command did not work: nobody is signed in to GitHub — and how to sign
/// in, and what that sign-in lets anything running as the person do (the
/// owner's decision of 2026-11-13: *say its broad access plainly, and say that
/// at full access alo can read a stored sign-in*).
pub const NOT_SIGNED_IN_TO_GITHUB: Word = Word::saying(
    "capability.running.not-signed-in-to-github-ask-again",
    "you are not signed in to GitHub. Sign in from Settings → Accounts, or type gh auth login \
     --web in your terminal and follow the steps GitHub shows you in your browser, and then ask \
     alo again: it does not try again by itself. That sign-in is kept on this computer, and it \
     lets anything you run — and alo, at full access — read and change every repository your \
     GitHub account can, and make new ones, until you type gh auth logout",
)
.noting(
    "Follows 'It did not work:' for a command that failed because the person has not signed in \
     to GitHub, the website where programmers keep their code. 'gh auth login --web' and 'gh auth \
     logout' are commands typed exactly as written and are never translated. 'Settings → \
     Accounts' and 'full access' (a setting in Settings → Running code) are named as your \
     translation names them there. 'ask alo again' matters: alo does not run the command a \
     second time by itself, so the person has to ask. No full stop at the end: the sentence \
     around it adds one.",
);

/// The question the terminal asks before a change alo proposed is carried out:
/// ADR 0124's own words, drawn in `docs/design/the-terminal-window.md`.
pub const RUN_THIS_COMMAND: Word = Word::saying(
    "capability.terminal.run-this-command",
    "Run this command? [y/N]",
)
.noting(
    "Asked in the terminal under the change alo proposes, before anything is done. Keep '[y/N]' \
     exactly as it is, with its letters untranslated: only the key y means yes, and every other \
     key, Enter among them, means no.",
);

/// The person said yes in the terminal, and it was done.
pub const DONE_AFTER_YOUR_OK: Word = Word::saying("capability.terminal.done", "Done.").noting(
    "Printed in the terminal after the person answered yes and the change was carried out.",
);

/// The same, for a change the machine could not keep a copy from before.
pub const DONE_AFTER_YOUR_OK_AND_IT_CANNOT_BE_PUT_BACK: Word = Word::saying(
    "capability.terminal.done-cannot-be-put-back",
    "Done. This one can't be put back: a copy of your files from before it could not be kept.",
)
.noting(
    "Printed after the person answered yes, for a change the computer carried out without being \
     able to keep a copy of the person's files from before it, so Put back cannot undo it.",
);

/// The person said no in the terminal.
pub const NOT_DONE_YOU_SAID_NO: Word = Word::saying(
    "capability.terminal.you-said-no",
    "Not done: you said no, and nothing was changed.",
)
.noting(
    "Printed in the terminal after the person answered anything but yes to alo's proposed \
     change. The no is kept in the record of what alo did.",
);

/// Nobody answered in the terminal while the question stood.
pub const NOT_DONE_NOBODY_ANSWERED: Word = Word::saying(
    "capability.terminal.nobody-answered",
    "Not done: there was no answer in time, and nothing was changed. Ask again when you are ready.",
)
.noting(
    "Printed in the terminal when nobody pressed a key before alo's question stopped standing, \
     or the terminal could no longer be read.",
);

/// The change the reply named is not one the service still holds.
pub const NOT_DONE_NO_LONGER_WAITING: Word = Word::saying(
    "capability.terminal.no-longer-waiting",
    "Not done: this change is no longer waiting, and nothing was changed. Ask again to have alo \
     propose it again.",
)
.noting(
    "Printed in the terminal when the change alo proposed had already stopped waiting by the \
     time it could be answered.",
);

/// A change was proposed for a request that did not come from a terminal, so
/// there is nobody to ask.
pub const IT_WAITS_FOR_A_TERMINAL: Word = Word::saying(
    "capability.terminal.waits-for-a-terminal",
    "Nothing was changed: alo asks for your OK before a change, and can only ask in a terminal. \
     Type the same request at a terminal prompt to answer it.",
)
.noting(
    "Printed when alo was asked to do something from a script or a pipe rather than typed at a \
     terminal, so the question 'Run this command?' could not be put to anybody.",
);

/// The service said nothing back about a change the terminal told it to go
/// ahead with.
pub const NO_WORD_BACK_IN_THE_TERMINAL: Word = Word::saying(
    "capability.terminal.no-word-back",
    "alo was told to go ahead, and the alo service did not say whether it did. See what alo did \
     to find out.",
)
.noting(
    "Printed in the terminal when alo was told to do something without asking and the service \
     that carries it out did not answer. 'See what alo did' is the record of everything alo did; \
     use the same words your translation uses for it.",
);

/// What the person approves when alo asks to run a command where no verb fits
/// (ADR 0149): the command, whole (`crate::ACommand::sentence`).
pub const RUN_A_COMMAND: Word = Word::saying(
    "capability.running.run-a-command",
    "run this command: {command}",
)
.noting(
    "The change alo proposes when nothing else it can do fits what was asked: running one \
     command on this computer, as the person. {command} is the command exactly as it will run, \
     and is never translated. Lower case at the start, like every other change alo proposes, \
     because it is shown after other words.",
);

/// A command alo was asked to run had nothing in it.
pub const A_COMMAND_THAT_IS_EMPTY: Word = Word::saying(
    "capability.running.command-empty",
    "Nothing ran: the command was empty.",
)
.noting("Said when alo was asked to run a command and the command had nothing in it.");

/// A command alo was asked to run was longer than it will run.
pub const A_COMMAND_THAT_IS_TOO_LONG: Word = Word::saying(
    "capability.running.command-too-long",
    "Nothing ran: the command was longer than {most} characters.",
)
.noting(
    "Said when alo was asked to run a command too long for a person to read before approving \
     it. {most} is a number.",
);

/// A command alo was asked to run was more than one line.
pub const A_COMMAND_THAT_IS_NOT_ONE_LINE: Word = Word::saying(
    "capability.running.command-not-one-line",
    "Nothing ran: a command must be one line.",
)
.noting(
    "Said when alo was asked to run a command that had a line break in it. A command is shown \
     to the person on one line before it runs, so a second line is refused.",
);

// ---------------------------------------------------------------------------
// A text argument, shown whole in the approval — [`crate::TheWholeText`].
//
// ADR 0001, amended 2026-11-13. Read by somebody about to approve a page of
// code or a post, with the whole text in front of them.
// ---------------------------------------------------------------------------

/// What fills a sentence's gap where a verb delivers text.
pub const THE_TEXT_BELOW: Word = Word::saying("capability.text.below", "the text below").noting(
    "Fills a gap in the sentence a person approves, such as \"type {text} into {application}\". \
     The text itself is shown in full underneath the sentence, so this points down to it.",
);

/// How long a text is, before the person starts reading it.
pub const TEXT_HOW_LONG: Word = Word::saying("capability.text.how-long", "{lines}, {characters}")
    .noting(
        "Shown above a text the person is asked to approve. {lines} is the number of lines \
         (\"12 lines\") and {characters} the number of characters (\"418 characters\"), each \
         already counted in your language. Order and punctuation are yours to choose.",
    );

/// A control character inside a text.
pub const TEXT_HOLDS_A_CONTROL: Word = Word::saying(
    "capability.text.holds-a-control",
    "control character {code}",
)
.noting(
    "Marks, inside a text, a character that is not shown but changes how text behaves — an \
         escape, a bell, a carriage return on its own. {code} is its Unicode code point, such as \
         U+001B, and is never translated.",
);

/// A character that changes the direction of a text.
pub const TEXT_HOLDS_A_DIRECTION: Word = Word::saying(
    "capability.text.holds-a-direction",
    "character that changes the direction of the text {code}",
)
.noting(
    "Marks, inside a text, an invisible character that makes the text after it appear in a \
     different order from the one it is in, so what a person reads is not what is delivered. \
     {code} is its Unicode code point, such as U+202E, and is never translated.",
);

/// A character that draws nothing.
pub const TEXT_HOLDS_AN_INVISIBLE: Word = Word::saying(
    "capability.text.holds-an-invisible",
    "invisible character {code}",
)
.noting(
    "Marks, inside a text, a character that takes up no space and draws nothing, such as a \
         zero-width space. {code} is its Unicode code point, such as U+200B, and is never \
         translated.",
);

/// A space that is not an ordinary space.
pub const TEXT_HOLDS_A_SPACE: Word =
    Word::saying("capability.text.holds-a-space", "unusual space {code}").noting(
        "Marks, inside a text, a space or line break that looks like an ordinary one and is \
         not, such as a no-break space. {code} is its Unicode code point, such as U+00A0, and is \
         never translated.",
    );

/// A letter from another alphabet in the middle of a word.
pub const TEXT_HOLDS_ANOTHER_ALPHABET: Word = Word::saying(
    "capability.text.holds-another-alphabet",
    "“{letter}” from another alphabet {code}",
)
.noting(
    "Marks, inside a word, a letter from a different alphabet that may look like a letter of \
     the word's own — a Cyrillic а in a Latin word. {letter} is the letter itself and {code} its \
     Unicode code point; neither is translated. Use your language's own quotation marks.",
);

/// Every string this crate can say, in the order a translator meets them.
pub const EVERY_WORD: [Word; 110] = [
    ANONYMOUS,
    NO_APPLICATION_NAMED,
    NOTHING_NAMED,
    THE_WHOLE_MACHINE,
    GRANT_NOT_A_FULL_PATH,
    GRANT_COULD_LEAD_ELSEWHERE,
    GRANT_NO_TIME,
    GRANT_NO_END,
    NOT_FOR_AN_AGENT,
    PERSONS_OWN,
    A_FOLDER,
    A_FILE,
    AN_APPLICATION,
    THE_CAMERA,
    THE_MICROPHONE,
    THE_SCREEN_ONCE,
    THE_SCREEN_CONTINUOUSLY,
    NOTIFICATIONS,
    THE_CLIPBOARD,
    THE_DESKTOP_BACKGROUND,
    APPEARANCE_SETTINGS,
    SLEEP,
    THE_NETWORK_STATE,
    THE_POWER_PROFILE,
    ITS_OWN_SECRETS,
    HAS_EXPIRED,
    NEVER_GRANTED,
    APPLICATION_NOT_GIVEN,
    NO_AGENT,
    APPLICATION_LAPSED,
    APPLICATION_NEVER_GRANTED,
    CHANGE_WAITS,
    WANTED_NUMBER,
    WANTED_TEXT,
    ARGUMENT_EMPTY,
    ARGUMENT_NOT_A_FULL_PATH,
    ARGUMENT_COULD_LEAD_ELSEWHERE,
    NOT_ONE_NAME,
    NOT_AN_IDENTIFIER,
    NOT_PRINTABLE,
    OUT_OF_RANGE,
    NOT_ON_THE_LIST,
    NO_SUCH_VERB,
    NO_SUCH_ARGUMENT,
    ARGUMENT_MISSING,
    SAME_ARGUMENT_TWICE,
    READ_DOES_NOT_WAIT,
    PROPOSAL_NO_TIME,
    PROPOSAL_NO_END,
    NOTHING_WAITING,
    LAPSED,
    HAS_AN_AGENT,
    HAS_NO_AGENT,
    RUNS_IN_A_SEALED_BOX,
    RUNS_AFTER_ASKING,
    RUNS_AT_FULL_TRUST,
    NOTHING_RAN_NOT_A_COMMAND,
    NOTHING_RAN_NO_BOX,
    NOTHING_RAN_WOULD_NOT_START,
    THE_SEALED_BOX_CANNOT_RUN_PROGRAMS_HERE_YET,
    THE_SEALED_BOX_IS_NOT_HERE_YET,
    NOTHING_RAN_NOT_THAT_WAY,
    WHAT_FULL_TRUST_COSTS,
    ALWAYS_THE_RECORD,
    ALWAYS_THE_SNAPSHOT,
    ALWAYS_THE_INDICATOR,
    WHAT_ACTING_WITHOUT_ASKING_COSTS,
    WORKS_IN_ITS_OWN_COPY,
    WORKS_IN_ITS_OWN_COPY_IS,
    MAY_WORK_IN_YOUR_WINDOW,
    MAY_WORK_IN_YOUR_WINDOW_IS,
    WHAT_WORKING_IN_YOUR_WINDOW_COSTS,
    NO_COPY_OF_ITS_OWN,
    DONE_WITHOUT_ASKING,
    DONE_WITHOUT_ASKING_AND_IT_CANNOT_BE_PUT_BACK,
    DONE_AT_FULL_ACCESS,
    DONE_AT_FULL_ACCESS_AND_IT_CANNOT_BE_PUT_BACK,
    DID_NOT_WORK_AT_FULL_ACCESS,
    DID_NOT_WORK_AT_FULL_ACCESS_AND_IT_CANNOT_BE_PUT_BACK,
    DID_NOT_WORK_AFTER_YOUR_OK,
    DID_NOT_WORK_AFTER_YOUR_OK_AND_IT_CANNOT_BE_PUT_BACK,
    A_COMMAND_ENDED_WITH_A_STATUS,
    A_COMMAND_WAS_STOPPED,
    A_COMMAND_RAN_TOO_LONG,
    NOT_SIGNED_IN_TO_GITHUB,
    RUN_THIS_COMMAND,
    DONE_AFTER_YOUR_OK,
    DONE_AFTER_YOUR_OK_AND_IT_CANNOT_BE_PUT_BACK,
    NOT_DONE_YOU_SAID_NO,
    NOT_DONE_NOBODY_ANSWERED,
    NOT_DONE_NO_LONGER_WAITING,
    IT_WAITS_FOR_A_TERMINAL,
    NO_WORD_BACK_IN_THE_TERMINAL,
    RUN_A_COMMAND,
    A_COMMAND_THAT_IS_EMPTY,
    A_COMMAND_THAT_IS_TOO_LONG,
    A_COMMAND_THAT_IS_NOT_ONE_LINE,
    NOTHING_RAN_THEY_CANCELLED,
    NOTHING_RAN_NOT_ACCEPTED,
    NOTHING_RAN_NOTHING_CAN_ASK,
    NOTHING_RAN_NOT_AN_ADMINISTRATOR,
    NOTHING_RAN_NO_ANSWER_IN_TIME,
    NOTHING_RAN_AS_ADMINISTRATOR_COULD_NOT_ASK,
    THE_TEXT_BELOW,
    TEXT_HOW_LONG,
    TEXT_HOLDS_A_CONTROL,
    TEXT_HOLDS_A_DIRECTION,
    TEXT_HOLDS_AN_INVISIBLE,
    TEXT_HOLDS_A_SPACE,
    TEXT_HOLDS_ANOTHER_ALPHABET,
];

/// How many lines a text holds.
pub const TEXT_LINES: Counted = Counted {
    named: "capability.text.lines",
    number: "lines",
    one: "one line",
    other: "{lines} lines",
    note: "How many lines a text the person is asked to approve holds. Goes into \
           capability.text.how-long beside the number of characters.",
};

/// How many characters a text holds.
pub const TEXT_CHARACTERS: Counted = Counted {
    named: "capability.text.characters",
    number: "characters",
    one: "one character",
    other: "{characters} characters",
    note: "How many characters a text the person is asked to approve holds. Goes into \
           capability.text.how-long beside the number of lines.",
};

/// How many characters in a text are marked as easy to miss.
pub const TEXT_UNSEEN: Counted = Counted {
    named: "capability.text.unseen",
    number: "unseen",
    one: "One character in this text is easy to miss, and it is marked where it is.",
    other: "{unseen} characters in this text are easy to miss, and each is marked where it is.",
    note: "Shown above a text the person is asked to approve when it holds characters that are \
           invisible or look like other characters. Nothing was removed or refused: each such \
           character is marked inside the text, and this sentence says how many there are.",
};

/// Every countable string this crate can say, beside [`EVERY_WORD`].
pub const EVERY_COUNTED: [Counted; 4] = [TOO_LONG, TEXT_LINES, TEXT_CHARACTERS, TEXT_UNSEEN];

/// Why this crate's own list could not be declared.
///
/// Not a refusal a person reads — it keeps its English and its `Display` for
/// the reason [`crate::VerbError`] does. It exists because [`declare_into`] can
/// genuinely fail against a vocabulary that already holds one of these keys.
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum WordsError {
    /// A word that is not a phrase: a sentence that is not one, or a note that
    /// could not be attached.
    #[error(transparent)]
    Word(#[from] alo_strings::WordError),
    /// A countable string that could not be declared.
    #[error(transparent)]
    Counting(#[from] alo_strings::PluralError),
    /// A key the vocabulary already has.
    #[error(transparent)]
    List(#[from] alo_strings::VocabularyError),
}

/// Everything this crate can say, as a vocabulary of its own.
///
/// # Errors
/// [`WordsError`], which the list above cannot cause.
pub fn capability_words() -> Result<Vocabulary, WordsError> {
    let mut vocabulary = Vocabulary::empty();
    declare_into(&mut vocabulary)?;
    Ok(vocabulary)
}

/// Put everything this crate can say into an existing vocabulary.
///
/// The shell has one vocabulary and every crate adds its own to it, which is
/// what the area at the front of a key is for.
///
/// # Errors
/// [`WordsError::List`] if the vocabulary already holds one of these keys —
/// nothing is replaced, because a key means one string and whoever declared it
/// first said what that string is.
pub fn declare_into(vocabulary: &mut Vocabulary) -> Result<(), WordsError> {
    for word in EVERY_WORD {
        vocabulary.says(word.phrase()?)?;
    }
    for counted in EVERY_COUNTED {
        vocabulary.counts(
            Plural::counting(counted.key(), counted.number, counted.one, counted.other)?
                .noting(counted.note)?,
        )?;
    }
    Ok(())
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;

    /// **What we ship is held to the rule everybody else is held to.**
    /// [`Word::key`] does not check, because a key written in this file cannot
    /// arrive from anywhere; this is the test that makes that true, and it is
    /// the same shape as `alo-shortcuts` putting every shipped binding back
    /// through `Chord::checked`.
    #[test]
    fn every_key_is_a_key() {
        for word in EVERY_WORD {
            assert_eq!(
                Key::named(word.named()),
                Ok(word.key()),
                "{}: {}",
                word.named(),
                Key::named(word.named()).unwrap_err()
            );
        }
        for counted in EVERY_COUNTED {
            assert_eq!(Key::named(counted.named), Ok(counted.key()));
        }
    }

    /// A key names one string. Two words sharing one would mean whichever was
    /// declared second is a string nobody can reach.
    #[test]
    fn no_two_words_are_named_the_same() {
        let named: BTreeSet<&str> = EVERY_WORD.iter().map(|word| word.named()).collect();
        assert_eq!(named.len(), EVERY_WORD.len());
        let counted: BTreeSet<&str> = EVERY_COUNTED.iter().map(|counted| counted.named).collect();
        assert_eq!(counted.len(), EVERY_COUNTED.len());
        assert!(named.is_disjoint(&counted));
    }

    /// Every one of them is in the area a reader can sort by, which is what
    /// lets one vocabulary hold every crate's strings.
    #[test]
    fn everything_this_crate_says_says_it_is_this_crate() {
        for word in EVERY_WORD {
            assert_eq!(word.key().area(), "capability", "{}", word.named());
        }
        for counted in EVERY_COUNTED {
            assert_eq!(counted.key().area(), "capability", "{}", counted.named);
        }
    }

    /// The list declares, and nothing about it is refused by the crate that
    /// receives it — which is the whole of what this file has to get right.
    #[test]
    fn the_whole_list_declares() {
        let vocabulary = capability_words().unwrap();
        assert_eq!(
            vocabulary.how_many(),
            EVERY_WORD.len() + EVERY_COUNTED.len()
        );
        assert_eq!(vocabulary.counted().count(), EVERY_COUNTED.len());
    }

    /// A vocabulary that already holds one of these keeps its own, and nothing
    /// is quietly replaced.
    #[test]
    fn a_key_already_taken_is_not_replaced() {
        let mut vocabulary = capability_words().unwrap();
        let again = declare_into(&mut vocabulary).unwrap_err();
        assert!(matches!(again, WordsError::List(_)), "{again}");
    }

    /// **No sentence an application reads calls it an agent** (ADR 0040).
    #[test]
    fn what_an_application_is_told_never_calls_it_an_agent() {
        for word in [
            NO_APPLICATION_NAMED,
            APPLICATION_LAPSED,
            APPLICATION_NEVER_GRANTED,
        ] {
            assert!(!word.says().contains("agent"), "{}", word.named());
            assert!(!word.says().contains("folder"), "{}", word.named());
        }
    }

    /// The sentences that carry something off the machine — a name, an
    /// identifier, a number, a list of options — say so, because a translator
    /// with no product in front of them cannot tell a gap that is a word from
    /// a gap that is a value.
    #[test]
    fn the_ones_a_translator_cannot_work_out_carry_a_note() {
        for word in [
            A_FOLDER,
            AN_APPLICATION,
            HAS_EXPIRED,
            NEVER_GRANTED,
            NO_AGENT,
            NOT_FOR_AN_AGENT,
            PERSONS_OWN,
            APPLICATION_LAPSED,
            APPLICATION_NEVER_GRANTED,
            CHANGE_WAITS,
            NOT_ON_THE_LIST,
            NO_SUCH_VERB,
            ARGUMENT_MISSING,
            LAPSED,
            HAS_AN_AGENT,
            HAS_NO_AGENT,
            THE_TEXT_BELOW,
            TEXT_HOW_LONG,
            TEXT_HOLDS_A_CONTROL,
            TEXT_HOLDS_A_DIRECTION,
            TEXT_HOLDS_AN_INVISIBLE,
            TEXT_HOLDS_A_SPACE,
            TEXT_HOLDS_ANOTHER_ALPHABET,
        ] {
            assert!(word.note().is_some(), "{}", word.named());
        }
        for counted in EVERY_COUNTED {
            assert!(!counted.note.is_empty(), "{}", counted.named);
        }
    }
}
