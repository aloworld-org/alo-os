//! One person's folder, from sign-in to the next change.
//!
//! Every clause of ADR 0038 is held crate by crate: `alo-kept` holds the rule,
//! `alo-appearance`, `alo-dock` and `alo-shortcuts` each hold their own file,
//! and this crate holds where the folder is. What none of them can hold is the
//! road a session takes through all of them at once — the road the shell's
//! Settings surface calls. A break between two crates that each pass their own
//! suite, a file name two of them disagree on or a folder one of them makes
//! with the wrong mode, is found here or by a person.
//!
//! So this walks it the way a session does. A temporary home directory is
//! handed to [`alo_choosing::the_persons_folder`] — never read from the
//! environment — and each keeper is handed the path its own `THE_FILE` names
//! inside that folder. The three files are written beside `settings.toml`,
//! read back at a second sign-in, one of them is broken by hand, the next
//! change to it is refused with its bytes unchanged while another section's
//! change is written, and putting it back as shipped lets the next change
//! through. The walk is made once with each of the three broken, because a
//! road that holds for one file and not for the others is not one road.
//!
//! `docs/contracts/person-settings.md` names, for the shell, the calls a
//! Settings surface makes for each section, and this test reads that section:
//! every call it names is one the walk makes, and every call the walk makes is
//! named.
//!
//! This crate is where the walk lives because it is where the folder is worked
//! out; the three keepers are dev-dependencies here, and none of them gains a
//! dependency on this crate (`the_folder_is_handed_rather_than_depended_on.rs`).

#![expect(
    clippy::expect_used,
    clippy::panic,
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::collections::BTreeSet;
use std::fmt::Debug;
use std::path::{Path, PathBuf};

use alo_appearance::{Accent, Appearance, TextScale};
use alo_choosing::{
    Choosing, SESSION_NO_FOLDER, Settings, THE_FOLDER, THE_SETTINGS, the_persons_folder,
    where_it_is,
};
use alo_dock::{Dock, Edge};
use alo_shortcuts::{Action, Chord, Key, Modifier, Modifiers, Shortcuts};
use alo_strings::{Language, Said, Strings};

/// Every call the walk makes into the crates, as the contract's section for
/// the shell must name them — no more and no fewer.
const THE_CALLS: [&str; 15] = [
    "alo_choosing::the_persons_folder",
    "alo_choosing::PersonsFolder::path_of",
    "alo_choosing::NoFolder::said",
    "alo_appearance::keeping::THE_FILE",
    "alo_appearance::keeping::at_sign_in",
    "alo_appearance::keeping::keep",
    "alo_appearance::keeping::put_back_as_shipped",
    "alo_dock::keeping::THE_FILE",
    "alo_dock::keeping::at_sign_in",
    "alo_dock::keeping::keep",
    "alo_dock::keeping::put_back_as_shipped",
    "alo_shortcuts::keeping::THE_FILE",
    "alo_shortcuts::keeping::at_sign_in",
    "alo_shortcuts::keeping::keep",
    "alo_shortcuts::keeping::put_back_as_shipped",
];

/// The heading of the contract's section for the shell.
const THE_SHELLS_SECTION: &str = "## A Settings surface, from sign-in to the next change";

/// A refusal a section says, reduced to what the walk asks of it.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Refused {
    /// The file the refusal is about.
    at: PathBuf,
    /// The key it names, when a key was what was wrong.
    key: Option<String>,
    /// What the person reads.
    said: Said,
}

/// A change a section did not write.
#[derive(Debug)]
struct NotKept {
    /// The file that was not replaced.
    at: PathBuf,
    /// What is wrong with that file, when it was not replaced because it did
    /// not read.
    did_not_read: Option<Refused>,
    /// What the person reads.
    said: Said,
}

/// One section of Settings, as a surface reaches it: through the keeping
/// module of the crate that owns its file and nothing else.
trait Section {
    /// What the section is called in a failure message.
    const NAME: &'static str;

    /// The file's name inside the folder, as the crate declares it.
    const FILE: &'static str;

    /// What the section draws.
    type Drawn: Clone + Debug + PartialEq;

    /// The section as the release ships it.
    fn shipped() -> Self::Drawn;

    /// What the section draws at sign-in, and the refusal beside it.
    fn at_sign_in(at: &Path, strings: &Strings) -> (Self::Drawn, Option<Refused>);

    /// A change a person makes in the section. `second` is a different change
    /// from the first, so a change after a change is visibly a change.
    fn changed(drawn: Self::Drawn, second: bool) -> Self::Drawn;

    /// The whole of the section's changes kept at `at`.
    fn keep(at: &Path, drawn: &Self::Drawn, strings: &Strings) -> Result<(), Box<NotKept>>;

    /// The section put back as the release ships it.
    fn put_back_as_shipped(at: &Path, strings: &Strings) -> Result<(), Box<NotKept>>;
}

/// Appearance.
struct AppearanceSection;

impl Section for AppearanceSection {
    const NAME: &'static str = "appearance";
    const FILE: &'static str = alo_appearance::keeping::THE_FILE;
    type Drawn = Appearance;

    fn shipped() -> Appearance {
        Appearance::shipped()
    }

    fn at_sign_in(at: &Path, strings: &Strings) -> (Appearance, Option<Refused>) {
        let (drawn, refused) = alo_appearance::keeping::at_sign_in(at);
        let refused = refused.map(|refused| Refused {
            at: refused.at().to_owned(),
            key: refused.key().map(str::to_owned),
            said: refused.said(strings),
        });
        (drawn, refused)
    }

    fn changed(mut drawn: Appearance, second: bool) -> Appearance {
        if second {
            drawn.set_accent(Accent::Moss);
        } else {
            drawn.set_accent(Accent::Rose);
            drawn.set_text(TextScale::percent(125).unwrap());
        }
        drawn
    }

    fn keep(at: &Path, drawn: &Appearance, strings: &Strings) -> Result<(), Box<NotKept>> {
        alo_appearance::keeping::keep(at, drawn.changes()).map_err(|refused| {
            Box::new(NotKept {
                at: refused.at().to_owned(),
                did_not_read: refused.did_not_read().map(|why| Refused {
                    at: why.at().to_owned(),
                    key: why.key().map(str::to_owned),
                    said: why.said(strings),
                }),
                said: refused.said(strings),
            })
        })
    }

    fn put_back_as_shipped(at: &Path, strings: &Strings) -> Result<(), Box<NotKept>> {
        alo_appearance::keeping::put_back_as_shipped(at).map_err(|refused| {
            Box::new(NotKept {
                at: refused.at().to_owned(),
                did_not_read: None,
                said: refused.said(strings),
            })
        })
    }
}

/// The dock.
struct DockSection;

impl Section for DockSection {
    const NAME: &'static str = "dock";
    const FILE: &'static str = alo_dock::keeping::THE_FILE;
    type Drawn = Dock;

    fn shipped() -> Dock {
        Dock::shipped()
    }

    fn at_sign_in(at: &Path, strings: &Strings) -> (Dock, Option<Refused>) {
        let (drawn, refused) = alo_dock::keeping::at_sign_in(at);
        let refused = refused.map(|refused| Refused {
            at: refused.at().to_owned(),
            key: refused.key().map(str::to_owned),
            said: refused.said(strings),
        });
        (drawn, refused)
    }

    fn changed(mut drawn: Dock, second: bool) -> Dock {
        drawn.set_edge(if second { Edge::Right } else { Edge::Left });
        drawn
    }

    fn keep(at: &Path, drawn: &Dock, strings: &Strings) -> Result<(), Box<NotKept>> {
        alo_dock::keeping::keep(at, drawn.changes()).map_err(|refused| {
            Box::new(NotKept {
                at: refused.at().to_owned(),
                did_not_read: refused.did_not_read().map(|why| Refused {
                    at: why.at().to_owned(),
                    key: why.key().map(str::to_owned),
                    said: why.said(strings),
                }),
                said: refused.said(strings),
            })
        })
    }

    fn put_back_as_shipped(at: &Path, strings: &Strings) -> Result<(), Box<NotKept>> {
        alo_dock::keeping::put_back_as_shipped(at).map_err(|refused| {
            Box::new(NotKept {
                at: refused.at().to_owned(),
                did_not_read: None,
                said: refused.said(strings),
            })
        })
    }
}

/// Shortcuts.
struct ShortcutsSection;

impl Section for ShortcutsSection {
    const NAME: &'static str = "shortcuts";
    const FILE: &'static str = alo_shortcuts::keeping::THE_FILE;
    type Drawn = Shortcuts;

    fn shipped() -> Shortcuts {
        Shortcuts::shipped()
    }

    fn at_sign_in(at: &Path, strings: &Strings) -> (Shortcuts, Option<Refused>) {
        let (drawn, refused) = alo_shortcuts::keeping::at_sign_in(at);
        let refused = refused.map(|refused| Refused {
            at: refused.at().to_owned(),
            key: refused.key().map(str::to_owned),
            said: refused.said(strings),
        });
        (drawn, refused)
    }

    fn changed(mut drawn: Shortcuts, second: bool) -> Shortcuts {
        if second {
            drawn.unbind(Action::Launcher);
        } else {
            let ctrl_alt_space = Chord::checked(
                Modifiers::just(Modifier::Ctrl).and(Modifier::Alt),
                Key::Space,
            )
            .unwrap();
            drawn.bind(Action::TheAgent, ctrl_alt_space).unwrap();
        }
        drawn
    }

    fn keep(at: &Path, drawn: &Shortcuts, strings: &Strings) -> Result<(), Box<NotKept>> {
        alo_shortcuts::keeping::keep(at, drawn.changes()).map_err(|refused| {
            Box::new(NotKept {
                at: refused.at().to_owned(),
                did_not_read: refused.did_not_read().map(|why| Refused {
                    at: why.at().to_owned(),
                    key: why.key().map(str::to_owned),
                    said: why.said(strings),
                }),
                said: refused.said(strings),
            })
        })
    }

    fn put_back_as_shipped(at: &Path, strings: &Strings) -> Result<(), Box<NotKept>> {
        alo_shortcuts::keeping::put_back_as_shipped(at).map_err(|refused| {
            Box::new(NotKept {
                at: refused.at().to_owned(),
                did_not_read: None,
                said: refused.said(strings),
            })
        })
    }
}

/// A person's folder, worked out once at sign-in and handed out as paths.
struct Folder {
    /// The folder itself.
    folder: PathBuf,
    /// `settings.toml`, this crate's own.
    settings: PathBuf,
    /// `appearance.toml`.
    appearance: PathBuf,
    /// `dock.toml`.
    dock: PathBuf,
    /// `shortcuts.toml`.
    shortcuts: PathBuf,
}

impl Folder {
    /// The folder of a login whose home is `home` and whose session names no
    /// `$XDG_CONFIG_HOME`, handed over rather than read from the environment.
    fn of(home: &Path) -> Self {
        let folder = the_persons_folder(None, Some(home.as_os_str()))
            .expect("a login with a home directory has a folder");
        Self {
            settings: where_it_is(None, Some(home.as_os_str())).unwrap(),
            appearance: folder.path_of(AppearanceSection::FILE),
            dock: folder.path_of(DockSection::FILE),
            shortcuts: folder.path_of(ShortcutsSection::FILE),
            folder: folder.folder().to_owned(),
        }
    }
}

/// A home directory of this test's own, with nothing in it.
fn a_home_of_our_own(what: &str) -> PathBuf {
    let home = std::env::temp_dir().join(format!("alo-choosing-one-persons-folder-{what}"));
    if home.exists() {
        std::fs::remove_dir_all(&home).unwrap();
    }
    std::fs::create_dir_all(&home).unwrap();
    home
}

/// Everything the machine can say, which is what a session really holds.
fn everything_this_machine_can_say() -> Strings {
    Strings::of(alo_saying::everything_this_machine_can_say().unwrap())
}

/// A hand edit that breaks a kept file: a key at the top of it that is not on
/// the list, typed straight under the `format` line.
fn broken_by_hand(at: &Path) {
    let written = std::fs::read_to_string(at).unwrap();
    assert!(written.starts_with("format = 1\n"), "{written}");
    let edited = written.replacen("format = 1\n", "format = 1\nwallpaper = \"harbour\"\n", 1);
    std::fs::write(at, edited).unwrap();
}

/// A sentence a person reads names this file, has every gap filled, and — when
/// a key is given — names the key.
fn says_the_file(said: &Said, at: &Path, key: Option<&str>) {
    assert!(
        said.text().contains(&at.display().to_string()),
        "the sentence does not name {}: {said}",
        at.display()
    );
    assert!(said.unfilled().is_empty(), "{said}");
    if let Some(key) = key {
        assert!(
            said.text().contains(key),
            "the sentence does not name {key}: {said}"
        );
    }
}

/// Every section as the person left it at the first sign-in's changes, for the
/// sections other than the one being broken.
struct LeftAs {
    /// Appearance, as changed.
    appearance: Appearance,
    /// The dock, as changed.
    dock: Dock,
    /// Shortcuts, as changed.
    shortcuts: Shortcuts,
}

/// The first sign-in on a machine nobody has configured: nothing is drawn but
/// the release, nothing is written by drawing it, and then the person makes a
/// change in each of the three sections and chooses what they read.
fn a_first_sign_in(at: &Folder, strings: &Strings) -> LeftAs {
    assert!(!at.folder.exists(), "a new login already had a folder");
    let (appearance, refused) = AppearanceSection::at_sign_in(&at.appearance, strings);
    assert_eq!((appearance.clone(), refused), (Appearance::shipped(), None));
    let (dock, refused) = DockSection::at_sign_in(&at.dock, strings);
    assert_eq!((dock.clone(), refused), (Dock::shipped(), None));
    let (shortcuts, refused) = ShortcutsSection::at_sign_in(&at.shortcuts, strings);
    assert_eq!((shortcuts.clone(), refused), (Shortcuts::shipped(), None));
    assert!(!at.folder.exists(), "drawing the release wrote something");

    let appearance = AppearanceSection::changed(appearance, false);
    AppearanceSection::keep(&at.appearance, &appearance, strings).unwrap();
    let dock = DockSection::changed(dock, false);
    DockSection::keep(&at.dock, &dock, strings).unwrap();
    let shortcuts = ShortcutsSection::changed(shortcuts, false);
    ShortcutsSection::keep(&at.shortcuts, &shortcuts, strings).unwrap();
    Choosing::at(&at.settings)
        .unwrap()
        .reading(vec![Language::written("de").unwrap()])
        .unwrap();

    LeftAs {
        appearance,
        dock,
        shortcuts,
    }
}

/// **The three files are beside `settings.toml`**, each under the name its own
/// crate declares, in the folder the session was handed — and on a machine
/// with modes, the folder is the person's alone and so is every file in it.
fn the_files_are_beside_the_settings(at: &Folder, home: &Path) {
    assert_eq!(
        at.folder,
        home.join(".config").join(THE_FOLDER),
        "the folder is not where a session looks"
    );
    assert_eq!(at.settings, at.folder.join(THE_SETTINGS));
    let mut in_the_folder: Vec<String> = std::fs::read_dir(&at.folder)
        .unwrap()
        .map(|entry| entry.unwrap().file_name().into_string().unwrap())
        .collect();
    in_the_folder.sort();
    assert_eq!(
        in_the_folder,
        [
            "appearance.toml",
            "dock.toml",
            "settings.toml",
            "shortcuts.toml"
        ],
        "the folder holds something other than the four files, or one of them under another name"
    );

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;

        let mode = |path: &Path| std::fs::metadata(path).unwrap().permissions().mode() & 0o777;
        assert_eq!(
            mode(&at.folder),
            0o700,
            "the folder went down {:o}",
            mode(&at.folder)
        );
        for file in [&at.settings, &at.appearance, &at.dock, &at.shortcuts] {
            assert_eq!(
                mode(file),
                0o600,
                "{} went down {:o}",
                file.display(),
                mode(file)
            );
        }
    }
}

/// **The second sign-in draws what the person left**, from the files, in every
/// section, and reads what they chose to read.
fn a_second_sign_in_draws_what_was_left(at: &Folder, left: &LeftAs, strings: &Strings) {
    assert_eq!(
        AppearanceSection::at_sign_in(&at.appearance, strings),
        (left.appearance.clone(), None)
    );
    assert_eq!(
        DockSection::at_sign_in(&at.dock, strings),
        (left.dock.clone(), None)
    );
    assert_eq!(
        ShortcutsSection::at_sign_in(&at.shortcuts, strings),
        (left.shortcuts.clone(), None)
    );
    assert_ne!(left.appearance, Appearance::shipped());
    assert_ne!(left.dock, Dock::shipped());
    assert_ne!(left.shortcuts, Shortcuts::shipped());
    what_the_person_reads_is_untouched(at);
}

/// `settings.toml` still says what the person chose to read.
fn what_the_person_reads_is_untouched(at: &Folder) {
    let settings = Settings::at(&at.settings).unwrap();
    let languages: Vec<&str> = settings.languages().iter().map(Language::tag).collect();
    assert_eq!(languages, ["de"]);
}

/// One section broken by hand, walked from the sign-in after the edit to the
/// change after it was put back as shipped. `Other` is a section that is not
/// broken, whose change is written while the broken one's is refused.
fn broken_and_put_back<Broken: Section, Other: Section>(
    broken_at: &Path,
    other_at: &Path,
    broken_left: &Broken::Drawn,
    strings: &Strings,
) {
    broken_by_hand(broken_at);
    let bytes = std::fs::read(broken_at).unwrap();

    // Drawn as shipped, with a sentence naming the file and the key.
    let (drawn, refused) = Broken::at_sign_in(broken_at, strings);
    assert_eq!(
        drawn,
        Broken::shipped(),
        "{}: nothing in a broken file is honoured",
        Broken::NAME
    );
    let refused = refused.unwrap_or_else(|| panic!("{}: a broken file read", Broken::NAME));
    assert_eq!(refused.at, broken_at);
    assert_eq!(refused.key.as_deref(), Some("wallpaper"));
    says_the_file(&refused.said, broken_at, Some("wallpaper"));

    // The next change to it is refused, naming the file and what is wrong with
    // it, and the person's edit is there byte for byte.
    let not_kept = Broken::keep(broken_at, &Broken::changed(drawn.clone(), true), strings)
        .expect_err("a change was written over a file that did not read");
    assert_eq!(not_kept.at, broken_at);
    says_the_file(&not_kept.said, broken_at, None);
    assert_eq!(not_kept.did_not_read.as_ref(), Some(&refused));
    assert_eq!(
        std::fs::read(broken_at).unwrap(),
        bytes,
        "{}: the person's edit is gone",
        Broken::NAME
    );
    assert_eq!(Broken::at_sign_in(broken_at, strings).1, Some(refused));

    // A change to another section, made at the same moment, is written.
    let (other, beside) = Other::at_sign_in(other_at, strings);
    assert_eq!(
        beside,
        None,
        "{}: one broken file broke another",
        Other::NAME
    );
    let other = Other::changed(other, true);
    Other::keep(other_at, &other, strings).unwrap();
    assert_eq!(Other::at_sign_in(other_at, strings), (other, None));

    // Put back as shipped, the section reads as the release, and the next
    // change goes through and is read back.
    Broken::put_back_as_shipped(broken_at, strings).unwrap();
    assert_eq!(std::fs::read_to_string(broken_at).unwrap(), "format = 1\n");
    assert_eq!(
        Broken::at_sign_in(broken_at, strings),
        (Broken::shipped(), None)
    );
    let next = Broken::changed(Broken::shipped(), true);
    Broken::keep(broken_at, &next, strings).unwrap();
    assert_eq!(Broken::at_sign_in(broken_at, strings), (next.clone(), None));
    assert_ne!(&next, broken_left, "the walk's two changes are one change");
}

/// Every `alo_…` code span in the contract's section for the shell.
fn the_calls_the_contract_names() -> BTreeSet<String> {
    let at = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../docs/contracts/person-settings.md");
    let contract = std::fs::read_to_string(&at)
        .unwrap_or_else(|why| panic!("{} could not be read: {why}", at.display()));
    let mut lines = contract
        .lines()
        .skip_while(|line| *line != THE_SHELLS_SECTION);
    assert!(
        lines.next().is_some(),
        "the contract has no section headed {THE_SHELLS_SECTION:?}"
    );
    let section: Vec<&str> = lines.take_while(|line| !line.starts_with("## ")).collect();
    let text = section.join(" ");
    assert!(text.contains(SESSION_NO_FOLDER.named()));
    assert!(text.contains(SESSION_NO_FOLDER.says()));
    section
        .join("\n")
        .split('`')
        .skip(1)
        .step_by(2)
        .filter(|span| span.starts_with("alo_"))
        .map(str::to_owned)
        .collect()
}

/// **One person's folder, from sign-in to the next change**, walked with each
/// of the three files broken in turn — and the contract's section for the
/// shell names exactly the calls the walk made.
#[test]
fn one_persons_folder_is_walked_from_sign_in_to_the_next_change() {
    let strings = everything_this_machine_can_say();
    let refused = the_persons_folder(None, None).unwrap_err();
    assert_eq!(refused.said(&strings).text(), SESSION_NO_FOLDER.says());

    for broken in ["appearance", "dock", "shortcuts"] {
        let home = a_home_of_our_own(broken);
        let at = Folder::of(&home);

        let left = a_first_sign_in(&at, &strings);
        the_files_are_beside_the_settings(&at, &home);
        a_second_sign_in_draws_what_was_left(&at, &left, &strings);

        match broken {
            "appearance" => {
                broken_and_put_back::<AppearanceSection, DockSection>(
                    &at.appearance,
                    &at.dock,
                    &left.appearance,
                    &strings,
                );
                assert_eq!(
                    ShortcutsSection::at_sign_in(&at.shortcuts, &strings),
                    (left.shortcuts.clone(), None),
                    "shortcuts are not drawn as the person left them"
                );
            }
            "dock" => {
                broken_and_put_back::<DockSection, ShortcutsSection>(
                    &at.dock,
                    &at.shortcuts,
                    &left.dock,
                    &strings,
                );
                assert_eq!(
                    AppearanceSection::at_sign_in(&at.appearance, &strings),
                    (left.appearance.clone(), None),
                    "appearance is not drawn as the person left it"
                );
            }
            _ => {
                broken_and_put_back::<ShortcutsSection, AppearanceSection>(
                    &at.shortcuts,
                    &at.appearance,
                    &left.shortcuts,
                    &strings,
                );
                assert_eq!(
                    DockSection::at_sign_in(&at.dock, &strings),
                    (left.dock.clone(), None),
                    "the dock is not drawn as the person left it"
                );
            }
        }
        what_the_person_reads_is_untouched(&at);
        the_files_are_beside_the_settings(&at, &home);
    }

    let named = the_calls_the_contract_names();
    let made: BTreeSet<String> = THE_CALLS.iter().map(|call| (*call).to_owned()).collect();
    assert_eq!(
        named, made,
        "the contract's section for the shell does not name exactly the calls a session makes"
    );
    assert_eq!(
        [
            AppearanceSection::FILE,
            DockSection::FILE,
            ShortcutsSection::FILE
        ],
        ["appearance.toml", "dock.toml", "shortcuts.toml"],
        "a keeper's file is not the file the contract names"
    );
}
