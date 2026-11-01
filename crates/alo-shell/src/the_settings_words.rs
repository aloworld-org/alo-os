//! What the Settings window says of its own: its title, the categories down its
//! side, each page's heading and introduction, the headings over each group of
//! rows, a switch's two positions, and the sentences that say a choice drawn
//! in the design is not on this computer yet.
//!
//! **Only what no other crate already says.** A row's own name — *on this
//! machine*, *Indigo*, *Open a terminal* — is the owning crate's word, and the
//! window draws it as that crate declares it. These are the words that belong
//! to the window itself, which the owner's redesign of 2026-10-31 brought in:
//! `docs/design/the-settings-window.md` has the frames each comes from.
//!
//! # Sentence case
//!
//! The owner decided on 2026-10-31 that **section headings are in sentence
//! case**: a capital on the first word and on names, nothing else. Every
//! heading here is written that way in English, and each note asks a
//! translator to keep it in the ordinary case of their language.
//!
//! # Nothing pretends
//!
//! Some choices the design draws are not on this computer yet — alo's own
//! service, saying where a provider runs. They are drawn as the design draws
//! them and choosing one says, in [`NOT_ON_THIS_COMPUTER_YET`], that nothing
//! changed. A row that looked like it worked and did nothing would be the
//! machine pretending, which this product never does.

use alo_strings::{Vocabulary, Word};

/// The placeholder for a model's name, in [`YOU_ADDED_THIS`] and its sibling.
pub const THE_MODEL: &str = "model";

/// The placeholder for where a provider says it runs, in [`IT_SAYS_IT_RUNS_IN`].
pub const THE_REGION: &str = "region";

/// The window's title, at the top of the side column.
pub const SETTINGS: Word = Word::saying("settings.title", "Settings").noting(
    "The title of the window where a person changes everything about their computer. One word, \
     the name of the window.",
);

/// The line under the title.
pub const EVERYTHING_IN_ONE_PLACE: Word = Word::saying(
    "settings.everything-in-one-place",
    "Everything about this computer, in one place.",
)
.noting("Under the title of the Settings window. A short sentence saying what the window is for.");

/// The first group of categories: what alo, the assistant, may do.
pub const THE_GROUP_ALO: Word = Word::saying("settings.group.alo", "alo").noting(
    "alo is the assistant built into alo OS, and its name is written in lower case. A small \
     heading over the categories about what alo may do. Keep the name as it is.",
);

/// The second group of categories: the computer itself.
pub const THE_GROUP_THIS_COMPUTER: Word =
    Word::saying("settings.group.this-computer", "This computer").noting(
        "A small heading over the categories about the computer itself — how it looks, its \
         region, its keyboard and its accessibility. Sentence case.",
    );

/// The line at the foot of the side column.
pub const YOURS_TO_CHANGE: Word = Word::saying(
    "settings.yours-to-change",
    "Everything here is yours to change.",
)
.noting(
    "At the foot of the Settings window's side column. A plain statement that the person may \
         change any setting.",
);

/// The category about where questions are answered.
pub const ALO_AND_AI: Word = Word::saying("settings.category.alo-and-ai", "alo and AI").noting(
    "A category in Settings: where the person's questions to alo are answered, by which model. \
     alo is the assistant's name, in lower case. AI is artificial intelligence.",
);

/// The category about the folders alo may read.
pub const FOLDERS_ALO_CAN_USE: Word =
    Word::saying("settings.category.folders", "Folders alo can use").noting(
        "A category in Settings: the folders the person has allowed alo, the assistant, to read, \
         and what alo did with them.",
    );

/// The category about running code.
pub const RUNNING_CODE: Word = Word::saying("settings.category.running-code", "Running code")
    .noting(
        "A category in Settings: how much alo, the assistant, may do when it needs to run a \
         command on the computer.",
    );

/// The category about the terminal.
pub const TERMINAL: Word = Word::saying("settings.category.terminal", "Terminal").noting(
    "A category in Settings: what alo may do in the person's terminal, the window where commands \
     are typed.",
);

/// The category about how the computer looks.
pub const APPEARANCE: Word = Word::saying("settings.category.appearance", "Appearance")
    .noting("A category in Settings: light or dark, the accent colour, and the Dock.");

/// The category about region and time.
pub const REGION_AND_TIME: Word =
    Word::saying("settings.category.region-and-time", "Region and time").noting(
        "A category in Settings: the country or region the computer is in, and its time zone.",
    );

/// The category about the keyboard.
pub const KEYBOARD_AND_SHORTCUTS: Word =
    Word::saying("settings.category.keyboard", "Keyboard and shortcuts").noting(
        "A category in Settings: the keyboard shortcuts, each of which a person can change.",
    );

/// The category about accessibility.
pub const ACCESSIBILITY: Word = Word::saying("settings.category.accessibility", "Accessibility")
    .noting("A category in Settings: making text and the pointer easier to see.");

/// What the alo and AI page is about.
pub const ALO_AND_AI_IS: Word = Word::saying(
    "settings.page.alo-and-ai",
    "Where your questions are answered, and what happens when that place cannot be reached.",
)
.noting("Under the heading of the alo and AI page in Settings.");

/// What the folders page is about.
pub const FOLDERS_ALO_CAN_USE_IS: Word = Word::saying(
    "settings.page.folders",
    "alo can read only the folders you give it. Each one stays on this list until it ends or you take it back.",
)
.noting(
    "Under the heading of the page listing the folders alo, the assistant, may read. alo is its \
     name and stays in lower case at the start of the sentence.",
);

/// What the running code page is about.
pub const RUNNING_CODE_IS: Word = Word::saying(
    "settings.page.running-code",
    "When alo needs to run a command, how much it may do without asking you. Only you can give this; alo cannot give it to itself.",
)
.noting("Under the heading of the Running code page. alo is the assistant's name.");

/// What the terminal page is about.
pub const TERMINAL_IS: Word = Word::saying(
    "settings.page.terminal",
    "Two things alo can do in your terminal. Each one is switched on by you, and says what it changes.",
)
.noting("Under the heading of the Terminal page. alo is the assistant's name.");

/// What the appearance page is about.
pub const APPEARANCE_IS: Word = Word::saying(
    "settings.page.appearance",
    "How this computer looks, and where the Dock sits.",
)
.noting("Under the heading of the Appearance page. The Dock is the bar of applications.");

/// What the region page is about.
pub const REGION_AND_TIME_IS: Word = Word::saying(
    "settings.page.region-and-time",
    "Where this computer is, and the time it keeps.",
)
.noting("Under the heading of the Region and time page.");

/// What the keyboard page is about.
pub const KEYBOARD_AND_SHORTCUTS_IS: Word = Word::saying(
    "settings.page.keyboard",
    "Every shortcut, in one list. Choose one to change it.",
)
.noting("Under the heading of the Keyboard and shortcuts page.");

/// What the accessibility page is about.
pub const ACCESSIBILITY_IS: Word = Word::saying(
    "settings.page.accessibility",
    "Make text and the pointer easier to see. Every setting here can also be reached from the keyboard alone.",
)
.noting("Under the heading of the Accessibility page.");

/// The heading over the four ways a question can be answered.
pub const WHERE_QUESTIONS_ARE_ANSWERED: Word = Word::saying(
    "settings.section.where-answered",
    "Where your questions are answered",
)
.noting("A heading over a group of choices in Settings. Sentence case.");

/// The sentence under the four ways.
pub const EVERY_ANSWER_SAYS_WHERE: Word = Word::saying(
    "settings.section.where-answered.note",
    "Every answer says where it was answered, right beside the answer. Nothing on this page can switch that off.",
)
.noting("Under the list of places a question can be answered.");

/// The heading over what happens when the chosen place is not available.
pub const WHEN_IT_IS_NOT_AVAILABLE: Word = Word::saying(
    "settings.section.not-available",
    "When what you chose is not available",
)
.noting("A heading over one switch in Settings. Sentence case.");

/// The switch that lets alo use another model.
pub const LET_ALO_USE_ANOTHER: Word = Word::saying(
    "settings.another.title",
    "Let alo use another model or provider",
)
.noting(
    "The name of a switch. When it is on, alo may answer a question with a different AI model or \
     service when the one the person chose cannot be reached. alo is the assistant's name.",
);

/// What the switch changes.
pub const WITH_IT_OFF: Word = Word::saying(
    "settings.another.with-it-off",
    "With this off, a question that cannot reach the place you chose is refused, and alo tells you so.",
)
.noting("Under the switch that lets alo use another model. Says what Off means.");

/// The heading of a box saying what a choice risks.
pub const WHAT_IT_RISKS: Word = Word::saying("settings.what-it-risks", "What it risks").noting(
    "The heading of a small box under a choice, saying in plain words what could go wrong if the \
     person turns it on.",
);

/// What letting alo use another model risks.
pub const ANOTHER_RISKS: Word = Word::saying(
    "settings.another.risks",
    "Your question may go somewhere you did not pick. The answer still says where it came from.",
)
.noting("In the box headed 'What it risks', under the switch that lets alo use another model.");

/// A switch that is on.
pub const ON: Word =
    Word::saying("settings.switch.on", "On").noting("Beside a switch that is on. One short word.");

/// A switch that is off.
pub const OFF: Word = Word::saying("settings.switch.off", "Off")
    .noting("Beside a switch that is off. One short word.");

/// A switch the person has never set.
pub const NOT_SET: Word = Word::saying("settings.switch.not-set", "Not set").noting(
    "Beside a switch the person has never turned on or off. It is neither: the person has not \
     answered yet. Two short words.",
);

/// A provider the person added, under its name.
pub const YOU_ADDED_THIS: Word = Word::saying(
    "settings.provider.you-added-this",
    "You added this · model {model}",
)
.noting(
    "Under the name of an AI service the person added. {model} is the model's name as the \
     service writes it. The dot separates two facts.",
);

/// A provider the person added whose key is in the keyring.
pub const YOU_ADDED_THIS_WITH_A_KEY: Word = Word::saying(
    "settings.provider.you-added-this.key",
    "You added this · model {model} · key kept in this computer’s keyring",
)
.noting(
    "Under the name of an AI service the person added, when its secret key is stored in the \
     computer's keyring. {model} is the model's name. The dots separate three facts.",
);

/// A provider that has not said where it runs.
pub const HAS_NOT_SAID_WHERE: Word = Word::saying(
    "settings.provider.has-not-said-where",
    "Has not said where it runs",
)
.noting(
    "A short warning under an AI service: nobody has said in which country or region it \
     processes questions.",
);

/// A provider that has said where it runs.
pub const IT_SAYS_IT_RUNS_IN: Word = Word::saying(
    "settings.provider.says-it-runs-in",
    "Says it runs in {region}",
)
.noting(
    "Under an AI service: where the person who added it said it processes questions. {region} \
     is what they wrote, such as 'the EU'. 'Says' because nothing on this computer can check it.",
);

/// The button that would let a person say where a provider runs.
pub const SAY_WHERE_IT_RUNS: Word =
    Word::saying("settings.provider.say-where", "Say where it runs")
        .noting("A button beside an AI service, to say in which country or region it runs.");

/// alo's own service, by its name.
pub const ALOS_OWN_SERVICE: Word = Word::saying("settings.provider.alo", "alo").noting(
    "The name of alo's own AI service, shown among the services a person can choose. It is the \
     product's name in lower case; keep it as it is.",
);

/// What alo's own service is.
pub const ALOS_OWN_SERVICE_IS: Word = Word::saying(
    "settings.provider.alo.is",
    "alo’s own service. Nothing to set up; it starts on free credits and asks before any subscription.",
)
.noting("Under the name of alo's own AI service in Settings.");

/// What choosing something not built yet says.
pub const NOT_ON_THIS_COMPUTER_YET: Word = Word::saying(
    "settings.not-yet",
    "This is not on this computer yet, so nothing was changed.",
)
.noting(
    "Shown when a person chooses something Settings draws that has not been built yet. Plain and \
     honest: nothing happened.",
);

/// Why a way of answering has nothing under it: no model brought.
pub const NO_MODEL_HERE_YET: Word = Word::saying(
    "settings.way.no-model-yet",
    "No model is on this computer yet, so there is nothing to choose here.",
)
.noting(
    "Shown when a person chooses 'On this machine' and no AI model has been put on the computer.",
);

/// Why a way of answering has nothing under it: nothing paired.
pub const NOTHING_PAIRED_YET: Word = Word::saying(
    "settings.way.nothing-paired-yet",
    "No other computer is paired with this one yet, so there is nothing to choose here.",
)
.noting(
    "Shown when a person chooses 'On a machine on your network' and no other computer has been \
     paired with this one.",
);

/// Why a way of answering has nothing under it: no provider added.
pub const NO_PROVIDER_YET: Word = Word::saying(
    "settings.way.no-provider-yet",
    "No provider has been added yet. Add one below.",
)
.noting(
    "Shown when a person chooses 'With a provider you add' before adding one. 'Below' is the row \
     'Add a provider' under it.",
);

/// The heading over the folders alo may read.
pub const GIVEN_TO_ALO: Word = Word::saying("settings.section.given", "Given to alo")
    .noting("A heading over the list of folders the person gave alo to read. Sentence case.");

/// The heading over the record.
pub const WHAT_ALO_DID: Word = Word::saying("settings.section.what-alo-did", "What alo did")
    .noting("A heading over the row that opens the record of what alo did. Sentence case.");

/// The heading over the machines paired with this one.
pub const PAIRED_MACHINES: Word = Word::saying(
    "settings.section.paired",
    "Computers paired with this one",
)
.noting(
    "A heading over the list of other computers paired with this one, each of which can be taken \
     back. Sentence case.",
);

/// The heading over the three levels of running code.
pub const ALO_RUNS_CODE: Word = Word::saying("settings.section.alo-runs-code", "alo runs code")
    .noting("A heading over three choices of how alo runs commands. alo is the assistant's name.");

/// The heading over what stays on at every level.
pub const STAYS_ON: Word = Word::saying("settings.section.stays-on", "Stays on at every level")
    .noting("A heading over three protections that no choice switches off. Sentence case.");

/// The heading over the terminal's two switches.
pub const IN_THE_TERMINAL: Word =
    Word::saying("settings.section.in-the-terminal", "In the terminal")
        .noting("A heading over two things alo can do in the terminal. Sentence case.");

/// The heading over the accent colours.
pub const LOOK: Word = Word::saying("settings.section.look", "Look")
    .noting("A heading over how the computer looks: light or dark, and the accent colour.");

/// The heading over the Dock's settings.
pub const THE_DOCK: Word = Word::saying("settings.section.the-dock", "The Dock")
    .noting("A heading over the Dock's settings. The Dock is the bar of applications.");

/// The heading over the region dates are written like.
pub const WHERE_THIS_COMPUTER_IS: Word = Word::saying(
    "settings.section.where-this-computer-is",
    "Where this computer is",
)
.noting("A heading over the country or region the computer is in. Sentence case.");

/// The heading over the time zone.
pub const TIME_ZONE: Word = Word::saying("settings.section.time-zone", "Time zone")
    .noting("A heading over the time zone the computer keeps. Sentence case.");

/// The heading over the shortcuts.
pub const SHORTCUTS: Word = Word::saying("settings.section.shortcuts", "Shortcuts")
    .noting("A heading over the list of keyboard shortcuts.");

/// Take back.
pub const TAKE_BACK: Word = Word::saying("settings.take-back", "Take back")
    .noting("A button beside a folder alo may read, or a computer paired with this one, that takes the permission back at once.");

/// Taking a folder back stops it at once: the next thing asked under it is refused.
pub const TAKING_BACK_STOPS_IT: Word = Word::saying("settings.folders.taking-back", "Taking a folder back stops it at once: the next thing asked under it is refused. It does not undo what was already done — the record and undo are for that.")
    .noting("Under the list of folders alo may read. The record is the list of what alo did; undo puts files back.");

/// You choose the folder and how long, in the same step.
pub const YOU_CHOOSE_FOLDER_AND_HOW_LONG: Word = Word::saying(
    "settings.folders.you-choose",
    "You choose the folder and how long, in the same step.",
)
.noting("Beside the button that gives alo a folder: what happens when it is pressed.");

/// Every action and every refusal, in order, with what it changed.
pub const EVERY_ACTION_AND_REFUSAL: Word = Word::saying(
    "settings.record.is",
    "Every action and every refusal, in order, with what it changed.",
)
.noting("Under the row that opens the record of what alo did.");

/// The system locks the box.
pub const SEALED_BOX_IS: Word = Word::saying(
    "settings.running.sealed-box.is",
    "The system locks the box. What runs inside it sees this project and nothing else.",
)
.noting("Under the choice that runs commands in a sealed box.");

/// Nothing runs until you have read it and said yes.
pub const ASKING_IS: Word = Word::saying(
    "settings.running.asking.is",
    "Nothing runs until you have read it and said yes. One yes runs it once.",
)
.noting("Under the choice that shows each command and waits for the person.");

/// Default.
pub const DEFAULT: Word = Word::saying("settings.default", "Default")
    .noting("A small tag beside the choice a fresh computer starts with. One word.");

/// Always on.
pub const ALWAYS_ON: Word = Word::saying("settings.always-on", "Always on")
    .noting("At the end of a protection that no setting switches off. Two words.");

/// These protect you without stopping anything, so no level turns them off.
pub const THEY_PROTECT_YOU: Word = Word::saying(
    "settings.running.stays-on.note",
    "These protect you without stopping anything, so no level turns them off.",
)
.noting("Under the three protections that stay on whatever level of running code is chosen.");

/// Related.
pub const RELATED: Word = Word::saying("settings.section.related", "Related")
    .noting("A heading over a row that opens a related page of Settings.");

/// How alo runs code.
pub const HOW_ALO_RUNS_CODE: Word =
    Word::saying("settings.terminal.how-code-runs", "How alo runs code").noting(
        "A row on the Terminal page that opens the Running code page. alo is the assistant.",
    );

/// In a sealed box, after asking you, or straight away.
pub const HOW_ALO_RUNS_CODE_IS: Word = Word::saying(
    "settings.terminal.how-code-runs.is",
    "In a sealed box, after asking you, or straight away. Set in Running code.",
)
.noting("Under the row that opens the Running code page. Running code is that page's name.");

/// Accent colour.
pub const ACCENT_COLOUR: Word = Word::saying("settings.appearance.accent", "Accent colour")
    .noting("The name of the setting for the colour that selections and switches are drawn in.");

/// Each colour has a version for light and for dark, so it reads properly either way.
pub const ACCENT_COLOUR_IS: Word = Word::saying(
    "settings.appearance.accent.is",
    "Each colour has a version for light and for dark, so it reads properly either way.",
)
.noting("Under Accent colour.");

/// The whole of alo OS follows the accent: selections, switches and choices.
pub const THE_ACCENT_NOTE: Word = Word::saying("settings.appearance.accent.note", "The whole of alo OS follows the accent: selections, switches and choices. Deep teal is not offered: it means alo is acting, and nothing else.")
    .noting("Under the accent colours. Deep teal is the colour reserved for alo, the assistant, acting.");

/// Where the Dock sits.
pub const WHERE_THE_DOCK_SITS: Word = Word::saying("settings.dock.where", "Where the Dock sits")
    .noting("The name of the setting for which edge of the screen the Dock is on. The Dock is the bar of applications.");

/// Bottom.
pub const BOTTOM: Word = Word::saying("settings.dock.bottom", "Bottom")
    .noting("One of four short choices for the edge of the screen the Dock sits on.");

/// Left.
pub const LEFT: Word = Word::saying("settings.dock.left", "Left")
    .noting("One of four short choices for the edge of the screen the Dock sits on.");

/// Right.
pub const RIGHT: Word = Word::saying("settings.dock.right", "Right")
    .noting("One of four short choices for the edge of the screen the Dock sits on.");

/// Top.
pub const TOP: Word = Word::saying("settings.dock.top", "Top")
    .noting("One of four short choices for the edge of the screen the Dock sits on.");

/// The Dock keeps its place on its edge of the screen.
pub const ALWAYS_SHOWN_IS: Word = Word::saying(
    "settings.dock.always-shown.is",
    "The Dock keeps its place on its edge of the screen.",
)
.noting("Under the choice Always shown, for the Dock.");

/// The Dock steps aside and comes back when you reach for it.
pub const GIVES_WAY_IS: Word = Word::saying(
    "settings.dock.gives-way.is",
    "The Dock steps aside and comes back when you reach for it.",
)
.noting("Under the choice for the Dock that gives way when a window needs the room.");

/// The heading over how windows look.
pub const WINDOWS: Word = Word::saying("settings.windows", "Windows").noting(
    "A heading over the settings for how the windows on the screen look. Sentence case. Not the \
     name of any product: windows as in the rectangles applications are shown in.",
);

/// Show a window's title only while the pointer is on it.
pub const SHOW_THE_TITLE_ONLY_WHEN_POINTED_AT: Word = Word::saying(
    "settings.windows.title-when-pointed-at",
    "Show the title only when I point at it",
)
.noting(
    "A switch. Off, the name of every window is always shown on the strip across its top. On, \
     the strip appears only while the pointer or the keyboard is on that edge of the window. \
     Spoken by the person, so first person.",
);

/// What the switch above does.
pub const SHOW_THE_TITLE_ONLY_WHEN_POINTED_AT_IS: Word = Word::saying(
    "settings.windows.title-when-pointed-at.is",
    "Off, every window shows its name and its application's icon along its top edge.",
)
.noting(
    "Under the switch Show the title only when I point at it. Says what is on the screen while \
     the switch is off, which is how this computer comes.",
);

/// Choose a country or region from the list.
pub const CHOOSE_A_COUNTRY: Word = Word::saying(
    "settings.region.choose",
    "Choose a country or region from the list.",
)
.noting(
    "Under Where this computer is, beside the button that opens the list of countries and regions.",
);

/// All countries and regions.
pub const ALL_COUNTRIES: Word = Word::saying("settings.region.all", "All countries and regions")
    .noting("A small heading over the full list of countries and regions in the list a person chooses from.");

/// All time zones.
pub const ALL_TIME_ZONES: Word = Word::saying("settings.zone.all", "All time zones")
    .noting("A small heading over the full list of time zones a person chooses from.");

/// How many countries, regions or time zones match what was typed.
///
/// **Counted**, because how a language counts is that language's business:
/// *one match* is not English's `{count} matches` with a one in it.
pub const MATCHES: (&str, &str, &str, &str, &str) = (
    "settings.picker.matches",
    "count",
    "1 match",
    "{count} matches",
    "A small heading over the countries, regions or time zones whose names contain what the      person typed. {count} is how many.",
);

/// The letter the text size slider shows small and large at its two ends.
///
/// **A letter rather than a word**, so not translated: it is a picture of
/// type at two sizes, as the design draws it (`551:1117`, `551:1122`), and an
/// *A* reads as *text* in every script this machine draws.
pub(crate) const THE_LETTER: &str = "A";

/// A text size as a percentage, in the person's language.
pub fn as_a_percentage(strings: &alo_strings::Strings, percent: u16) -> String {
    strings
        .say(
            &AS_A_PERCENTAGE.key(),
            &alo_strings::Filling::of("percent", percent.to_string()),
        )
        .into_text()
}

/// *N matches*, in the person's language and counted the way it counts.
pub fn how_many_match(strings: &alo_strings::Strings, how_many: usize) -> String {
    let (key, ..) = MATCHES;
    alo_strings::Key::named(key).map_or_else(
        |_| how_many.to_string(),
        |key| {
            strings
                .count(
                    &key,
                    &alo_strings::Counting::of(u64::try_from(how_many).unwrap_or(u64::MAX)),
                    &alo_strings::Filling::nothing(),
                )
                .into_text()
        },
    )
}

/// Light and dark.
pub const LIGHT_AND_DARK: Word = Word::saying("settings.appearance.light-and-dark", "Light and dark")
    .noting("The name of the setting for whether the computer is drawn light, dark, or light by day and dark by evening.");

/// Following the time of day turns dark in the evening and light again in the morning.
pub const LIGHT_AND_DARK_IS: Word = Word::saying(
    "settings.appearance.light-and-dark.is",
    "Following the time of day turns dark in the evening and light again in the morning.",
)
.noting("Under Light and dark: what the third choice does.");

/// Follow the time of day.
pub const FOLLOW_THE_TIME_OF_DAY: Word =
    Word::saying("settings.appearance.follow", "Follow the time of day").noting(
        "One of three short choices beside Light and Dark: light by day, dark in the evening.",
    );

/// Text.
pub const TEXT: Word =
    Word::saying("settings.section.text", "Text").noting("A heading over the text size setting.");

/// Text size.
pub const TEXT_SIZE: Word = Word::saying("settings.access.text-size", "Text size")
    .noting("The name of the setting for how large text is drawn everywhere.");

/// Makes text larger everywhere.
pub const TEXT_SIZE_IS: Word = Word::saying(
    "settings.access.text-size.is",
    "Makes text larger everywhere. Layouts make room for it rather than cutting it off.",
)
.noting("Under Text size.");

/// Larger text makes controls larger with it, so they stay easy to reach.
pub const LARGER_TEXT_NOTE: Word = Word::saying(
    "settings.access.text-size.note",
    "Larger text makes controls larger with it, so they stay easy to reach.",
)
.noting("Under the text size setting.");

/// Pointer.
pub const POINTER: Word = Word::saying("settings.section.pointer", "Pointer")
    .noting("A heading over the pointer size setting. The pointer is the arrow the mouse moves.");

/// Pointer size.
pub const POINTER_SIZE: Word = Word::saying("settings.access.pointer-size", "Pointer size")
    .noting("The name of the setting for how large the pointer is drawn.");

/// Large.
pub const LARGE: Word = Word::saying("settings.access.large", "Large")
    .noting("One of three short choices for the pointer's size.");

/// Larger.
pub const LARGER: Word = Word::saying("settings.access.larger", "Larger")
    .noting("One of three short choices for the pointer's size, larger than Large.");

/// A text size, as a percentage.
pub const AS_A_PERCENTAGE: Word = Word::saying("settings.access.percent", "{percent}%").noting(
    "Beside the text size slider: the size as a percentage of the ordinary size. {percent} is      the number. Write a percentage the way your language does, with or without a space.",
);

/// Every word this window says of its own.
///
/// The length is written out so that adding a word above and forgetting it
/// here does not compile — a word declared and left out of this list reaches a
/// person as its own key.
pub const THE_WINDOWS_WORDS: [Word; 93] = [
    SETTINGS,
    EVERYTHING_IN_ONE_PLACE,
    THE_GROUP_ALO,
    THE_GROUP_THIS_COMPUTER,
    YOURS_TO_CHANGE,
    ALO_AND_AI,
    FOLDERS_ALO_CAN_USE,
    RUNNING_CODE,
    TERMINAL,
    APPEARANCE,
    REGION_AND_TIME,
    KEYBOARD_AND_SHORTCUTS,
    ACCESSIBILITY,
    ALO_AND_AI_IS,
    FOLDERS_ALO_CAN_USE_IS,
    RUNNING_CODE_IS,
    TERMINAL_IS,
    APPEARANCE_IS,
    REGION_AND_TIME_IS,
    KEYBOARD_AND_SHORTCUTS_IS,
    ACCESSIBILITY_IS,
    WHERE_QUESTIONS_ARE_ANSWERED,
    EVERY_ANSWER_SAYS_WHERE,
    WHEN_IT_IS_NOT_AVAILABLE,
    LET_ALO_USE_ANOTHER,
    WITH_IT_OFF,
    WHAT_IT_RISKS,
    ANOTHER_RISKS,
    ON,
    OFF,
    NOT_SET,
    YOU_ADDED_THIS,
    YOU_ADDED_THIS_WITH_A_KEY,
    HAS_NOT_SAID_WHERE,
    IT_SAYS_IT_RUNS_IN,
    SAY_WHERE_IT_RUNS,
    ALOS_OWN_SERVICE,
    ALOS_OWN_SERVICE_IS,
    NOT_ON_THIS_COMPUTER_YET,
    NO_MODEL_HERE_YET,
    NOTHING_PAIRED_YET,
    NO_PROVIDER_YET,
    GIVEN_TO_ALO,
    WHAT_ALO_DID,
    PAIRED_MACHINES,
    ALO_RUNS_CODE,
    STAYS_ON,
    IN_THE_TERMINAL,
    LOOK,
    THE_DOCK,
    WHERE_THIS_COMPUTER_IS,
    TIME_ZONE,
    SHORTCUTS,
    TAKE_BACK,
    TAKING_BACK_STOPS_IT,
    YOU_CHOOSE_FOLDER_AND_HOW_LONG,
    EVERY_ACTION_AND_REFUSAL,
    SEALED_BOX_IS,
    ASKING_IS,
    DEFAULT,
    ALWAYS_ON,
    THEY_PROTECT_YOU,
    RELATED,
    HOW_ALO_RUNS_CODE,
    HOW_ALO_RUNS_CODE_IS,
    ACCENT_COLOUR,
    ACCENT_COLOUR_IS,
    THE_ACCENT_NOTE,
    WHERE_THE_DOCK_SITS,
    BOTTOM,
    LEFT,
    RIGHT,
    TOP,
    ALWAYS_SHOWN_IS,
    GIVES_WAY_IS,
    WINDOWS,
    SHOW_THE_TITLE_ONLY_WHEN_POINTED_AT,
    SHOW_THE_TITLE_ONLY_WHEN_POINTED_AT_IS,
    CHOOSE_A_COUNTRY,
    ALL_COUNTRIES,
    ALL_TIME_ZONES,
    LIGHT_AND_DARK,
    LIGHT_AND_DARK_IS,
    FOLLOW_THE_TIME_OF_DAY,
    TEXT,
    TEXT_SIZE,
    TEXT_SIZE_IS,
    LARGER_TEXT_NOTE,
    POINTER,
    POINTER_SIZE,
    LARGE,
    LARGER,
    AS_A_PERCENTAGE,
];

/// Why this window's words could not be declared.
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum TheSettingsWordsError {
    /// A word that is not a phrase.
    #[error(transparent)]
    Word(#[from] alo_strings::WordError),
    /// A key the vocabulary already has.
    #[error(transparent)]
    List(#[from] alo_strings::VocabularyError),
    /// A counted sentence that does not say its number.
    #[error(transparent)]
    Counted(#[from] alo_strings::PluralError),
    /// A key that is not a key.
    #[error(transparent)]
    Key(#[from] alo_strings::KeyError),
}

/// Add this window's wording atomically, before any translation is loaded.
///
/// # Errors
/// [`TheSettingsWordsError`] for a malformed phrase or a key already taken.
pub fn declare_the_settings_words(
    vocabulary: &mut Vocabulary,
) -> Result<(), TheSettingsWordsError> {
    // **Prepared and swapped**, which is `declare_the_pickers_words`' shape: a
    // declaration that failed halfway would leave a vocabulary holding some of
    // these and not others.
    let mut prepared = vocabulary.clone();
    for word in THE_WINDOWS_WORDS {
        prepared.says(word.phrase()?)?;
    }
    let (key, number, one, other, note) = MATCHES;
    prepared.counts(
        alo_strings::Plural::counting(alo_strings::Key::named(key)?, number, one, other)?
            .noting(note)?,
    )?;
    *vocabulary = prepared;
    Ok(())
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected Err is the failure being reported"
)]
mod tests {
    use super::*;

    /// Every one of them declares, which is what the shell's boot relies on.
    #[test]
    fn every_word_here_can_be_declared() {
        let mut vocabulary = Vocabulary::empty();
        declare_the_settings_words(&mut vocabulary).unwrap();
        for word in THE_WINDOWS_WORDS {
            assert!(vocabulary.phrase(&word.key()).is_some(), "{word:?}");
        }
    }

    /// No key here collides with another in this file.
    #[test]
    fn no_two_words_here_share_a_key() {
        let mut keys: Vec<String> = THE_WINDOWS_WORDS
            .iter()
            .map(|it| it.key().to_string())
            .collect();
        keys.sort_unstable();
        keys.dedup();
        assert_eq!(keys.len(), THE_WINDOWS_WORDS.len());
    }

    /// The two sentences naming a model, and the one naming a region, keep
    /// their gaps: a translator who drops one leaves a person reading a
    /// sentence about nothing.
    #[test]
    fn the_sentences_with_a_gap_keep_it() {
        for (word, gap) in [
            (YOU_ADDED_THIS, THE_MODEL),
            (YOU_ADDED_THIS_WITH_A_KEY, THE_MODEL),
            (IT_SAYS_IT_RUNS_IN, THE_REGION),
        ] {
            assert!(
                word.says().contains(&format!("{{{gap}}}")),
                "{}",
                word.says()
            );
        }
    }
}
