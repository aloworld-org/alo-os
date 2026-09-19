//! Everything alo OS can say, gathered into one vocabulary.
//!
//! Every crate that says something to a person declares its own strings — the
//! key, the English beside it, and the note a translator cannot work without —
//! and hands them over through a `declare_into` of the same shape. This file is
//! the one place that calls all of them.
//!
//! # Why one vocabulary and not one per process
//!
//! Because a translation is **checked against the vocabulary it is loaded
//! into**, and `alo_strings::Amiss::NotSaidHere` is what a key nothing declares
//! gets. A process that declared only its own strings would therefore look at a
//! translator's correct line for another part of the system and call it a
//! mistake — the shell reading the daemon's refusals as wrong, the daemon
//! reading the shortcuts panel's rows as wrong, and each of them able to show
//! its own half of a file somebody wrote as one.
//!
//! So a translation file covers the machine, this is what the machine says, and
//! the crates a particular process happens to link are not the question. It
//! costs this crate a dependency on every crate that has a word in it, and
//! their dependencies with them; `crate` documentation says why that is the
//! cheaper of the two mistakes.
//!
//! # `alo-agentd` is not here, and the reason is not that it is a daemon
//!
//! It is Linux, and every module in it is compiled out anywhere else. A
//! vocabulary assembled here would then hold three fewer strings on a host with
//! no daemon than on a machine with one, so one host would refuse a translation
//! file the other accepted — the exact failure this file exists to prevent, in
//! its platform-shaped form. The daemon declares its own three on top of this
//! one, and [`crate::loading`] is why leaving a line out is survivable rather
//! than the end of somebody's language.
//!
//! # What a failure here means
//!
//! Two crates claiming one key, or a crate whose own list does not declare. It
//! is alo OS's own bug and it cannot be fixed on the machine it happens on, so
//! it keeps its English and its `Display`: whoever reads it is whoever is
//! fixing the list, which is `alo_shortcuts::DefaultsError`'s reader one layer
//! up. It is also caught by the test at the bottom of this file rather than by
//! somebody's machine.

use std::fmt::Display;

use alo_strings::Vocabulary;

/// Why alo OS's own list of what it can say could not be put together.
///
/// Not a refusal anybody reads in their own language, and deliberately: the
/// thing that has gone wrong is the vocabulary, so there is nothing to ask.
#[derive(Debug, Clone, thiserror::Error, PartialEq, Eq)]
#[error("alo OS's own words are wrong: {list} could not be declared — {why}")]
pub struct NotCollected {
    /// Which crate's list.
    list: &'static str,
    /// What that crate's own declaration said about it.
    why: String,
}

impl NotCollected {
    /// Which crate's list could not be declared.
    #[must_use]
    pub fn list(&self) -> &'static str {
        self.list
    }

    /// What that crate said was wrong with it.
    #[must_use]
    pub fn why(&self) -> &str {
        &self.why
    }
}

/// Every crate whose words are collected here, in the order they are declared.
///
/// The length follows the entries rather than a number maintained beside them.
/// The tests compare these names with the actual declarations, so adding a
/// vocabulary must still include it in the collection. The workspace check in
/// `alo-collected` also names any crate omitted from both lists.
pub const EVERY_LIST: &[&str] = &[
    "alo-adapting",
    "alo-access",
    "alo-accounts",
    "alo-adapters",
    "alo-answering",
    "alo-appearance",
    "alo-applications",
    "alo-approving",
    "alo-asking",
    "alo-bluetooth",
    "alo-cameras",
    "alo-capability",
    "alo-capturing",
    "alo-changing",
    "alo-changing-network",
    "alo-changing-printers",
    "alo-choosing",
    "alo-clipboard",
    "alo-context",
    "alo-converting",
    "alo-corridor",
    "alo-desktops",
    "alo-displays",
    "alo-dividing",
    "alo-dock",
    "alo-egress",
    "alo-files",
    "alo-finding",
    "alo-granted",
    "alo-greeting",
    "alo-handing",
    "alo-in-use",
    "alo-indicator",
    "alo-installer",
    "alo-installing",
    "alo-keeping",
    "alo-keeping-up",
    "alo-keyboards",
    "alo-leaving",
    "alo-locking",
    "alo-measuring",
    "alo-menus",
    "alo-models",
    "alo-nearby",
    "alo-opening",
    "alo-overlay",
    "alo-picking",
    "alo-portals",
    "alo-power",
    "alo-printing",
    "alo-protocol",
    "alo-proxy",
    "alo-recounting",
    "alo-sessiond",
    "alo-setting-up",
    "alo-shortcuts",
    "alo-sleeping",
    "alo-software",
    "alo-sound",
    "alo-telling",
    "alo-turn",
];

/// Every crate that declares words and is deliberately **not** collected here,
/// with the reason it is not.
///
/// One, and it is `alo-agentd` — the argument is in this file's own
/// documentation and is restated here because this is the list a check can read.
/// An exception with no reason beside it is indistinguishable from the failure
/// the check exists for, which is a crate whose words nothing collects: the
/// difference between a documented exception and silence is entirely the
/// sentence.
///
/// `crates/alo-collected` holds both lists to this workspace's own member list:
/// a crate that declares words and is on neither of them is a finding naming it,
/// and so is a name on either of them that no longer declares anything.
pub const DELIBERATELY_APART: [(&str, &str); 1] = [(
    "alo-agentd",
    "it is Linux, and every module in it is compiled out anywhere else, so a \
     vocabulary assembled here would hold three fewer strings on a host with no \
     daemon than on a machine with one — and one host would then refuse a \
     translation file the other accepted. It declares its own three on top of \
     this one, which is the rule docs/contracts/translations.md states for \
     anything that assembles a vocabulary.",
)];

/// Everything alo OS can say, in one vocabulary.
///
/// What a process adds to this is whatever it says that the rest of the machine
/// does not — today that is `alo-agentd`'s three, and nothing else in this
/// workspace has any.
///
/// # Errors
///
/// [`NotCollected`], naming the crate whose list would not declare. It cannot
/// happen on a machine that shipped: the test below runs it.
pub fn everything_this_machine_can_say() -> Result<Vocabulary, NotCollected> {
    let mut vocabulary = Vocabulary::empty();
    declare(
        &mut vocabulary,
        "alo-adapting",
        alo_adapting::words::declare_into,
    )?;
    declare(
        &mut vocabulary,
        "alo-access",
        alo_access::words::declare_into,
    )?;
    declare(&mut vocabulary, "alo-accounts", alo_accounts::declare_into)?;
    declare(
        &mut vocabulary,
        "alo-adapters",
        alo_adapters::words::declare_into,
    )?;
    declare(
        &mut vocabulary,
        "alo-answering",
        alo_answering::declare_into,
    )?;
    declare(
        &mut vocabulary,
        "alo-appearance",
        alo_appearance::declare_into,
    )?;
    // `alo-applications` and `alo-files` are the two crates that declare verbs
    // as well as words, and a crate root holds one `declare_into`. Theirs is
    // the verbs' — the older meaning and the one their callers use — so the
    // words are named through the module they are in rather than by moving
    // somebody else's public surface out from under them.
    declare(
        &mut vocabulary,
        "alo-applications",
        alo_applications::words::declare_into,
    )?;
    declare(
        &mut vocabulary,
        "alo-approving",
        alo_approving::declare_into,
    )?;
    declare(&mut vocabulary, "alo-asking", alo_asking::declare_into)?;
    declare(
        &mut vocabulary,
        "alo-bluetooth",
        alo_bluetooth::declare_into,
    )?;
    declare(&mut vocabulary, "alo-cameras", alo_cameras::declare_into)?;
    declare(
        &mut vocabulary,
        "alo-capability",
        alo_capability::declare_into,
    )?;
    declare(
        &mut vocabulary,
        "alo-capturing",
        alo_capturing::declare_into,
    )?;
    declare(&mut vocabulary, "alo-changing", alo_changing::declare_into)?;
    declare(
        &mut vocabulary,
        "alo-changing-network",
        alo_changing_network::declare_into,
    )?;
    declare(
        &mut vocabulary,
        "alo-changing-printers",
        alo_changing_printers::declare_into,
    )?;
    declare(&mut vocabulary, "alo-choosing", alo_choosing::declare_into)?;
    declare(
        &mut vocabulary,
        "alo-clipboard",
        alo_clipboard::declare_into,
    )?;
    declare(&mut vocabulary, "alo-context", alo_context::declare_into)?;
    declare(
        &mut vocabulary,
        "alo-converting",
        alo_converting::declare_into,
    )?;
    declare(&mut vocabulary, "alo-corridor", alo_corridor::declare_into)?;
    declare(&mut vocabulary, "alo-desktops", alo_desktops::declare_into)?;
    declare(&mut vocabulary, "alo-displays", alo_displays::declare_into)?;
    declare(&mut vocabulary, "alo-dividing", alo_dividing::declare_into)?;
    declare(&mut vocabulary, "alo-dock", alo_dock::declare_into)?;
    declare(&mut vocabulary, "alo-egress", alo_egress::declare_into)?;
    declare(&mut vocabulary, "alo-files", alo_files::words::declare_into)?;
    declare(&mut vocabulary, "alo-finding", alo_finding::declare_into)?;
    declare(&mut vocabulary, "alo-granted", alo_granted::declare_into)?;
    declare(&mut vocabulary, "alo-greeting", alo_greeting::declare_into)?;
    declare(&mut vocabulary, "alo-handing", alo_handing::declare_into)?;
    declare(&mut vocabulary, "alo-in-use", alo_in_use::declare_into)?;
    declare(
        &mut vocabulary,
        "alo-indicator",
        alo_indicator::declare_into,
    )?;
    declare(
        &mut vocabulary,
        "alo-installer",
        alo_installer::declare_into,
    )?;
    declare(
        &mut vocabulary,
        "alo-installing",
        alo_installing::declare_into,
    )?;
    declare(&mut vocabulary, "alo-keeping", alo_keeping::declare_into)?;
    declare(
        &mut vocabulary,
        "alo-keeping-up",
        alo_keeping_up::declare_into,
    )?;
    declare(
        &mut vocabulary,
        "alo-keyboards",
        alo_keyboards::words::declare_into,
    )?;
    declare(&mut vocabulary, "alo-leaving", alo_leaving::declare_into)?;
    declare(&mut vocabulary, "alo-locking", alo_locking::declare_into)?;
    declare(
        &mut vocabulary,
        "alo-measuring",
        alo_measuring::declare_into,
    )?;
    declare(&mut vocabulary, "alo-menus", alo_menus::declare_into)?;
    declare(&mut vocabulary, "alo-models", alo_models::declare_into)?;
    declare(
        &mut vocabulary,
        "alo-nearby",
        alo_nearby::words::declare_into,
    )?;
    declare(&mut vocabulary, "alo-opening", alo_opening::declare_into)?;
    declare(&mut vocabulary, "alo-overlay", alo_overlay::declare_into)?;
    declare(&mut vocabulary, "alo-picking", alo_picking::declare_into)?;
    declare(&mut vocabulary, "alo-portals", alo_portals::declare_into)?;
    declare(&mut vocabulary, "alo-power", alo_power::declare_into)?;
    declare(&mut vocabulary, "alo-printing", alo_printing::declare_into)?;
    declare(&mut vocabulary, "alo-protocol", alo_protocol::declare_into)?;
    declare(&mut vocabulary, "alo-proxy", alo_proxy::declare_into)?;
    declare(
        &mut vocabulary,
        "alo-recounting",
        alo_recounting::declare_into,
    )?;
    declare(&mut vocabulary, "alo-sessiond", alo_sessiond::declare_into)?;
    declare(
        &mut vocabulary,
        "alo-setting-up",
        alo_setting_up::declare_into,
    )?;
    declare(
        &mut vocabulary,
        "alo-shortcuts",
        alo_shortcuts::declare_into,
    )?;
    declare(&mut vocabulary, "alo-sleeping", alo_sleeping::declare_into)?;
    declare(&mut vocabulary, "alo-software", alo_software::declare_into)?;
    declare(&mut vocabulary, "alo-sound", alo_sound::declare_into)?;
    declare(&mut vocabulary, "alo-telling", alo_telling::declare_into)?;
    declare(&mut vocabulary, "alo-turn", alo_turn::declare_into)?;
    Ok(vocabulary)
}

/// One crate's list, into the vocabulary being built.
///
/// Every crate's `declare_into` has the same shape and its own error type, each
/// of which is that crate's English for whoever is fixing the list. What is
/// kept here is the sentence rather than the type: a fifteen-armed enum would
/// say nothing this does not, and the reader is looking for which crate and
/// which key.
fn declare<Why: Display>(
    vocabulary: &mut Vocabulary,
    list: &'static str,
    declaring: impl FnOnce(&mut Vocabulary) -> Result<(), Why>,
) -> Result<(), NotCollected> {
    declaring(vocabulary).map_err(|why| NotCollected {
        list,
        why: why.to_string(),
    })
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use alo_strings::Key;

    /// One string each crate declares, which is how the test below proves that
    /// crate was reached rather than that the total came out right.
    const ONE_STRING_EACH: &[(&str, &str)] = &[
        ("alo-adapting", "adapting.deleting-this-adapter"),
        ("alo-access", "access.screen-reader"),
        ("alo-accounts", "accounts.not-signed-in"),
        ("alo-adapters", "adapters.not-carried-out.not-there"),
        ("alo-answering", "answering.wrong.nothing-answered"),
        ("alo-appearance", "appearance.token.navy"),
        ("alo-applications", "applications.not-installed"),
        ("alo-approving", "approving.nothing-to-answer"),
        ("alo-asking", "asking.question.nothing"),
        ("alo-bluetooth", "bluetooth.radio-off"),
        ("alo-cameras", "cameras.camera-is-off"),
        ("alo-capability", "capability.grant.anonymous"),
        ("alo-capturing", "capturing.the-lock-screen"),
        ("alo-changing", "changing.not-kept"),
        (
            "alo-changing-network",
            "changing-network.verb.loses-its-connection",
        ),
        (
            "alo-changing-printers",
            "changing-printers.refused.more-than-one-called",
        ),
        ("alo-choosing", "choosing.settings.not-understood"),
        ("alo-clipboard", "clipboard.nothing-copied"),
        ("alo-context", "context.the-document"),
        ("alo-converting", "converting.carried.everything"),
        ("alo-corridor", "corridor.at-the-door.not-granted-there"),
        ("alo-desktops", "desktops.always.egress-indicator"),
        ("alo-displays", "displays.as-you-left-them"),
        ("alo-dividing", "dividing.place.left-half"),
        ("alo-dock", "dock.edge.bottom"),
        ("alo-egress", "egress.destination.paired-machine"),
        ("alo-files", "files.failed.not-a-file-verb"),
        ("alo-finding", "finding.not-absolute"),
        ("alo-granted", "granted.nothing-granted"),
        ("alo-greeting", "greeting.make-an-account"),
        ("alo-handing", "handing.offered-for-this-question"),
        ("alo-in-use", "in-use.nothing-is-in-use"),
        ("alo-indicator", "indicator.nothing-is-leaving"),
        ("alo-installer", "installer.not-genuine"),
        ("alo-installing", "installing.not-genuine"),
        ("alo-keeping", "keeping.forever"),
        ("alo-keeping-up", "keeping-up.ready"),
        ("alo-keyboards", "keyboards.compose.none"),
        ("alo-leaving", "leaving.would-not-close"),
        ("alo-locking", "locking.locked"),
        ("alo-measuring", "measuring.not-on-this-host"),
        ("alo-menus", "menus.action.ask-the-agent-about-this"),
        ("alo-models", "models.source.this-machine"),
        ("alo-nearby", "nearby.may.ask-its-models"),
        ("alo-opening", "opening.cannot.damaged"),
        ("alo-overlay", "overlay.at-rest.nothing-chosen"),
        ("alo-picking", "picking.the-whole-machine"),
        ("alo-portals", "portals.portal.camera"),
        ("alo-power", "power.nearly-gone"),
        ("alo-printing", "printing.stopped.jammed"),
        ("alo-protocol", "protocol.too-long"),
        ("alo-proxy", "proxy.the-proxy.none"),
        ("alo-recounting", "recounting.nothing-to-tell"),
        ("alo-sessiond", "signing-in.not-opened"),
        ("alo-setting-up", "setup.the-question"),
        ("alo-shortcuts", "shortcuts.action.the-agent"),
        ("alo-sleeping", "sleeping.lid.stays-awake"),
        ("alo-software", "software.refused.signature-not-shown"),
        ("alo-sound", "sound.muted"),
        ("alo-telling", "telling.carry-on"),
        ("alo-turn", "turn.closed"),
    ];

    /// **alo OS's own words do not contradict each other.** This is the test
    /// that keeps [`NotCollected`] a thing nobody's machine ever sees: two
    /// crates landing on one key fails here rather than at somebody's first
    /// sign-in.
    #[test]
    fn everything_this_machine_says_can_be_collected() {
        let vocabulary = everything_this_machine_can_say().unwrap();
        assert!(!vocabulary.is_empty());
    }

    /// **Every crate on the list was actually reached.** A total would pass
    /// while a crate was silently dropped and another grew; one key from each
    /// cannot.
    #[test]
    fn every_crate_that_says_something_is_in_it() {
        let vocabulary = everything_this_machine_can_say().unwrap();
        for &(list, named) in ONE_STRING_EACH {
            let key = Key::named(named).unwrap();
            assert!(
                vocabulary.phrase(&key).is_some() || vocabulary.plural(&key).is_some(),
                "{list} was not collected: nothing here says {named}"
            );
        }
    }

    /// The two lists of crates are one list, so the count below is a count of
    /// something.
    #[test]
    fn the_lists_of_crates_agree() {
        let named: Vec<&str> = ONE_STRING_EACH.iter().map(|(list, _)| *list).collect();
        assert_eq!(named, EVERY_LIST);
    }

    /// **Nothing was lost and nothing was shared.** The machine's vocabulary
    /// holds exactly as many strings as the crates hold between them, which is
    /// only true while no key was dropped on the way in and no two crates
    /// declared the same one.
    #[test]
    fn the_machine_says_what_the_crates_say_between_them() {
        let each = [
            alo_adapting::words::adapting_words().unwrap().how_many(),
            alo_access::words::access_words().unwrap().how_many(),
            alo_accounts::accounts_words().unwrap().how_many(),
            alo_adapters::adapter_words().unwrap().how_many(),
            alo_answering::answering_words().unwrap().how_many(),
            alo_appearance::appearance_words().unwrap().how_many(),
            alo_applications::application_words().unwrap().how_many(),
            alo_approving::approving_words().unwrap().how_many(),
            alo_asking::asking_words().unwrap().how_many(),
            alo_bluetooth::bluetooth_words().unwrap().how_many(),
            alo_cameras::camera_words().unwrap().how_many(),
            alo_capability::capability_words().unwrap().how_many(),
            alo_capturing::capturing_words().unwrap().how_many(),
            alo_changing::changing_words().unwrap().how_many(),
            alo_changing_network::changing_network_words()
                .unwrap()
                .how_many(),
            alo_changing_printers::changing_printers_words()
                .unwrap()
                .how_many(),
            alo_choosing::choosing_words().unwrap().how_many(),
            alo_clipboard::clipboard_words().unwrap().how_many(),
            alo_context::context_words().unwrap().how_many(),
            alo_converting::converting_words().unwrap().how_many(),
            alo_corridor::corridor_words().unwrap().how_many(),
            alo_desktops::desktop_words().unwrap().how_many(),
            alo_displays::display_words().unwrap().how_many(),
            alo_dividing::dividing_words().unwrap().how_many(),
            alo_dock::dock_words().unwrap().how_many(),
            alo_egress::egress_words().unwrap().how_many(),
            alo_files::file_words().unwrap().how_many(),
            alo_finding::finding_words().unwrap().how_many(),
            alo_granted::granted_words().unwrap().how_many(),
            alo_greeting::greeting_words().unwrap().how_many(),
            alo_handing::handing_words().unwrap().how_many(),
            alo_in_use::in_use_words().unwrap().how_many(),
            alo_indicator::indicator_words().unwrap().how_many(),
            alo_installer::installer_words().unwrap().how_many(),
            alo_installing::installing_words().unwrap().how_many(),
            alo_keeping::keeping_words().unwrap().how_many(),
            alo_keeping_up::keeping_up_words().unwrap().how_many(),
            alo_keyboards::keyboard_words().unwrap().how_many(),
            alo_leaving::leaving_words().unwrap().how_many(),
            alo_locking::locking_words().unwrap().how_many(),
            alo_measuring::measuring_words().unwrap().how_many(),
            alo_menus::menu_words().unwrap().how_many(),
            alo_models::model_words().unwrap().how_many(),
            alo_nearby::nearby_words().unwrap().how_many(),
            alo_opening::opening_words().unwrap().how_many(),
            alo_overlay::overlay_words().unwrap().how_many(),
            alo_picking::picking_words().unwrap().how_many(),
            alo_portals::portal_words().unwrap().how_many(),
            alo_power::power_words().unwrap().how_many(),
            alo_printing::printing_words().unwrap().how_many(),
            alo_protocol::protocol_words().unwrap().how_many(),
            alo_proxy::proxy_words().unwrap().how_many(),
            alo_recounting::recounting_words().unwrap().how_many(),
            alo_sessiond::sessiond_words().unwrap().how_many(),
            alo_setting_up::setting_up_words().unwrap().how_many(),
            alo_shortcuts::shortcut_words().unwrap().how_many(),
            alo_sleeping::sleeping_words().unwrap().how_many(),
            alo_software::software_words().unwrap().how_many(),
            alo_sound::words::sound_words().unwrap().how_many(),
            alo_telling::telling_words().unwrap().how_many(),
            alo_turn::turn_words().unwrap().how_many(),
        ];
        assert_eq!(each.len(), EVERY_LIST.len());
        assert_eq!(
            everything_this_machine_can_say().unwrap().how_many(),
            each.iter().sum::<usize>()
        );
    }

    /// A crate whose list will not declare names itself, because the person
    /// reading this is looking for a file to open.
    #[test]
    fn a_list_that_will_not_declare_names_the_crate() {
        let mut vocabulary = Vocabulary::empty();
        let refused = declare(&mut vocabulary, "alo-nothing", |_| {
            Err::<(), &str>("two of them are called files.gone")
        })
        .unwrap_err();
        assert_eq!(refused.list(), "alo-nothing");
        assert_eq!(refused.why(), "two of them are called files.gone");
        assert!(refused.to_string().contains("alo-nothing"), "{refused}");
        assert!(refused.to_string().contains("files.gone"), "{refused}");
    }
}
