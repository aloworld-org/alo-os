//! Every string `alo-app` can say, and the English beside each one.
//!
//! Names and values for a program — an identifier, an ability's name, a kind of
//! control as `press` takes it — are never here and never translated, so a
//! script gives the same result on a Greek machine as on a German one
//! (`docs/contracts/terminal-commands.md`, *What it says*). What is here is
//! every sentence a person reads.
//!
//! Sentences other crates already say are said by them: a value refused by the
//! validation alo's own calls go through is `alo-capability`'s sentence, a
//! control that is not there is the accessibility fallback's, and an
//! application that would not answer is the adapter's. One refusal, one
//! wording, whoever asked.

use alo_strings::{Vocabulary, VocabularyError, WordError};

/// One string a crate can say.
pub use alo_strings::Word;

/// The gap holding an application, as a person reads it: its identifier, or
/// its name and the window for `front`.
pub const APPLICATION: &str = "application";

/// The gap holding an application's identifier.
pub const IDENTIFIER: &str = "identifier";

/// The gap holding an application's name as its menu entry shows it.
pub const NAME: &str = "name";

/// The gap holding a short name.
pub const SHORT: &str = "short";

/// The gap holding an ability's declared name.
pub const ABILITY: &str = "ability";

/// The gap holding a window's title.
pub const WINDOW: &str = "window";

/// The gap holding a kind of control, in words.
pub const KIND: &str = "kind";

/// The gap holding a control's name.
pub const CONTROL: &str = "control";

/// The gap holding a list, already joined.
pub const LIST: &str = "list";

/// The gap holding a sentence said elsewhere.
pub const SAID: &str = "said";

/// The gap holding a word that was typed and not understood.
pub const TYPED: &str = "typed";

// ---------------------------------------------------------------------------
// The command was not written in this shape (exit 64).
// ---------------------------------------------------------------------------

/// Nothing was typed after the command's name.
pub const NOTHING_ASKED: Word = Word::saying(
    "app.typed.nothing",
    "Nothing was asked. Type alo-app list to see what is installed, and alo-app show and an \
     application to see what it offers",
)
.noting(
    "Said when a person types alo-app alone. alo-app, list and show are what they type and are \
     not translated.",
);

/// An application was named and nothing to do with it.
pub const NOTHING_TO_DO: Word = Word::saying(
    "app.typed.nothing-to-do",
    "Say what to do with {application}: one of its abilities, or controls, read, press, open, \
     focus or close",
)
.noting(
    "{application} is what the person typed. controls, read, press, open, focus and close are \
     words they type and are not translated.",
);

/// Something was left over after a complete command.
pub const LEFT_OVER: Word = Word::saying(
    "app.typed.left-over",
    "“{typed}” was left over at the end, so nothing was done",
)
.noting(
    "{typed} is exactly what the person typed after a command that was already complete. \
     Nothing happened.",
);

/// `press` without a kind and a name.
pub const PRESS_NEEDS_TWO: Word = Word::saying(
    "app.typed.press-needs-two",
    "press takes the kind of control and its name, like press button Save, so nothing was \
     pressed",
)
.noting("press and button are words the person types and are not translated.");

/// `--in` with nothing after it.
pub const IN_NEEDS_A_WINDOW: Word = Word::saying(
    "app.typed.in-needs-a-window",
    "--in takes the title of one window after it, so nothing was done",
)
.noting("--in is typed as it is and is not translated.");

/// `--in` twice.
pub const IN_TWICE: Word = Word::saying(
    "app.typed.in-twice",
    "--in was given twice, and a command reaches one window, so nothing was done",
)
.noting("--in is typed as it is and is not translated.");

/// `front` and `--in` together.
pub const FRONT_AND_IN: Word = Word::saying(
    "app.typed.front-and-in",
    "front already means one window, the one you were working in, so --in cannot be added to it",
)
.noting("front and --in are typed as they are and are not translated.");

/// `--in`, or `front`, with a form that acts on the whole application.
pub const WHOLE_APPLICATION: Word = Word::saying(
    "app.typed.whole-application",
    "open, focus and close act on the whole application, not on one window, so they cannot be \
     narrowed with --in or used with front",
)
.noting(
    "Said rather than closing every window of an application when the person meant one. open, \
     focus, close, --in and front are typed as they are and are not translated.",
);

/// `--json` where it is not offered.
pub const JSON_NOT_HERE: Word = Word::saying(
    "app.typed.json-not-here",
    "--json is offered for list and show, so nothing was done",
)
.noting("--json, list and show are typed as they are and are not translated.");

/// `--plain` and `--json` together.
pub const PLAIN_AND_JSON: Word = Word::saying(
    "app.typed.plain-and-json",
    "--plain and --json ask for two different kinds of answer, so nothing was done",
)
.noting("--plain and --json are typed as they are and are not translated.");

/// More values than an ability takes.
pub const TOO_MANY_VALUES: Word = Word::saying(
    "app.typed.too-many-values",
    "More values were given than {ability} takes, so nothing was done. alo-app show and the \
     application lists what it takes",
)
.noting(
    "{ability} is the declared name of what was asked, like open_document, and is not \
     translated.",
);

/// More than one value from the pipe.
pub const TWO_PIPES: Word = Word::saying(
    "app.typed.two-pipes",
    "Only one value can be read from the pipe, and more than one was given as -, so nothing was \
     done",
)
.noting("- is what a person types to mean the pipe, and is not translated.");

/// `--<argument>` with nothing after it.
pub const NAMED_NEEDS_A_VALUE: Word = Word::saying(
    "app.typed.named-needs-a-value",
    "--{typed} takes a value after it, so nothing was done",
)
.noting("{typed} is the argument's declared name and is not translated.");

// ---------------------------------------------------------------------------
// Naming the application.
// ---------------------------------------------------------------------------

/// No installed application answers to that name (exit 69).
pub const NOT_INSTALLED: Word = Word::saying(
    "app.naming.not-installed",
    "No installed application is called {typed}. alo-app list shows what is installed",
)
.noting(
    "{typed} is what the person typed. Applications are named by their identifier or their \
     short name, never by the name their menu shows. Nothing happened.",
);

/// Two installed applications share a short name (exit 65).
pub const SHARED_SHORT_NAME: Word = Word::saying(
    "app.naming.shared-short-name",
    "More than one installed application is called {typed}: {list}. Name the one you mean by its \
     identifier",
)
.noting(
    "{typed} is the short name the person typed; {list} is every identifier it could mean, \
     separated by commas, and is not translated. Nothing is guessed and nothing happened.",
);

// ---------------------------------------------------------------------------
// The window the person was working in, and alo's own surfaces.
// ---------------------------------------------------------------------------

/// `front`, and nothing was in front before this terminal (exit 69).
pub const NO_FRONT: Word = Word::saying(
    "app.front.none",
    "There is no window you were working in before this terminal",
)
.noting(
    "The contract's own sentence. Said when nothing had the keyboard before the terminal, or \
     the command was not typed in a terminal at all. Nothing happened.",
);

/// `front`, and the window cannot be named (exit 69).
pub const FRONT_UNNAMED: Word = Word::saying(
    "app.front.unnamed",
    "The window you were working in before this terminal does not say which application it \
     belongs to, so nothing was done to it",
)
.noting(
    "Said rather than acting on a window as something unknown. Nothing happened, and no other \
     window was chosen instead.",
);

/// The window `front` means, as a person reads it.
pub const FRONT_WINDOW: Word = Word::saying("app.front.window", "{application}, {window}").noting(
    "Names the window the person was working in: {application} is the application's name \
         as its menu shows it, and {window} is the window's title, both shown as they are. It \
         goes inside other sentences, like “Pressed the button “Save” in Text Editor, \
         today.txt”.",
);

/// A sentence said elsewhere, about the window in front, naming that window.
pub const FRONT_SAYS: Word = Word::saying("app.front.says", "{application}: {said}").noting(
    "Puts the window the person was working in in front of a sentence another part of alo OS \
     wrote, so every line about it names it. {application} is the application's name and the \
     window's title; {said} is that sentence, already in the person's language.",
);

/// The desktop did not answer a question about which window or which tree.
pub const DESKTOP_SILENT: Word = Word::saying(
    "app.desktop.silent",
    "The desktop did not answer, so nothing was done",
)
.noting(
    "Said when the compositor's door could not be reached or answered something this command \
     does not understand, before anything was done to any application.",
);

/// alo's own surfaces are on no tree this session (exit 69).
pub const NO_ALO_TREE: Word = Word::saying(
    "app.alo.no-tree",
    "alo OS's own surfaces are not on the accessibility tree in this session, so they cannot be \
     reached from the terminal",
)
.noting(
    "Said when the shell publishes no accessibility tree: no accessibility bus answered when \
     the session began. Nothing happened.",
);

/// alo's own surfaces go by the tree only (exit 69).
pub const ALO_TREE_ONLY: Word = Word::saying(
    "app.alo.tree-only",
    "alo OS's own surfaces are reached through the accessibility tree only: controls, read and \
     press, with --in naming the surface",
)
.noting(
    "Said when a person asks alo for anything else. controls, read, press and --in are typed \
     as they are and are not translated. Nothing happened.",
);

// ---------------------------------------------------------------------------
// Values.
// ---------------------------------------------------------------------------

/// A kind of control `press` does not take (exit 65).
pub const NO_SUCH_KIND: Word = Word::saying(
    "app.press.no-such-kind",
    "{typed} is not a kind of control alo-app can press. It presses: {list}",
)
.noting(
    "{typed} is what the person typed; {list} is the kinds press takes, as they are typed \
     (button, check_box, …), and is not translated. Nothing was pressed.",
);

/// A value a sentence cannot hold (exit 65).
pub const UNREADABLE_VALUE: Word = Word::saying(
    "app.value.unreadable",
    "A value had a character in it that cannot be shown in a sentence, such as an escape \
     sequence, so it was not used",
)
.noting(
    "The value itself is never shown, because showing it is what could change what the \
     terminal says. Nothing was done with it.",
);

// ---------------------------------------------------------------------------
// What was done.
// ---------------------------------------------------------------------------

/// A control was pressed.
pub const PRESSED: Word = Word::saying(
    "app.press.done",
    "Pressed the {kind} “{control}” in {application}",
)
.noting(
    "{kind} is a kind of control in words, like button; {control} is the name the control \
     shows; {application} is the application, or its name and the window's title.",
);

/// A press was sent and the application did not answer (exit 75).
pub const PRESS_UNANSWERED: Word = Word::saying(
    "app.press.unanswered",
    "Asked {application} to press the {kind} “{control}”, and it did not answer, so whether it \
     was pressed is not known",
)
.noting(
    "Not a failure: the press was sent and may have happened. It is written down as having \
     run.",
);

/// A declared ability was carried out.
pub const DECLARED_DONE: Word = Word::saying("app.declared.done", "Done: {said}").noting(
    "{said} is the ability's own sentence with the person's values in it, like “open \
     /home/anna/today.txt in GNOME Text Editor”, already in their language.",
);

/// A declared ability was sent and not answered (exit 75).
pub const DECLARED_UNANSWERED: Word = Word::saying(
    "app.declared.unanswered",
    "Sent: {said}. It did not answer, so whether it happened is not known",
)
.noting(
    "{said} is the ability's own sentence with the person's values in it. It is written down as \
     having run, because it may have.",
);

/// An application does not offer that ability (exit 69).
pub const NO_SUCH_ABILITY: Word = Word::saying(
    "app.declared.no-such-ability",
    "{application} offers no ability called {ability}. alo-app show {application} lists what it \
     offers",
)
.noting(
    "{application} is the identifier and {ability} is what the person typed; neither is \
     translated. Nothing happened.",
);

/// The window was asked to open.
pub const ASKED_TO_OPEN: Word = Word::saying("app.window.opened", "Asked {application} to open")
    .noting("The desktop asked; the application may take a moment to appear.");

/// The window was asked to come forward.
pub const ASKED_TO_FOCUS: Word = Word::saying(
    "app.window.focused",
    "Asked {application} to come to the front",
)
.noting("The desktop brought forward the window of it the person used most recently.");

/// The window was asked to close.
pub const ASKED_TO_CLOSE: Word = Word::saying(
    "app.window.closed",
    "Asked {application} to close. It may ask you about unsaved work first",
)
.noting(
    "Nothing is forced closed: every window of the application is asked, and it may put up its \
     own question.",
);

/// The application has no window (exit 1).
pub const NO_WINDOW_TO_ACT_ON: Word = Word::saying(
    "app.window.no-window",
    "{application} has no window here, so nothing was done",
)
.noting("Said for focus and close when the application is not showing anything.");

/// The desktop says it is not installed (exit 69).
pub const DESKTOP_SAYS_NOT_INSTALLED: Word = Word::saying(
    "app.window.not-installed",
    "The desktop says {application} is not installed, so nothing was done",
)
.noting("{application} is the identifier and is not translated.");

/// The application was asked and has not done it (exit 1).
pub const IT_DID_NOT: Word = Word::saying(
    "app.window.did-not",
    "{application} was asked and has not done it",
)
.noting(
    "Not a fault: an application asked to close may ask the person first, and that answer is \
     theirs.",
);

/// The desktop could not ask the application (exit 69).
pub const COULD_NOT_ASK: Word = Word::saying(
    "app.window.could-not-ask",
    "This machine could not ask {application}, so nothing was done",
)
.noting("{application} is the identifier and is not translated.");

/// The desktop answered something unknown after the act was sent (exit 75).
pub const WINDOW_UNANSWERED: Word = Word::saying(
    "app.window.unanswered",
    "Asked the desktop to act on {application}, and its answer could not be understood, so \
     whether anything happened is not known",
)
.noting("Written down as having run, because it may have.");

// ---------------------------------------------------------------------------
// What the application sends, while the command runs (ADR 0148).
// ---------------------------------------------------------------------------

/// No count could be held.
pub const CANNOT_COUNT: Word = Word::saying(
    "app.count.cannot",
    "alo cannot show what {application} sends during this command",
)
.noting(
    "Said on the error output, and kept in the record, when this machine could not count what \
     an application sent while the person's command ran. The command still ran.",
);

/// A count began and ended without an answer.
pub const COUNT_UNENDED: Word = Word::saying(
    "app.count.unended",
    "How much {application} sent during this command is not known: the count ended without an \
     answer",
)
.noting("Kept in the record. The command ran.");

/// alo's own surfaces are not counted again.
pub const ALOS_OWN_NOT_COUNTED: Word = Word::saying(
    "app.count.alos-own",
    "What alo OS itself sends is shown on the indicator, so it is not counted again for this \
     command",
)
.noting(
    "Kept in the record for a press on Settings, the Dock or another of alo OS's own surfaces.",
);

// ---------------------------------------------------------------------------
// The place to return to (ADR 0127).
// ---------------------------------------------------------------------------

/// The service did not answer, so no place was kept.
pub const NO_PLACE_UNANSWERED: Word = Word::saying(
    "app.place.unanswered",
    "No place to return to was kept before this command, because alo-agentd did not answer. The \
     command ran anyway",
)
.noting(
    "alo-agentd is the service that keeps places to return to and is not translated. Put back \
     cannot bring back what this command changes.",
);

// ---------------------------------------------------------------------------
// What a window shows.
// ---------------------------------------------------------------------------

/// The heading above one window's controls.
pub const IN_THE_WINDOW: Word = Word::saying("app.controls.window", "In the window “{window}”:")
    .noting("{window} is the window's title, shown as it is.");

/// One control.
pub const A_CONTROL: Word = Word::saying("app.controls.one", "{kind} “{control}”")
    .noting("{kind} is a kind of control in words; {control} is the name it shows.");

/// One control, with what is true of it.
pub const A_CONTROL_WITH: Word =
    Word::saying("app.controls.one-with", "{kind} “{control}”: {list}").noting(
        "{list} is what is true of it now, separated by commas: greyed out, on, off, or what \
         it holds.",
    );

/// A window that shows no control.
pub const NO_CONTROLS: Word = Word::saying(
    "app.controls.none",
    "This window shows no control that can be named",
)
.noting("Said under a window's heading.");

/// A window that shows no text, said rather than printing nothing.
pub const NOTHING_TO_READ: Word = Word::saying(
    "app.read.none",
    "The window “{window}” shows no text that can be read. An app shows none when it is not \n     describing itself: Settings, Accessibility, Apps describe themselves.",
)
.noting(
    "Said when a person asked to read an application's window and nothing in it could be read. \n     {window} is the window's title. The last part names a switch in Settings by its words.",
);

/// A text field, as a kind.
pub const A_FIELD: Word =
    Word::saying("app.role.field", "field").noting("A kind of control: a box a person types into.");

/// A password field, as a kind.
pub const A_PASSWORD_FIELD: Word = Word::saying("app.role.password-field", "password field")
    .noting("A kind of control. What is in it is never read.");

/// Greyed out.
pub const GREYED_OUT: Word = Word::saying("app.state.greyed-out", "greyed out")
    .noting("A control that cannot be used right now.");

/// On.
pub const STATE_ON: Word = Word::saying("app.state.on", "on")
    .noting("A check box that is ticked, or a switch that is on.");

/// Off.
pub const STATE_OFF: Word = Word::saying("app.state.off", "off")
    .noting("A check box that is not ticked, or a switch that is off.");

/// What a field holds.
pub const HOLDS: Word = Word::saying("app.state.holds", "holds “{said}”")
    .noting("{said} is the text in a field, shown as it is.");

/// A password field's contents.
pub const WITHHELD: Word = Word::saying("app.state.withheld", "what it holds is never read")
    .noting("Said of a password field.");

/// A control with no name.
pub const UNNAMED_CONTROL: Word = Word::saying(
    "app.controls.unnamed",
    "a {kind} with no name, which cannot be named or pressed",
)
.noting("Often a button that shows only an icon.");

/// An area the application draws itself.
pub const NOT_DESCRIBED: Word = Word::saying(
    "app.controls.not-described",
    "an area the application draws itself and does not describe",
)
.noting("A canvas, a game or a video: nothing inside it can be named.");

/// A part shown by another program.
pub const ANOTHER_PROGRAMS: Word = Word::saying(
    "app.controls.another-program",
    "a part of the window shown by another program, which was not read",
)
.noting("Often a web page inside an application.");

// ---------------------------------------------------------------------------
// What is installed, and what each offers.
// ---------------------------------------------------------------------------

/// One application in the list.
pub const LIST_LINE: Word = Word::saying("app.list.line", "{name} — {said}. {list}").noting(
    "{name} is the application's name as its menu shows it. {said} says how to type it and \
     {list} what it offers; both are sentences of their own, already in the person's language.",
);

/// How to type an application: its short name.
pub const TYPE_SHORT: Word = Word::saying("app.reach.short", "type {short}")
    .noting("{short} is the short name, like texteditor, and is not translated.");

/// How to type it: the identifier, because the short name is reserved.
pub const TYPE_IDENTIFIER_RESERVED: Word = Word::saying(
    "app.reach.reserved",
    "reach this application as {identifier}, because {short} is a word alo-app keeps for itself",
)
.noting(
    "The contract's own words. {identifier} and {short} are not translated. The words alo-app \
     keeps are list, show, front and alo.",
);

/// How to type it: the identifier, because the short name is shared.
pub const TYPE_IDENTIFIER_SHARED: Word = Word::saying(
    "app.reach.shared",
    "reach this application as {identifier}, because another installed application is also \
     called {short}",
)
.noting("{identifier} and {short} are not translated.");

/// What every application offers.
pub const OFFERS_ITS_WINDOW: Word = Word::saying(
    "app.offers.window",
    "It can be opened, focused and closed, and its controls pressed where its windows describe \
     them",
)
.noting("Said of every installed application.");

/// What an application with an adapter offers too.
pub const OFFERS_ABILITIES: Word = Word::saying(
    "app.offers.abilities",
    "It can be opened, focused and closed, its controls pressed where its windows describe them, \
     and asked for these abilities of its own: {list}",
)
.noting("{list} is the abilities' declared names, like open_document, and is not translated.");

/// A terminal.
pub const A_TERMINAL: Word = Word::saying(
    "app.offers.terminal",
    "This is a terminal — type into it directly",
)
.noting("The contract's own words, said of a terminal application.");

/// The heading of `show`.
pub const SHOW_HEADING: Word = Word::saying("app.show.heading", "{name} ({identifier}): {said}")
    .noting("{said} says how to type it, already in the person's language.");

/// The window road.
pub const ROAD_WINDOW: Word = Word::saying(
    "app.road.window",
    "Its window: open, focus and close, carried out by the desktop",
)
.noting("open, focus and close are typed as they are and are not translated.");

/// The tree road, and that its windows describe themselves now.
pub const ROAD_TREE_DESCRIBED: Word = Word::saying(
    "app.road.tree.described",
    "Its controls: controls, read and press, through the accessibility tree. Its windows \
     describe themselves there now",
)
.noting("controls, read and press are typed as they are and are not translated.");

/// The tree road, and that none of its windows is open.
pub const ROAD_TREE_NOT_OPEN: Word = Word::saying(
    "app.road.tree.not-open",
    "Its controls: controls, read and press, through the accessibility tree. None of its \
     windows is open now, so whether they describe themselves is not known until one is",
)
.noting("controls, read and press are typed as they are and are not translated.");

/// The tree road, and that this session has no tree.
pub const ROAD_TREE_NOT_HERE: Word = Word::saying(
    "app.road.tree.not-here",
    "Its controls: controls, read and press, through the accessibility tree — which this \
     session does not have now, so they cannot be reached",
)
.noting("controls, read and press are typed as they are and are not translated.");

/// The declared road.
pub const ROAD_DECLARED: Word = Word::saying(
    "app.road.declared",
    "Abilities of its own, declared by an adapter alo OS ships and reviewed:",
)
.noting("A heading above a list of abilities.");

/// No adapter.
pub const ROAD_NO_DECLARED: Word = Word::saying(
    "app.road.no-declared",
    "No adapter declares abilities of its own for it",
)
.noting("An adapter is a reviewed list of what an application can be asked to do.");

/// The discovered road, which this release does not look at.
pub const ROAD_DISCOVERED_NOT_ASKED: Word = Word::saying(
    "app.road.discovered.not-asked",
    "What it publishes about itself on the session bus was not asked: alo-app does not look \
     there yet",
)
.noting("Said so nobody takes silence for an answer. Nothing was looked for.");

/// The screen road, which is not built.
pub const ROAD_SCREEN_NOT_BUILT: Word = Word::saying(
    "app.road.screen.not-built",
    "Reading the screen is not built yet, so a control its windows do not describe cannot be \
     reached",
)
.noting("The screen is the least reliable road, and the last.");

/// One declared ability in `show`.
pub const ABILITY_LINE: Word = Word::saying("app.show.ability", "{ability} — {said}")
    .noting("{ability} is its declared name and is not translated; {said} is what it does.");

/// The surfaces of `alo`.
pub const ALO_SURFACES: Word = Word::saying(
    "app.show.alo",
    "alo OS's own surfaces, reached through the accessibility tree only, with --in naming one: \
     {list}",
)
.noting("{list} is the surfaces' names as they are typed after --in, and is not translated.");

/// What an ability does, in its own sentence.
pub const IN_WORDS: Word = Word::saying("app.show.in-words", "What it does: {said}")
    .noting("{said} is the ability's sentence, with each value it takes shown in angle brackets.");

/// How to type an ability.
pub const TYPED_AS: Word = Word::saying("app.show.typed-as", "Typed as: {said}")
    .noting("{said} is the command line, which is not translated.");

/// One argument of an ability.
pub const ARGUMENT_LINE: Word = Word::saying("app.show.argument", "<{typed}> — {said}")
    .noting("{typed} is the argument's declared name and is not translated; {said} is what it is.");

/// A path argument.
pub const KIND_PATH: Word = Word::saying(
    "app.kind.path",
    "{said}: a path; one that does not begin with / is taken from the folder you are in",
)
.noting("{said} is what the argument is for.");

/// A name argument.
pub const KIND_NAME: Word =
    Word::saying("app.kind.name", "{said}: one name").noting("{said} is what the argument is for.");

/// A number argument.
pub const KIND_COUNT: Word = Word::saying("app.kind.count", "{said}: a whole number")
    .noting("{said} is what the argument is for.");

/// A choice argument.
pub const KIND_CHOICE: Word = Word::saying("app.kind.choice", "{said}: one of {list}")
    .noting("{said} is what the argument is for; {list} is the options as they are typed.");

/// An ability that changes something.
pub const IT_CHANGES: Word = Word::saying(
    "app.show.changes",
    "It changes something, and runs at once when you type it. A place to return to is kept \
     before it, so Put back can bring your files back as they were",
)
.noting(
    "Put back is the record's own button. It brings back files in the person's home, never \
     anything already sent.",
);

/// An ability that only reads.
pub const IT_READS: Word = Word::saying("app.show.reads", "It only reads, and changes nothing")
    .noting("Said of an ability that changes nothing.");

/// An ability that may send something.
pub const CANNOT_BE_UNDONE_ONCE_SENT: Word = Word::saying(
    "app.show.sends",
    "This cannot be undone once it has been sent",
)
.noting("The contract's own words, said beside an ability that sends, or may send, something off the machine.");

/// Every word of this crate's own.
pub const EVERY_WORD: [Word; 84] = [
    NOTHING_ASKED,
    NOTHING_TO_DO,
    LEFT_OVER,
    PRESS_NEEDS_TWO,
    IN_NEEDS_A_WINDOW,
    IN_TWICE,
    FRONT_AND_IN,
    WHOLE_APPLICATION,
    JSON_NOT_HERE,
    PLAIN_AND_JSON,
    TOO_MANY_VALUES,
    TWO_PIPES,
    NAMED_NEEDS_A_VALUE,
    NOT_INSTALLED,
    SHARED_SHORT_NAME,
    NO_FRONT,
    FRONT_UNNAMED,
    FRONT_WINDOW,
    FRONT_SAYS,
    DESKTOP_SILENT,
    NO_ALO_TREE,
    ALO_TREE_ONLY,
    NO_SUCH_KIND,
    UNREADABLE_VALUE,
    PRESSED,
    PRESS_UNANSWERED,
    DECLARED_DONE,
    DECLARED_UNANSWERED,
    NO_SUCH_ABILITY,
    ASKED_TO_OPEN,
    ASKED_TO_FOCUS,
    ASKED_TO_CLOSE,
    NO_WINDOW_TO_ACT_ON,
    DESKTOP_SAYS_NOT_INSTALLED,
    IT_DID_NOT,
    COULD_NOT_ASK,
    WINDOW_UNANSWERED,
    CANNOT_COUNT,
    COUNT_UNENDED,
    ALOS_OWN_NOT_COUNTED,
    NO_PLACE_UNANSWERED,
    IN_THE_WINDOW,
    A_CONTROL,
    A_CONTROL_WITH,
    NO_CONTROLS,
    NOTHING_TO_READ,
    A_FIELD,
    A_PASSWORD_FIELD,
    GREYED_OUT,
    STATE_ON,
    STATE_OFF,
    HOLDS,
    WITHHELD,
    UNNAMED_CONTROL,
    NOT_DESCRIBED,
    ANOTHER_PROGRAMS,
    LIST_LINE,
    TYPE_SHORT,
    TYPE_IDENTIFIER_RESERVED,
    TYPE_IDENTIFIER_SHARED,
    OFFERS_ITS_WINDOW,
    OFFERS_ABILITIES,
    A_TERMINAL,
    SHOW_HEADING,
    ROAD_WINDOW,
    ROAD_TREE_DESCRIBED,
    ROAD_TREE_NOT_OPEN,
    ROAD_TREE_NOT_HERE,
    ROAD_DECLARED,
    ROAD_NO_DECLARED,
    ROAD_DISCOVERED_NOT_ASKED,
    ROAD_SCREEN_NOT_BUILT,
    ABILITY_LINE,
    ALO_SURFACES,
    IN_WORDS,
    TYPED_AS,
    ARGUMENT_LINE,
    KIND_PATH,
    KIND_NAME,
    KIND_COUNT,
    KIND_CHOICE,
    IT_CHANGES,
    IT_READS,
    CANNOT_BE_UNDONE_ONCE_SENT,
];

/// Why this crate's list would not declare.
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum WordsError {
    /// A word that is not a phrase.
    #[error(transparent)]
    Word(#[from] WordError),
    /// A key the vocabulary already has.
    #[error(transparent)]
    List(#[from] VocabularyError),
}

/// Everything this crate says, as a vocabulary of its own.
///
/// # Errors
/// [`WordsError`], which the list above cannot cause — a test declares it.
pub fn app_words() -> Result<Vocabulary, WordsError> {
    let mut vocabulary = Vocabulary::empty();
    declare_into(&mut vocabulary)?;
    Ok(vocabulary)
}

/// Put everything this crate says into an existing vocabulary.
///
/// # Errors
/// [`WordsError`] for a word that is not a phrase, or a key already declared —
/// nothing is replaced.
pub fn declare_into(vocabulary: &mut Vocabulary) -> Result<(), WordsError> {
    for word in EVERY_WORD.iter() {
        vocabulary.says(word.phrase()?)?;
    }
    Ok(())
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected Err is the failure being reported"
)]
mod tests {
    use super::*;

    /// **Every word declares**, under a key of this crate's own, once.
    #[test]
    fn every_word_declares_under_this_crates_own_keys() {
        let mut vocabulary = Vocabulary::empty();
        declare_into(&mut vocabulary).unwrap();
        let mut keys: Vec<&str> = EVERY_WORD.iter().map(Word::named).collect();
        assert!(keys.iter().all(|key| key.starts_with("app.")), "{keys:?}");
        keys.sort_unstable();
        let before = keys.len();
        keys.dedup();
        assert_eq!(keys.len(), before, "a key is declared twice");
        assert!(matches!(
            declare_into(&mut vocabulary),
            Err(WordsError::List(_))
        ));
    }

    /// **Nothing a person reads names the machinery**: not the bus, not the
    /// socket, not the door's own words.
    #[test]
    fn nothing_a_person_reads_names_the_machinery() {
        for word in EVERY_WORD.iter() {
            let says = word.says().to_lowercase();
            for machinery in ["d-bus", "dbus", "socket", "at-spi", "shell.sock", "stdin"] {
                assert!(!says.contains(machinery), "{}: {says}", word.named());
            }
        }
    }
}
