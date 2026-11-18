//! Every string this crate can say, and the English beside each one.
//!
//! Four groups: what became of an installation, an update or a removal; why one
//! was refused; the update a person is offered; and the one verb an agent
//! proposes an installation through. Every refusal says **what did not happen
//! and what a person does about it**, because *nothing was installed* on its
//! own leaves somebody holding a question.
//!
//! *A fifth group, what opens a web address, was retired with
//! `WhatOpensWebAddresses`: which application opens a web link is said by
//! `alo_applications` and the open-with portal, from the person's `[links]`
//! choice and what the desktop entries declare.*
//!
//! # Nothing a person reads names the machinery
//!
//! Not the tool that installs, not the service an application arrives from by
//! its tooling, not a repository, a remote, a key, a branch or a sandbox. A
//! person installs an application — `docs/features.md` says so in those words
//! — and a test at the bottom of this file reads every sentence and every note
//! for the words that would undo it. The name of a place applications come from
//! is data, filled into `{source}` exactly as this machine knows it, and never
//! written into a sentence.

use alo_strings::{Vocabulary, VocabularyError, WordError};

/// One string a crate can say — `alo-strings`' type, re-exported so this
/// crate's files name it as `crate::words::Word`.
pub use alo_strings::Word;

/// The gap holding an application's identifier.
pub const APPLICATION: &str = "application";

/// The gap holding the name of a place applications come from.
pub const SOURCE: &str = "source";

/// The gap holding the identifier of the catalogue's application, where one
/// was asked for by an identifier the catalogue does not hold.
pub const INSTEAD: &str = "instead";

/// The gap holding the kind of work a person is looking under.
///
/// **Filled with the word a person reads, not with the identifier.**
/// `alo_capability::Takes::Choice` holds `Offered`s — a name a model sends and
/// a word a person reads — and `crate::Category`'s own header is blunt about
/// which goes where: *a market screen may not draw it … a screen that drew
/// `page-layout` at somebody would be the hardcoded-English bug wearing a data
/// file.* The same holds for a sentence in a record.
pub const KIND_OF_WORK: &str = "kind_of_work";

// ---------------------------------------------------------------------------
// What became of it.
// ---------------------------------------------------------------------------

/// An application was installed.
pub const INSTALLED: Word = Word::saying(
    "software.installed",
    "{application} is installed. It has been given nothing: it asks when it needs a file, the \
     camera or anything else, and you answer",
)
.noting(
    "Said once an application a person chose has arrived on this machine. {application} is its \
     identifier, like org.gnome.TextEditor, and is not translated. The second sentence is the \
     promise that matters: a new application can reach nothing until the person allows it \
     something.",
);

/// An application a person chose to install unconfined, said as what it costs.
///
/// **The key does not name the machinery either.** This crate's test reads the sentence and
/// the translator's note; a key is read by a translator too, and it is what a person sees
/// when a translation is missing. So the key says what this application can do, the same
/// as the sentence does, while the Rust name beside it is free to say `unsandboxed`
/// because no person reads it.
pub const INSTALLED_UNSANDBOXED: Word = Word::saying(
    "software.installed-reaching-everything",
    "{application} is installed, and you chose to let it reach your files and the network \
     without asking you first. Anything else you install has to ask; this one does not",
)
.noting(
    "Said once an application a person deliberately chose to install without the usual \
     confinement has arrived on this machine. {application} is its identifier, like \
     org.gnome.TextEditor, and is not translated. This sentence replaces the ordinary installed \
     sentence rather than being added to it: that one promises the application has been given \
     nothing, and the promise is not true here. Say what the person gave up and what is \
     different about this one, never the name of the machinery that would otherwise have held \
     it, and not that it is dangerous — the person chose this and these words are not a warning \
     to dismiss.",
);
/// An update is ready and waits for the person.
pub const UPDATE_OFFERED: Word = Word::saying(
    "software.update-offered",
    "A newer version of {application} is ready. It is updated when you choose, and never while it \
     is open",
)
.noting(
    "Shown after this machine found a newer version of an application. Nothing has changed yet: \
     the person chooses when, and an application that is open is never updated underneath the \
     person using it. {application} is the application's identifier and is not translated.",
);

/// An application was updated.
pub const UPDATED: Word = Word::saying(
    "software.updated",
    "{application} is updated. Nothing else was closed or restarted",
)
.noting(
    "Said once a newer version of an application is in place. \"Nothing else\" means no other \
     application and not the computer itself: an application's update never restarts the machine. \
     {application} is the application's identifier.",
);

/// An application was removed, and its grants with it.
pub const REMOVED: Word = Word::saying(
    "software.removed",
    "{application} is removed. Everything it was allowed has ended, and so has every permission \
     to use it",
)
.noting(
    "Said once an application has been taken off this machine. Everything the person had allowed \
     it — a folder, the camera — ended in the same moment, and so did every permission to use it, \
     such as an assistant's permission to open documents in it. Installing it again later starts \
     with nothing allowed either way. {application} is the application's identifier.",
);

// ---------------------------------------------------------------------------
// Why it was refused.
// ---------------------------------------------------------------------------

/// The place named is not one this machine installs from.
pub const NOT_ENABLED: Word = Word::saying(
    "software.refused.not-enabled",
    "{source} is not a place this machine installs applications from, so nothing was installed. \
     Choose one of the places listed under Software in Settings",
)
.noting(
    "Said when an application was asked for from a place that is not set up on this machine. \
     {source} is the name of that place exactly as it was given, and is not translated. Software \
     and Settings are the names of places in this machine's own interface and should be the words \
     used there.",
);

/// An organisation does not permit this place.
pub const OUTSIDE_THE_BOUND_BY_AN_ADMINISTRATOR: Word = Word::saying(
    "software.refused.outside-the-bound.administrator",
    "The organisation that manages this machine does not permit installing applications from \
     {source}, so nothing was installed or updated from it. Someone who looks after this machine \
     can say which places are permitted",
)
.noting(
    "Said on a machine an organisation manages, when a place applications come from is set up but \
     the organisation's rule does not include it. The person is told who made the rule, because a \
     refusal nobody can account for is worse than none. {source} is the name of the place.",
);

/// This machine's own description does not permit this place.
pub const OUTSIDE_THE_BOUND_BY_THIS_PERSON: Word = Word::saying(
    "software.refused.outside-the-bound.this-person",
    "This machine's own settings do not permit installing applications from {source}, so nothing \
     was installed or updated from it. No organisation set this: it was set on this machine, and \
     can be changed there",
)
.noting(
    "Said on a machine nobody else manages, when the machine's owner limited where applications may \
     come from and this place is not included. It says plainly that no organisation is involved. \
     {source} is the name of the place.",
);

/// The place is set up without proving what it sends is its own.
pub const NOT_VERIFIED: Word = Word::saying(
    "software.refused.not-verified",
    "{source} is set up without checking that what it sends really comes from it, so this machine \
     does not install or update anything from it. Nothing was installed",
)
.noting(
    "Said when a place applications come from was set up with its signature checking turned off. \
     This machine refuses such a place rather than trusting it. {source} is the name of the place.",
);

/// What arrived could not be shown to come from the place.
pub const SIGNATURE_NOT_SHOWN: Word = Word::saying(
    "software.refused.signature-not-shown",
    "What {source} sent for {application} could not be shown to come from {source}, so nothing was \
     installed and nothing it sent was kept. Try again later, or ask whoever runs {source}",
)
.noting(
    "Said when an application or its update arrived without a valid signature from the place it \
     should have come from — it may have been damaged or replaced on the way. {source} is the name \
     of the place and {application} the application's identifier; neither is translated.",
);

/// The place does not offer this application.
pub const NOT_OFFERED: Word = Word::saying(
    "software.refused.not-offered",
    "{source} does not offer {application}, so nothing was installed. Check the application's \
     name, or choose another place it comes from",
)
.noting(
    "Said when the place named has no application by that identifier. {application} is the \
     identifier exactly as it was given, and {source} the name of the place.",
);

/// An identifier the catalogue does not hold, where it holds one application
/// the identifier's last part answers to.
pub const NOT_HELD_BUT_THIS_IS: Word = Word::saying(
    "software.refused.not-held-but-this-is",
    "{application} is not in what alo knows {source} offers, but {instead} is, by that name. Name \
     that one if it is the one meant",
)
.noting(
    "Said when an application to install was named by an identifier the catalogue does not \
     hold, and one application the catalogue holds answers to its last part. {application} is \
     the identifier exactly as it was given, {instead} the catalogue's, and {source} the name of \
     the place. None of them is translated.",
);

/// The application is already on this machine.
pub const ALREADY_INSTALLED: Word = Word::saying(
    "software.refused.already-installed",
    "{application} is already installed on this machine, so it was not installed again. Updates \
     for it are offered in Software",
)
.noting(
    "Said when a person or an assistant asked to install an application that is already here. \
     {application} is its identifier. Software is the name of a place in this machine's own \
     interface.",
);

/// The application is not on this machine.
pub const NOT_INSTALLED: Word = Word::saying(
    "software.refused.not-installed",
    "{application} is not installed on this machine, so there is nothing to update or remove. \
     Check the application's name",
)
.noting(
    "Said when an update or a removal named an application that is not here. {application} is the \
     identifier exactly as it was given.",
);

/// A name more than one application in what can be installed answers to.
pub const NAMES_MORE_THAN_ONE: Word = Word::saying(
    "software.refused.names-more-than-one",
    "“{application}” could be more than one application that can be installed, so alo did not \
     guess which and nothing was proposed. Name the one you mean by its identifier",
)
.noting(
    "Said when a model named an application to install by a name several in the catalogue \
     answer to. {application} is the name as it was given, and is not translated.",
);

/// The application is open, so its update waits.
pub const STILL_OPEN: Word = Word::saying(
    "software.refused.still-open",
    "{application} is open, so it was not updated. Close it and choose to update it again: an \
     update never closes an application by itself",
)
.noting(
    "Said when a person chose an update for an application they are still using. Nothing was \
     closed and nothing was lost; the update simply waits. {application} is the application's \
     identifier.",
);

/// The place has no address this machine can reach.
pub const NOWHERE_TO_REACH: Word = Word::saying(
    "software.refused.nowhere-to-reach",
    "{source} is not set up with an address on the network this machine can reach, so nothing was \
     installed or updated from it. Whoever set it up can give it one",
)
.noting(
    "Said when a place applications come from is set up in a way this machine cannot use — for \
     example with no address, or one written in a form it does not read. {source} is the name of \
     the place.",
);

/// The part of the machine that installs did not respond.
pub const DID_NOT_ANSWER: Word = Word::saying(
    "software.refused.did-not-answer",
    "The part of this machine that installs applications did not respond, so nothing was changed. \
     Restarting this machine starts it again",
)
.noting(
    "Said when installing, updating or removing could not begin or did not finish because the \
     machine's own installing component failed. \"The part of this machine that installs \
     applications\" is deliberate: the person never needs to know what it is called.",
);

// ---------------------------------------------------------------------------
// The verb an agent is shown the catalogue through.
// ---------------------------------------------------------------------------

/// What the read is for.
pub const READ_PURPOSE: Word = Word::saying(
    "software.verb.what-can-be-installed.purpose",
    "see what can be installed on this machine, and what kind of work each one is for",
)
.noting(
    "What an assistant's ability to look at the catalogue is for, shown in a list of what it can \
     do. It is the list this machine ships and is the same on every machine: it says nothing \
     about what this person has installed, and there is no way for an assistant to ask that.",
);

/// The argument naming the kind of work.
pub const READ_KIND_OF_WORK: Word = Word::saying(
    "software.verb.what-can-be-installed.kind-of-work",
    "the kind of work to look under, or everything",
)
.noting(
    "Describes the one part of this request: which shelf of the catalogue to look at. The choices \
     are the kinds of work a person can make something with, plus everything, and nothing else is \
     accepted.",
);

/// The sentence the record keeps.
pub const READ_SENTENCE: Word = Word::saying(
    "software.verb.what-can-be-installed.sentence",
    "see what can be installed under {kind_of_work}",
)
.noting(
    "What the record says an assistant did when it looked at the catalogue. {kind_of_work} is one \
     of the kinds of work, or the word for everything, and IS translated — unlike an \
     application's identifier. A person does not approve this sentence: looking at the catalogue \
     changes nothing and answers inside the turn.",
);

// ---------------------------------------------------------------------------
// The verb an agent proposes an installation through.
// ---------------------------------------------------------------------------

/// What the verb is for.
pub const VERB_PURPOSE: Word = Word::saying(
    "software.verb.install-application.purpose",
    "install an application from a place this machine installs applications from",
)
.noting(
    "What an assistant's ability to propose installing an application is for, shown in a list of \
     what it can do. It only ever proposes: nothing is installed until the person approves.",
);

/// The argument naming the application.
pub const VERB_APPLICATION: Word = Word::saying(
    "software.verb.install-application.application",
    "the application to install, by its identifier",
)
.noting(
    "Describes one part of the proposal: which application. An identifier is the exact name this \
     machine knows an application by, like org.gnome.TextEditor.",
);

/// The argument naming the place.
pub const VERB_SOURCE: Word = Word::saying(
    "software.verb.install-application.source",
    "the place to install it from, by the name this machine gives that place",
)
.noting(
    "Describes one part of the proposal: where the application comes from. It is one of the places \
     listed under Software in Settings.",
);

/// The sentence a person approves.
pub const VERB_SENTENCE: Word = Word::saying(
    "software.verb.install-application.sentence",
    "install {application} from {source}",
)
.noting(
    "The sentence a person approves before an assistant's proposal to install an application is \
     carried out. {application} is the application's identifier and {source} the name of the place \
     it comes from; neither is translated. Approving it installs once, and the application arrives \
     allowed nothing.",
);

/// Every word that names something, read inside another sentence.
pub const THE_NAMES: [Word; 5] = [
    VERB_PURPOSE,
    VERB_APPLICATION,
    VERB_SOURCE,
    READ_PURPOSE,
    READ_KIND_OF_WORK,
];

/// Every word that is a line or a sentence of its own.
pub const THE_SENTENCES: [Word; 71] = [
    READ_SENTENCE,
    INSTALLED,
    INSTALLED_UNSANDBOXED,
    UPDATE_OFFERED,
    UPDATED,
    REMOVED,
    NOT_ENABLED,
    OUTSIDE_THE_BOUND_BY_AN_ADMINISTRATOR,
    OUTSIDE_THE_BOUND_BY_THIS_PERSON,
    NOT_VERIFIED,
    SIGNATURE_NOT_SHOWN,
    NOT_OFFERED,
    NOT_HELD_BUT_THIS_IS,
    NAMES_MORE_THAN_ONE,
    ALREADY_INSTALLED,
    NOT_INSTALLED,
    STILL_OPEN,
    NOWHERE_TO_REACH,
    DID_NOT_ANSWER,
    VERB_SENTENCE,
    VERB_CHANGES,
    VERB_LEAVES,
    THE_APPLICATIONS,
    EACH_SAYS_WHAT_IT_REACHES,
    RUNS_PROGRAMS_OUTSIDE_ITS_BOX,
    RUNS_PROGRAMS_OUTSIDE_ITS_BOX_IN_FULL,
    REACHES_YOUR_FILES,
    REACHES_YOUR_FILES_IN_FULL,
    INSTALLED_WITH_WHAT_ITS_MAKERS_ASK,
    VERB_LEAVES_REACHING_YOUR_FILES,
    VERB_LEAVES_RUNNING_PROGRAMS_OUTSIDE,
    VERB_LEAVES_USING_THE_INTERNET,
    VERB_LEAVES_REACHING_YOUR_FILES_AND_THE_INTERNET,
    VERB_LEAVES_RUNNING_PROGRAMS_OUTSIDE_AND_YOUR_FILES,
    VERB_LEAVES_RUNNING_PROGRAMS_OUTSIDE_AND_THE_INTERNET,
    VERB_LEAVES_RUNNING_PROGRAMS_OUTSIDE_YOUR_FILES_AND_THE_INTERNET,
    INSTALL_IT,
    INSTALLING_IT,
    OPEN_IT,
    EVERYTHING,
    PICTURES,
    PAINTING,
    ILLUSTRATION,
    PHOTOGRAPHS,
    PAGE_LAYOUT,
    VIDEO,
    AUDIO,
    DOCUMENTS,
    DIAGRAMS,
    MODELLING,
    MUSIC_MAKING,
    SCREEN_RECORDING,
    CODE,
    CHAT,
    CALLS,
    EMAIL,
    NOTES,
    MUSIC,
    BOOKS,
    MAPS,
    GAMES,
    PASSWORDS,
    REMOTE_DESKTOP,
    NOT_OPEN_SOURCE,
    NOT_OPEN_SOURCE_IN_FULL,
    NOT_VERIFIED_BY_ITS_MAKER,
    NOT_VERIFIED_BY_ITS_MAKER_IN_FULL,
    EARLY_VERSION,
    EARLY_VERSION_IN_FULL,
    NO_AUTOMATIC_UPDATES,
    NO_AUTOMATIC_UPDATES_IN_FULL,
];

/// Every refusal, so that a test can hold each of them to saying what did not
/// happen **and** what a person does about it.
pub const THE_REFUSALS: [Word; 13] = [
    NOT_ENABLED,
    OUTSIDE_THE_BOUND_BY_AN_ADMINISTRATOR,
    OUTSIDE_THE_BOUND_BY_THIS_PERSON,
    NOT_VERIFIED,
    SIGNATURE_NOT_SHOWN,
    NOT_OFFERED,
    NOT_HELD_BUT_THIS_IS,
    NAMES_MORE_THAN_ONE,
    ALREADY_INSTALLED,
    NOT_INSTALLED,
    STILL_OPEN,
    NOWHERE_TO_REACH,
    DID_NOT_ANSWER,
];

// ---------------------------------------------------------------------------
// The window of what a person can install.
// ---------------------------------------------------------------------------

/// What the window of what a person can install is called.
pub const THE_APPLICATIONS: Word = Word::saying("software.the-applications", "Applications")
    .noting(
        "The name of the window a person opens to find something to install. One word, and it \
         names what the window holds rather than what it is for.",
    );

/// The line under the list.
///
/// **Replaced *each one … arrives able to reach nothing* on 2026-11-17**,
/// which was measured false for every application in the catalogue: each
/// arrives with what its makers ask for, and nothing on this machine overrides
/// that. This says what is true of all of them and leaves the rest to each row.
pub const EACH_SAYS_WHAT_IT_REACHES: Word = Word::saying(
    "software.each-says-what-it-reaches",
    "Each one installs from the place it is published, with what its makers ask for. Its row says if that reaches your files or runs programs outside its box",
)
.noting(
    "Stands under the list of applications a person can install. Each application arrives with the access its makers ask for, which can include the network; the row of each one says when it can also reach the person's files without asking, or start programs outside the box applications are kept in. 'Its box' is the closed space an application runs in.",
);

/// Beside an application whose makers ask for it to start programs outside its
/// box.
pub const RUNS_PROGRAMS_OUTSIDE_ITS_BOX: Word = Word::saying(
    "software.label.runs-programs-outside-its-box",
    "Runs programs outside its box",
)
.noting(
    "Shown beside an application in the list a person installs from, when its makers ask for it to start programs on this computer outside the closed box applications are kept in. Such an application can do anything the person can. A plain fact, not a warning to dismiss: the person may install it. Short, because it sits on one line under the application's name.",
);

/// The same, in full.
pub const RUNS_PROGRAMS_OUTSIDE_ITS_BOX_IN_FULL: Word = Word::saying(
    "software.label.runs-programs-outside-its-box.in-full",
    "Runs programs outside its box: its makers ask for it to start programs on this computer outside the box applications are kept in, so it can do anything you can",
)
.noting(
    "What a screen reader says for the short label Runs programs outside its box, and what a person reads if they ask what it means.",
);

/// Beside an application whose makers ask for it to open the person's files.
pub const REACHES_YOUR_FILES: Word =
    Word::saying("software.label.reaches-your-files", "Reaches your files").noting(
        "Shown beside an application in the list a person installs from, when its makers ask for it to read and change the person's files without asking them first. A plain fact, not a warning to dismiss. Short, because it sits on one line under the application's name.",
    );

/// The same, in full.
pub const REACHES_YOUR_FILES_IN_FULL: Word = Word::saying(
    "software.label.reaches-your-files.in-full",
    "Reaches your files: its makers ask for it to read and change your files without asking you first",
)
.noting(
    "What a screen reader says for the short label Reaches your files, and what a person reads if they ask what it means.",
);

/// An application installed with access past its box, said once it arrives.
pub const INSTALLED_WITH_WHAT_ITS_MAKERS_ASK: Word = Word::saying(
    "software.installed-with-what-its-makers-ask",
    "{application} is installed, with what its makers ask for: it can reach more than its box without asking you first. Applications says what",
)
.noting(
    "Said once an application arrives whose makers ask for it to reach the person's files or start programs outside the closed box applications are kept in. It replaces the sentence that says an application has been given nothing, which is not true of this one. {application} is its identifier and is not translated; Applications is the name of the window that lists what can be installed.",
);

/// What installing leaves alone, for an application whose makers ask for it to
/// reach the person's files.
pub const VERB_LEAVES_REACHING_YOUR_FILES: Word = Word::saying(
    "software.verb.install-application.leaves.reaching-your-files",
    "Its makers ask for it to read and change your files without asking you first, and it will be able to. No other application is updated or removed.",
)
.noting(
    "Shown on the card a person answers before alo installs an application, in place of the line that says it is given no access to their files, when its makers ask for exactly that. Said before the person approves, so they choose knowing it.",
);

/// What installing leaves alone, for an application whose makers ask for it to
/// start programs outside its box.
pub const VERB_LEAVES_RUNNING_PROGRAMS_OUTSIDE: Word = Word::saying(
    "software.verb.install-application.leaves.running-programs-outside",
    "Its makers ask for it to run programs outside its box, so it will be able to do anything you can on this computer. No other application is updated or removed.",
)
.noting(
    "Shown on the card a person answers before alo installs an application, in place of the line that says it is given no access to their files, when its makers ask for it to start programs outside the closed box applications are kept in. Said before the person approves, so they choose knowing it.",
);

/// What installing leaves alone, for an application whose makers ask for it to
/// use the internet.
pub const VERB_LEAVES_USING_THE_INTERNET: Word = Word::saying(
    "software.verb.install-application.leaves.using-the-internet",
    "It is given no access to your files. Its makers ask for it to use the internet, and it will be able to. No other application is updated or removed.",
)
.noting(
    "Shown on the card a person answers before alo installs an application, in place of the line that says it is given no access to their files. It names everything the application's makers ask for that reaches past the closed box applications are kept in, said before the person approves, so they choose knowing it.",
);

/// What installing leaves alone, for an application whose makers ask for it to
/// read and change the person's files and use the internet.
pub const VERB_LEAVES_REACHING_YOUR_FILES_AND_THE_INTERNET: Word = Word::saying(
    "software.verb.install-application.leaves.reaching-your-files-and-the-internet",
    "Its makers ask for it to read and change your files without asking you first, and to use the internet, and it will be able to do both. No other application is updated or removed.",
)
.noting(
    "Shown on the card a person answers before alo installs an application, in place of the line that says it is given no access to their files. It names everything the application's makers ask for that reaches past the closed box applications are kept in, said before the person approves, so they choose knowing it.",
);

/// What installing leaves alone, for an application whose makers ask for it to
/// start programs outside its box and read and change the person's files.
pub const VERB_LEAVES_RUNNING_PROGRAMS_OUTSIDE_AND_YOUR_FILES: Word = Word::saying(
    "software.verb.install-application.leaves.running-programs-outside-and-your-files",
    "Its makers ask for it to run programs outside its box and to read and change your files without asking you first, so it will be able to do anything you can on this computer. No other application is updated or removed.",
)
.noting(
    "Shown on the card a person answers before alo installs an application, in place of the line that says it is given no access to their files. It names everything the application's makers ask for that reaches past the closed box applications are kept in, said before the person approves, so they choose knowing it.",
);

/// What installing leaves alone, for an application whose makers ask for it to
/// start programs outside its box and use the internet.
pub const VERB_LEAVES_RUNNING_PROGRAMS_OUTSIDE_AND_THE_INTERNET: Word = Word::saying(
    "software.verb.install-application.leaves.running-programs-outside-and-the-internet",
    "Its makers ask for it to run programs outside its box and to use the internet, so it will be able to do anything you can on this computer. No other application is updated or removed.",
)
.noting(
    "Shown on the card a person answers before alo installs an application, in place of the line that says it is given no access to their files. It names everything the application's makers ask for that reaches past the closed box applications are kept in, said before the person approves, so they choose knowing it.",
);

/// What installing leaves alone, for an application whose makers ask for it to
/// start programs outside its box, read and change the person's files and use the internet.
pub const VERB_LEAVES_RUNNING_PROGRAMS_OUTSIDE_YOUR_FILES_AND_THE_INTERNET: Word = Word::saying(
    "software.verb.install-application.leaves.running-programs-outside-your-files-and-the-internet",
    "Its makers ask for it to run programs outside its box, to read and change your files without asking you first, and to use the internet, so it will be able to do anything you can on this computer. No other application is updated or removed.",
)
.noting(
    "Shown on the card a person answers before alo installs an application, in place of the line that says it is given no access to their files. It names everything the application's makers ask for that reaches past the closed box applications are kept in, said before the person approves, so they choose knowing it.",
);

/// What a person presses to have an application installed.
pub const INSTALL_IT: Word = Word::saying("software.install-it", "Install").noting(
    "The control a person presses on an application they have not got. One word where the \
     language has one. It asks for the installation; it is not a promise that it has happened.",
);

/// What the control says while the machine is doing it.
///
/// **Not a progress figure.** How far an installation has got is the rented
/// tool's business and this machine does not translate its numbers; what a
/// person needs here is that it is going, and the indicator says what is
/// leaving while it does.
pub const INSTALLING_IT: Word = Word::saying("software.installing-it", "Installing…").noting(
    "Replaces the install control while an application is arriving. The ellipsis is the \
     ordinary one for work in progress; use whatever a reader of this language expects, or \
     nothing if the language does not use one.",
);

/// What the control says once the application is on the machine.
pub const OPEN_IT: Word = Word::saying("software.open-it", "Open").noting(
    "Replaces the install control once an application is on this machine. One word where the \
     language has one. It opens what is already here and installs nothing.",
);

/// The kind of work that narrows nothing.
///
/// **A category is a thing a person makes, never a product it replaces.**
/// `crate::Category` says why, and these are the words that enum is drawn
/// through — never `Category::named`, which is the spelling the data file uses
/// and is not translated.
pub const EVERYTHING: Word = Word::saying("software.category.everything", "Everything").noting(
    "The first of the kinds of work a person can narrow the applications by, and the one that \
     narrows nothing. Shown chosen while no kind has been chosen.",
);

/// Applications for changing a picture that exists already.
pub const PICTURES: Word = Word::saying("software.category.pictures", "Pictures").noting(
    "A kind of work: changing a picture that already exists — cropping it, correcting it, \
     cutting something out, putting two together.",
);

/// Applications for making a picture with a brush.
pub const PAINTING: Word = Word::saying("software.category.painting", "Painting")
    .noting("A kind of work: making a picture that does not exist yet, with a brush.");

/// Applications for drawing lines that stay sharp at any size.
pub const ILLUSTRATION: Word = Word::saying("software.category.illustration", "Illustration")
    .noting(
        "A kind of work: drawing something that stays sharp at any size — a logo, a sign, a \
         shape to be cut out.",
    );

/// Applications for developing what a camera recorded.
pub const PHOTOGRAPHS: Word = Word::saying("software.category.photographs", "Photographs").noting(
    "A kind of work: developing what a camera recorded, graded and exported without altering \
     the originals.",
);

/// Applications for arranging a page to be printed.
pub const PAGE_LAYOUT: Word = Word::saying("software.category.page-layout", "Page layout").noting(
    "A kind of work: arranging a page that will be printed, with the colour a printer asks for. \
     Two words in English; one word in languages that have one.",
);

/// Applications for cutting video.
pub const VIDEO: Word = Word::saying("software.category.video", "Video").noting(
    "A kind of work: cutting video — clips in order, with sound, exported to play elsewhere.",
);

/// Applications for recording and editing sound.
pub const AUDIO: Word = Word::saying("software.category.audio", "Audio").noting(
    "A kind of work: recording and editing sound — an interview, a voice over, a recording to \
     publish.",
);

/// Applications for writing something somebody else can open.
pub const DOCUMENTS: Word = Word::saying("software.category.documents", "Documents").noting(
    "A kind of work: writing, calculating or presenting something somebody else can open. This \
     machine can read such a document before anything is installed; this is the kind of \
     application that writes one.",
);

/// Applications for this kind of work: diagrams.
pub const DIAGRAMS: Word = Word::saying("software.category.diagrams", "Diagrams").noting(
    "A kind of work: drawing boxes joined by lines and arrows — a flow chart, a plan, an organisation, a network.",
);

/// Applications for this kind of work: 3d models.
pub const MODELLING: Word = Word::saying("software.category.modelling", "3D models").noting(
    "A kind of work: making things in three dimensions — an object to render, animate or manufacture. Keep it short; 3D is understood in most languages.",
);

/// Applications for this kind of work: making music.
pub const MUSIC_MAKING: Word = Word::saying("software.category.music-making", "Making music").noting(
    "A kind of work: writing, playing in and recording music — a score, a track, a mix. Not listening to music, which is its own kind.",
);

/// Applications for this kind of work: screen recording.
pub const SCREEN_RECORDING: Word =
    Word::saying("software.category.screen-recording", "Screen recording")
        .noting("A kind of work: recording or streaming what is on the screen as it happens.");

/// Applications for this kind of work: code.
pub const CODE: Word = Word::saying("software.category.code", "Code")
    .noting("A kind of work: writing, running and correcting computer programs.");

/// Applications for this kind of work: chat.
pub const CHAT: Word = Word::saying("software.category.chat", "Chat").noting(
    "A kind of application: writing messages to other people and reading theirs as they arrive.",
);

/// Applications for this kind of work: calls.
pub const CALLS: Word = Word::saying("software.category.calls", "Calls").noting(
    "A kind of application: talking to other people over the network, with or without video.",
);

/// Applications for this kind of work: email.
pub const EMAIL: Word = Word::saying("software.category.email", "Email")
    .noting("A kind of application: sending and receiving letters by address.");

/// Applications for this kind of work: notes.
pub const NOTES: Word = Word::saying("software.category.notes", "Notes").noting(
    "A kind of application: keeping notes for oneself, typed, linked together or written by hand.",
);

/// Applications for this kind of work: music.
pub const MUSIC: Word = Word::saying("software.category.music", "Music").noting(
    "A kind of application: listening to music and radio. Not making music, which is its own kind.",
);

/// Applications for this kind of work: books.
pub const BOOKS: Word = Word::saying("software.category.books", "Books")
    .noting("A kind of application: reading books on the screen and keeping a shelf of them.");

/// Applications for this kind of work: maps.
pub const MAPS: Word = Word::saying("software.category.maps", "Maps")
    .noting("A kind of application: finding where a place is and the way there.");

/// Applications for this kind of work: games.
pub const GAMES: Word = Word::saying("software.category.games", "Games")
    .noting("A kind of application: playing games, and the places games are bought or kept.");

/// Applications for this kind of work: passwords.
pub const PASSWORDS: Word = Word::saying("software.category.passwords", "Passwords")
    .noting("A kind of application: keeping passwords and other secrets in one locked place.");

/// Applications for this kind of work: another computer, used from this one.
pub const REMOTE_DESKTOP: Word = Word::saying("software.category.remote-desktop", "Another computer").noting(
    "A kind of application: seeing and using another computer from this one, over the network. Use the ordinary term in the language, if it has one.",
);

/// Every kind of work, in the order the window shows them.
pub const EVERY_CATEGORY: [Word; 24] = [
    EVERYTHING,
    PICTURES,
    PAINTING,
    ILLUSTRATION,
    PHOTOGRAPHS,
    PAGE_LAYOUT,
    VIDEO,
    AUDIO,
    DOCUMENTS,
    DIAGRAMS,
    MODELLING,
    MUSIC_MAKING,
    SCREEN_RECORDING,
    CODE,
    CHAT,
    CALLS,
    EMAIL,
    NOTES,
    MUSIC,
    BOOKS,
    MAPS,
    GAMES,
    PASSWORDS,
    REMOTE_DESKTOP,
];

// ---------------------------------------------------------------------------
// What the market says beside an application, when there is something to know.
// ---------------------------------------------------------------------------

/// Beside an application whose source code is not published.
pub const NOT_OPEN_SOURCE: Word = Word::saying("software.label.not-open-source", "Not open source")
    .noting(
        "Shown beside an application in the list a person installs from, when the people who make it do not publish the code it is built from. A plain fact, not a warning: the person may install it. Three or four words at most where the language allows, because it sits on one line under the application's name.",
    );

/// The same, in full, for a screen reader.
pub const NOT_OPEN_SOURCE_IN_FULL: Word = Word::saying(
    "software.label.not-open-source.in-full",
    "Not open source: the people who make it do not publish the code it is built from, so nobody outside can check what it does",
)
.noting(
    "What a screen reader says for the short label Not open source, and what a person reads if they ask what it means. It states a fact and its consequence; it does not tell the person not to install the application.",
);

/// Beside an application its makers did not publish at its source themselves.
pub const NOT_VERIFIED_BY_ITS_MAKER: Word = Word::saying("software.label.not-verified", "Not verified")
    .noting(
        "Shown beside an application in the list a person installs from, when the place it comes from cannot confirm that the people who make it published it there themselves. Somebody else may have published it. A plain fact, not a warning. Short, because it sits on one line under the application's name.",
    );

/// The same, in full, for a screen reader.
pub const NOT_VERIFIED_BY_ITS_MAKER_IN_FULL: Word = Word::saying(
    "software.label.not-verified.in-full",
    "Not verified: the people who make it did not publish it here themselves, so the place it comes from cannot confirm who did",
)
.noting(
    "What a screen reader says for the short label Not verified, and what a person reads if they ask what it means. 'Here' is the place the application is installed from. It does not say the application is unsafe, only who published it.",
);

/// Beside an application that is new and still changing.
pub const EARLY_VERSION: Word = Word::saying("software.label.early-version", "Early version")
    .noting(
        "Shown beside an application in the list a person installs from, when it is new and its makers are still changing it a great deal. A plain fact, not a warning. Short, because it sits on one line under the application's name.",
    );

/// The same, in full, for a screen reader.
pub const EARLY_VERSION_IN_FULL: Word = Word::saying(
    "software.label.early-version.in-full",
    "Early version: it is new and still changing, so some of it may not work yet",
)
.noting(
    "What a screen reader says for the short label Early version, and what a person reads if they ask what it means.",
);

/// Beside an application installed from its maker's own signed release.
pub const NO_AUTOMATIC_UPDATES: Word = Word::saying(
    "software.label.no-automatic-updates",
    "No automatic updates",
)
.noting(
    "Shown beside an application in the list a person installs from, when it comes from its \
         makers' own signed release rather than from the usual place. This computer does not \
         offer its newer versions the way it does for other applications. A plain fact, not a \
         warning. Short, because it sits on one line under the application's name.",
);

/// The same, in full, for a screen reader.
pub const NO_AUTOMATIC_UPDATES_IN_FULL: Word = Word::saying(
    "software.label.no-automatic-updates.in-full",
    "No automatic updates: it comes from its makers' own release, and this computer does not \
     offer its newer versions the way it does for your other applications",
)
.noting(
    "What a screen reader says for the short label No automatic updates, and what a person reads \
     if they ask what it means. It does not say the application can never be updated, only that \
     this computer does not offer it.",
);

/// Every label's words, short then in full, in `crate::Label::EVERY`'s order.
pub const EVERY_LABEL: [Word; 12] = [
    RUNS_PROGRAMS_OUTSIDE_ITS_BOX,
    RUNS_PROGRAMS_OUTSIDE_ITS_BOX_IN_FULL,
    REACHES_YOUR_FILES,
    REACHES_YOUR_FILES_IN_FULL,
    NOT_OPEN_SOURCE,
    NOT_OPEN_SOURCE_IN_FULL,
    NOT_VERIFIED_BY_ITS_MAKER,
    NOT_VERIFIED_BY_ITS_MAKER_IN_FULL,
    EARLY_VERSION,
    EARLY_VERSION_IN_FULL,
    NO_AUTOMATIC_UPDATES,
    NO_AUTOMATIC_UPDATES_IN_FULL,
];

/// What `install_application` changes, under the action on the approval card.
pub const VERB_CHANGES: Word = Word::saying(
    "software.verb.install-application.changes",
    "Downloads this application and installs it for everyone who uses this computer.",
)
.noting(
    "Shown on the card a person answers before alo installs an application, beside a tick. \
     'For everyone' is true: it is installed for the whole computer, not one account. Held to \
     it by a test.",
);
/// What `install_application` leaves alone.
pub const VERB_LEAVES: Word = Word::saying(
    "software.verb.install-application.leaves",
    "It is given no access to your files, and no other application is updated or removed.",
)
.noting(
    "Shown beside VERB_CHANGES. A newly installed application can reach nothing the person has \
     not granted it. Held to it by a test.",
);

/// Every string this crate can say.
pub const EVERY_WORD: [Word; 76] = [
    VERB_PURPOSE,
    VERB_APPLICATION,
    VERB_SOURCE,
    READ_PURPOSE,
    READ_KIND_OF_WORK,
    READ_SENTENCE,
    INSTALLED,
    INSTALLED_UNSANDBOXED,
    UPDATE_OFFERED,
    UPDATED,
    REMOVED,
    NOT_ENABLED,
    OUTSIDE_THE_BOUND_BY_AN_ADMINISTRATOR,
    OUTSIDE_THE_BOUND_BY_THIS_PERSON,
    NOT_VERIFIED,
    SIGNATURE_NOT_SHOWN,
    NOT_OFFERED,
    NOT_HELD_BUT_THIS_IS,
    NAMES_MORE_THAN_ONE,
    ALREADY_INSTALLED,
    NOT_INSTALLED,
    STILL_OPEN,
    NOWHERE_TO_REACH,
    DID_NOT_ANSWER,
    VERB_SENTENCE,
    VERB_CHANGES,
    VERB_LEAVES,
    THE_APPLICATIONS,
    EACH_SAYS_WHAT_IT_REACHES,
    RUNS_PROGRAMS_OUTSIDE_ITS_BOX,
    RUNS_PROGRAMS_OUTSIDE_ITS_BOX_IN_FULL,
    REACHES_YOUR_FILES,
    REACHES_YOUR_FILES_IN_FULL,
    INSTALLED_WITH_WHAT_ITS_MAKERS_ASK,
    VERB_LEAVES_REACHING_YOUR_FILES,
    VERB_LEAVES_RUNNING_PROGRAMS_OUTSIDE,
    VERB_LEAVES_USING_THE_INTERNET,
    VERB_LEAVES_REACHING_YOUR_FILES_AND_THE_INTERNET,
    VERB_LEAVES_RUNNING_PROGRAMS_OUTSIDE_AND_YOUR_FILES,
    VERB_LEAVES_RUNNING_PROGRAMS_OUTSIDE_AND_THE_INTERNET,
    VERB_LEAVES_RUNNING_PROGRAMS_OUTSIDE_YOUR_FILES_AND_THE_INTERNET,
    INSTALL_IT,
    INSTALLING_IT,
    OPEN_IT,
    EVERYTHING,
    PICTURES,
    PAINTING,
    ILLUSTRATION,
    PHOTOGRAPHS,
    PAGE_LAYOUT,
    VIDEO,
    AUDIO,
    DOCUMENTS,
    DIAGRAMS,
    MODELLING,
    MUSIC_MAKING,
    SCREEN_RECORDING,
    CODE,
    CHAT,
    CALLS,
    EMAIL,
    NOTES,
    MUSIC,
    BOOKS,
    MAPS,
    GAMES,
    PASSWORDS,
    REMOTE_DESKTOP,
    NOT_OPEN_SOURCE,
    NOT_OPEN_SOURCE_IN_FULL,
    NOT_VERIFIED_BY_ITS_MAKER,
    NOT_VERIFIED_BY_ITS_MAKER_IN_FULL,
    EARLY_VERSION,
    EARLY_VERSION_IN_FULL,
    NO_AUTOMATIC_UPDATES,
    NO_AUTOMATIC_UPDATES_IN_FULL,
];

/// Why this crate's own words could not be declared.
///
/// Neither can happen to the list above — the tests at the bottom of this file
/// say so — and [`declare_into`] can genuinely fail against a vocabulary that
/// already holds one of these keys.
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum WordsError {
    /// A word that is not a phrase.
    #[error(transparent)]
    Word(#[from] WordError),
    /// A key the vocabulary already has.
    #[error(transparent)]
    List(#[from] VocabularyError),
}

/// Everything this crate can say, as a vocabulary of its own.
///
/// # Errors
/// [`WordsError`], which the list above cannot cause.
pub fn software_words() -> Result<Vocabulary, WordsError> {
    let mut vocabulary = Vocabulary::empty();
    declare_into(&mut vocabulary)?;
    Ok(vocabulary)
}

/// Put everything this crate can say into an existing vocabulary.
///
/// # Errors
/// [`WordsError::List`] if the vocabulary already holds one of these keys —
/// nothing is replaced.
pub fn declare_into(vocabulary: &mut Vocabulary) -> Result<(), WordsError> {
    for word in EVERY_WORD {
        vocabulary.says(word.phrase()?)?;
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

    /// Every key is a key, in this crate's area, and names one string.
    #[test]
    fn every_key_is_one_of_this_crates_and_names_one_string() {
        for word in EVERY_WORD {
            assert_eq!(alo_strings::Key::named(word.named()), Ok(word.key()));
            assert_eq!(word.key().area(), "software", "{}", word.named());
        }
        let named: BTreeSet<&str> = EVERY_WORD.iter().map(Word::named).collect();
        assert_eq!(named.len(), EVERY_WORD.len());
    }

    /// The two groups are the whole list.
    #[test]
    fn the_two_groups_are_the_whole_list() {
        let mut both: Vec<&str> = THE_NAMES.iter().map(Word::named).collect();
        both.extend(THE_SENTENCES.iter().map(Word::named));
        let every: Vec<&str> = EVERY_WORD.iter().map(Word::named).collect();
        assert_eq!(both, every);
    }

    /// Every refusal is one of the sentences, and each of them is there once.
    #[test]
    fn every_refusal_is_one_of_the_sentences() {
        let sentences: BTreeSet<&str> = THE_SENTENCES.iter().map(Word::named).collect();
        let refusals: BTreeSet<&str> = THE_REFUSALS.iter().map(Word::named).collect();
        assert_eq!(refusals.len(), THE_REFUSALS.len());
        assert!(refusals.is_subset(&sentences));
    }

    /// The list declares, and a key already taken is not replaced.
    #[test]
    fn the_list_declares_and_nothing_is_replaced() {
        let mut vocabulary = software_words().unwrap();
        assert_eq!(vocabulary.how_many(), EVERY_WORD.len());
        assert!(matches!(
            declare_into(&mut vocabulary),
            Err(WordsError::List(_))
        ));
    }

    /// **Every word carries a note**, and every gap is one this crate fills.
    #[test]
    fn every_word_carries_a_note_and_only_gaps_this_crate_fills() {
        for word in EVERY_WORD {
            assert!(word.note().is_some(), "{}", word.named());
            for gap in word.phrase().unwrap().source().gaps() {
                assert!(
                    [APPLICATION, SOURCE, INSTEAD, KIND_OF_WORK].contains(&gap.as_str()),
                    "{} has a gap called {gap}",
                    word.named()
                );
            }
        }
    }

    /// **Nothing here hedges and nothing names the machinery** — not in the
    /// sentence and not in the note a translator reads, because a translator
    /// handed the tool's name has been handed our plumbing.
    #[test]
    fn nothing_here_hedges_or_names_the_machinery() {
        for word in EVERY_WORD {
            for text in [word.says(), word.note().unwrap_or_default()] {
                let text = text.to_lowercase();
                for forbidden in [
                    "probably",
                    "might",
                    "perhaps",
                    "possibly",
                    "flatpak",
                    "flathub",
                    "ostree",
                    "remote",
                    "repository",
                    "repo ",
                    "gpg",
                    "branch",
                    "sandbox",
                    "package",
                    "error",
                    "d-bus",
                    "portal",
                ] {
                    assert!(
                        !text.contains(forbidden),
                        "{} says \"{forbidden}\"",
                        word.named()
                    );
                }
            }
        }
    }

    /// **Every refusal says what did not happen and what to do about it** — a
    /// second sentence or clause after the trouble, never the trouble alone.
    #[test]
    fn every_refusal_says_what_to_do_about_it() {
        for word in THE_REFUSALS {
            let said = word.says();
            assert!(
                said.contains("nothing") || said.contains("not"),
                "{}",
                word.named()
            );
            assert!(
                said.contains(". ") || said.contains(": "),
                "{} names the trouble and not what to do",
                word.named()
            );
        }
    }
}
