//! A machine with nothing real behind it: a desktop door that answers as it is
//! told, an accessibility tree written by the test, a session bus that keeps
//! what it is sent, a daemon that keeps a place or does not, and a record that
//! is a list.

#![expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::cell::RefCell;
use std::collections::{BTreeMap, VecDeque};
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

use alo_acting::{Answer, Counted, InFront, Request, ToCount, WhereAlosTreeIs};
use alo_adapters::{
    AccessibilityTree, Adapters, Delivers, Facts, Message, NodeAt, NotDelivered, Running, States,
    TreeFault, adapter_verbs, shipped_adapters,
};
use alo_applications::{Application, Installed};
use alo_capability::Verbs;
use alo_record::Entry;
use alo_strings::{Said, Strings, Vocabulary};

use crate::door::{HeldCount, TheDoor, Unanswered};
use crate::ended::Ended;
use crate::machine::{APlace, Knowing, Machine};

/// The text editor.
pub const EDITOR: &str = "org.gnome.TextEditor";

/// The unique name alo's own tree answers as.
pub const ALOS_TREE: &str = ":1.7";

/// A count, as the door answers it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Counting {
    /// It begins, and ends with this.
    Ends(Option<Counted>),
    /// It cannot.
    Cannot,
}

/// A desktop door that answers as it is told, and keeps what it was asked.
#[derive(Debug)]
pub struct ADoor {
    /// What `front` answers.
    pub front: Result<InFront, Unanswered>,
    /// What `alo-tree` answers.
    pub alos_tree: Result<WhereAlosTreeIs, Unanswered>,
    /// What an act answers.
    pub acting: Result<Answer, Unanswered>,
    /// What a count does.
    pub counting: Counting,
    /// Every line it was asked.
    pub asked: Vec<String>,
}

impl Default for ADoor {
    fn default() -> Self {
        Self {
            front: Ok(InFront::None),
            alos_tree: Ok(WhereAlosTreeIs::Nowhere),
            acting: Ok(Answer::Acted),
            counting: Counting::Ends(Some(Counted::Sent(0))),
            asked: Vec::new(),
        }
    }
}

/// A count being held by [`ADoor`].
struct AHeldCount(Option<Counted>);

impl HeldCount for AHeldCount {
    fn done(self: Box<Self>) -> Option<Counted> {
        self.0
    }
}

impl TheDoor for ADoor {
    fn front(&mut self) -> Result<InFront, Unanswered> {
        self.asked.push("front".to_owned());
        self.front.clone()
    }

    fn alos_tree(&mut self) -> Result<WhereAlosTreeIs, Unanswered> {
        self.asked.push("alo-tree".to_owned());
        self.alos_tree.clone()
    }

    fn act(&mut self, request: &Request) -> Result<Answer, Unanswered> {
        self.asked.push(request.written());
        self.acting.clone()
    }

    fn begin_counting(&mut self, asked: &ToCount) -> Option<Box<dyn HeldCount>> {
        self.asked.push(asked.written());
        match self.counting {
            Counting::Ends(ending) => Some(Box::new(AHeldCount(ending))),
            Counting::Cannot => None,
        }
    }
}

/// `AtspiRole` numbers a test writes.
pub mod roles {
    /// An application's root.
    pub const APPLICATION: u32 = 75;
    /// A frame: a window.
    pub const FRAME: u32 = 23;
    /// A button.
    pub const PUSH_BUTTON: u32 = 43;
    /// A label.
    pub const LABEL: u32 = 29;
}

/// A tree with one application holding one window of buttons and a label.
#[derive(Debug, Default)]
pub struct ATree {
    /// The applications listed.
    running: Vec<Running>,
    /// Each thing's facts.
    facts: BTreeMap<NodeAt, Facts>,
    /// Each thing's actions.
    actions: BTreeMap<NodeAt, Vec<String>>,
    /// How many presses were asked.
    pub pressed: RefCell<usize>,
}

impl ATree {
    /// `application`, on `holder`, with a window titled `title` holding a
    /// button for each name and a label saying `words`.
    #[must_use]
    pub fn with(
        mut self,
        holder: &str,
        application: Option<&str>,
        title: &str,
        buttons: &[&str],
        words: &str,
    ) -> Self {
        let at = |object: String| NodeAt {
            holder: holder.to_owned(),
            object,
        };
        let n = self.facts.len();
        let root = at(format!("/root/{n}"));
        let window = at(format!("/window/{n}"));
        let mut inside = Vec::new();
        for (i, name) in buttons.iter().enumerate() {
            let button = at(format!("/button/{n}/{i}"));
            self.facts
                .insert(button.clone(), facts(roles::PUSH_BUTTON, name, Vec::new()));
            self.actions
                .insert(button.clone(), vec!["click".to_owned()]);
            inside.push(button);
        }
        let label = at(format!("/label/{n}"));
        self.facts
            .insert(label.clone(), facts(roles::LABEL, words, Vec::new()));
        inside.push(label);
        self.facts
            .insert(window.clone(), facts(roles::FRAME, title, inside));
        self.facts
            .insert(root.clone(), facts(roles::APPLICATION, "", vec![window]));
        self.running.push(Running {
            at: root,
            is: application.map(str::to_owned),
        });
        self
    }
}

impl ATree {
    /// Every button offering only these actions instead of `click`.
    #[must_use]
    pub fn offering(mut self, actions: &[&str]) -> Self {
        for (at, offered) in &mut self.actions {
            if self
                .facts
                .get(at)
                .is_some_and(|facts| facts.role == roles::PUSH_BUTTON)
            {
                *offered = actions.iter().map(|action| (*action).to_owned()).collect();
            }
        }
        self
    }
}

/// One thing's facts: showing, usable and named.
fn facts(role: u32, name: &str, children: Vec<NodeAt>) -> Facts {
    Facts {
        role,
        name: name.to_owned(),
        states: States::showing_and_usable().as_the_tree_says().to_vec(),
        children,
        has_text: false,
        unlisted_children: false,
    }
}

impl AccessibilityTree for ATree {
    fn applications(&self) -> Result<Vec<Running>, TreeFault> {
        Ok(self.running.clone())
    }

    fn facts(&self, at: &NodeAt) -> Result<Facts, TreeFault> {
        self.facts.get(at).cloned().ok_or(TreeFault::Vanished)
    }

    fn text(&self, _: &NodeAt) -> Result<String, TreeFault> {
        Ok(String::new())
    }

    fn actions(&self, at: &NodeAt) -> Result<Vec<String>, TreeFault> {
        Ok(self.actions.get(at).cloned().unwrap_or_default())
    }

    fn act(&self, _: &NodeAt, _: usize) -> Result<bool, TreeFault> {
        *self.pressed.borrow_mut() += 1;
        Ok(true)
    }
}

/// A session bus that keeps what it is sent and answers as it is told.
#[derive(Debug)]
pub struct ABus {
    /// What it answers.
    pub answering: Result<(), NotDelivered>,
    /// Everything sent.
    pub sent: RefCell<Vec<Message>>,
}

impl Delivers for ABus {
    fn deliver(&self, message: &Message) -> Result<(), NotDelivered> {
        self.sent.borrow_mut().push(message.clone());
        self.answering
    }
}

/// A machine.
#[derive(Debug)]
pub struct AMachine {
    /// Its door.
    pub door: ADoor,
    /// Its tree, or none.
    pub tree: Option<ATree>,
    /// Its bus.
    pub bus: ABus,
    /// What the daemon answers.
    pub place: APlace,
    /// How many places were asked for.
    pub places_asked: usize,
    /// Whether the record opens.
    pub record_opens: bool,
    /// The record.
    pub kept: Vec<Entry>,
    /// The pipe.
    pub pipe: VecDeque<String>,
    /// What was told on the output.
    pub told: Vec<String>,
    /// What was said on the error output.
    pub warned: Vec<String>,
    /// What the person turned on in Settings → Accessibility.
    pub access: alo_access::TurnedOn,
}

impl Default for AMachine {
    fn default() -> Self {
        Self {
            door: ADoor::default(),
            tree: None,
            bus: ABus {
                answering: Ok(()),
                sent: RefCell::new(Vec::new()),
            },
            place: APlace::Kept("1762516800-42-their-own".to_owned()),
            places_asked: 0,
            record_opens: true,
            kept: Vec::new(),
            pipe: VecDeque::new(),
            told: Vec::new(),
            warned: Vec::new(),
            access: alo_access::TurnedOn::nothing(),
        }
    }
}

impl Machine for AMachine {
    fn now(&self) -> SystemTime {
        SystemTime::UNIX_EPOCH + Duration::from_secs(1_762_516_800)
    }

    fn door(&mut self) -> &mut dyn TheDoor {
        &mut self.door
    }

    fn tree(&mut self) -> Result<&dyn AccessibilityTree, TreeFault> {
        self.tree
            .as_ref()
            .map(|tree| tree as &dyn AccessibilityTree)
            .ok_or(TreeFault::NotThere)
    }

    fn bus(&mut self) -> Result<&dyn Delivers, NotDelivered> {
        Ok(&self.bus)
    }

    fn access(&mut self) -> alo_access::TurnedOn {
        self.access.clone()
    }

    fn a_place_to_return_to(&mut self, _: &Strings) -> APlace {
        self.places_asked += 1;
        self.place.clone()
    }

    fn the_record_is_open(&mut self, strings: &Strings) -> Result<(), Said> {
        if self.record_opens {
            Ok(())
        } else {
            Err(strings.say(
                &alo_strings::Key::named("app.typed.nothing").unwrap(),
                &alo_strings::Filling::nothing(),
            ))
        }
    }

    fn keep(&mut self, entry: &Entry, strings: &Strings) -> Result<(), Said> {
        self.the_record_is_open(strings)?;
        self.kept.push(entry.clone());
        Ok(())
    }

    fn next_on_the_pipe(&mut self) -> Option<String> {
        self.pipe.pop_front()
    }

    fn told(&mut self, line: &str) {
        self.told.push(line.to_owned());
    }

    fn warned(&mut self, line: &str) {
        self.warned.push(line.to_owned());
    }
}

/// What a command knows, owned by the test.
pub struct Known {
    /// The strings, in English.
    pub strings: Strings,
    /// What is installed.
    pub installed: Installed,
    /// The adapters.
    pub adapters: Adapters,
    /// Their verbs.
    pub verbs: Verbs,
    /// Where the shell is.
    pub standing_in: PathBuf,
}

impl Known {
    /// A machine with these installed.
    #[must_use]
    pub fn holding(identifiers: &[&str]) -> Self {
        let mut vocabulary = Vocabulary::empty();
        crate::words::declare_into(&mut vocabulary).unwrap();
        alo_adapters::words::declare_into(&mut vocabulary).unwrap();
        alo_applications::words::declare_into(&mut vocabulary).unwrap();
        alo_capability::declare_into(&mut vocabulary).unwrap();
        Self {
            strings: Strings::of(vocabulary),
            installed: Installed::holding(
                identifiers
                    .iter()
                    .map(|identifier| Application::identified(identifier).unwrap()),
            ),
            adapters: shipped_adapters().unwrap(),
            verbs: adapter_verbs().unwrap(),
            standing_in: Path::new("/home/anna/Notes").to_path_buf(),
        }
    }

    /// As a command knows it.
    #[must_use]
    pub fn knowing(&self) -> Knowing<'_> {
        Knowing {
            strings: &self.strings,
            installed: &self.installed,
            adapters: &self.adapters,
            verbs: &self.verbs,
            standing_in: &self.standing_in,
        }
    }

    /// Run one command line, as on a terminal.
    pub fn running(&self, line: &[&str], machine: &mut AMachine) -> Ended {
        let typed: Vec<String> = line.iter().map(|word| (*word).to_owned()).collect();
        crate::command::run(&typed, true, self.knowing(), machine)
    }

    /// Run one command line, its output going to a pipe.
    pub fn piped(&self, line: &[&str], machine: &mut AMachine) -> Ended {
        let typed: Vec<String> = line.iter().map(|word| (*word).to_owned()).collect();
        crate::command::run(&typed, false, self.knowing(), machine)
    }
}
