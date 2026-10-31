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
pub const YOURS_TO_CHANGE: Word =
    Word::saying("settings.yours-to-change", "Everything here is yours to change.").noting(
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
pub const APPEARANCE: Word = Word::saying("settings.category.appearance", "Appearance").noting(
    "A category in Settings: light or dark, the accent colour, and the Dock.",
);

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

/// What the accessibility page says while nothing on it can be changed from
/// this window yet.
pub const NOTHING_HERE_YET: Word = Word::saying(
    "settings.page.nothing-here-yet",
    "Nothing on this page can be changed from this window yet.",
)
.noting(
    "Shown on a page of Settings whose controls are not on this computer yet. Plain and \
     factual: nothing is broken, the controls have not been built.",
);

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
pub const ON: Word = Word::saying("settings.switch.on", "On")
    .noting("Beside a switch that is on. One short word.");

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
pub const SAY_WHERE_IT_RUNS: Word = Word::saying("settings.provider.say-where", "Say where it runs")
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
.noting("Shown when a person chooses 'On this machine' and no AI model has been put on the computer.");

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
pub const IN_THE_TERMINAL: Word = Word::saying("settings.section.in-the-terminal", "In the terminal")
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

/// Every word this window says of its own.
///
/// The length is written out so that adding a word above and forgetting it
/// here does not compile — a word declared and left out of this list reaches a
/// person as its own key.
pub const THE_SETTINGS_WORDS: [Word; 54] = [
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
    NOTHING_HERE_YET,
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
    for word in THE_SETTINGS_WORDS {
        prepared.says(word.phrase()?)?;
    }
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
        for word in THE_SETTINGS_WORDS {
            assert!(vocabulary.phrase(&word.key()).is_some(), "{word:?}");
        }
    }

    /// No key here collides with another in this file.
    #[test]
    fn no_two_words_here_share_a_key() {
        let mut keys: Vec<String> = THE_SETTINGS_WORDS
            .iter()
            .map(|it| it.key().to_string())
            .collect();
        keys.sort_unstable();
        keys.dedup();
        assert_eq!(keys.len(), THE_SETTINGS_WORDS.len());
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
