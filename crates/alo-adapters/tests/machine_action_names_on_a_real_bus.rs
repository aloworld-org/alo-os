//! AT-SPI GetActions is localized; GetName identifies what a client may invoke.
//! These isolated wire fixtures are not an installed Settings acceptance run.

#![cfg(target_os = "linux")]
#![expect(clippy::unwrap_used, clippy::expect_used,
    reason = "unexpected fixture refusal is a test failure")]

use std::{collections::HashMap, io::{BufRead, BufReader}, path::PathBuf,
    process::{Child, Command, Stdio}, sync::{Arc, Mutex}, time::{SystemTime, UNIX_EPOCH}};
use alo_adapters::{AccessibilityTree, AccessibleSession, NodeAt, Pressable, Whose, press_by_hand};
use alo_portals::Sandboxes;
use zbus::{blocking::Connection, blocking::connection::Builder,
    zvariant::{OwnedObjectPath, OwnedValue}};

const ROOT: &str = "/org/a11y/atspi/accessible/root";
const WINDOW: &str = "/org/a11y/atspi/accessible/window";
const CONTROL: &str = "/org/a11y/atspi/accessible/control";

struct Bus { directory: PathBuf, daemon: Child, address: String }
impl Bus {
    fn start() -> Self {
        let directory = std::env::temp_dir().join(format!("alo-machine-actions-{}-{}",
            std::process::id(), SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos()));
        std::fs::create_dir(&directory).unwrap();
        let config = directory.join("bus.conf");
        std::fs::write(&config, format!(
            "<busconfig><type>session</type><listen>unix:path={}</listen><auth>EXTERNAL</auth>\
            <policy context=\"default\"><allow send_destination=\"*\"/><allow own=\"*\"/></policy></busconfig>",
            directory.join("bus").display())).unwrap();
        let mut daemon = Command::new("dbus-daemon").arg(format!("--config-file={}", config.display()))
            .args(["--nofork", "--print-address"]).stdout(Stdio::piped()).spawn()
            .expect("the test requires an actual dbus-daemon");
        let mut address = String::new();
        BufReader::new(daemon.stdout.take().unwrap()).read_line(&mut address).unwrap();
        Self { directory, daemon, address: address.trim().into() }
    }
}
impl Drop for Bus {
    fn drop(&mut self) {
        let _ = self.daemon.kill();
        let _ = self.daemon.wait();
        let _ = std::fs::remove_dir_all(&self.directory);
    }
}

#[derive(Clone)]
struct Node { path: &'static str, holder: String }
#[zbus::interface(name = "org.a11y.atspi.Accessible")]
impl Node {
    #[zbus(property)]
    fn name(&self) -> &str { match self.path { ROOT => "alo", WINDOW => "Settings", _ => "Running code" } }
    #[zbus(property)]
    fn child_count(&self) -> i32 { i32::from(self.path != CONTROL) }
    fn get_children(&self) -> Vec<(String, OwnedObjectPath)> {
        vec![(self.holder.clone(), OwnedObjectPath::try_from(ROOT).unwrap())]
    }
    fn get_child_at_index(&self, _index: i32) -> (String, OwnedObjectPath) {
        (self.holder.clone(), OwnedObjectPath::try_from(if self.path == ROOT { WINDOW } else { CONTROL }).unwrap())
    }
    fn get_role(&self) -> u32 { match self.path { ROOT => 75, WINDOW => 23, _ => 43 } }
    fn get_state(&self) -> Vec<u32> { vec![(1 << 8) | (1 << 24) | (1 << 25), 0] }
    fn get_interfaces(&self) -> Vec<&str> { vec!["org.a11y.atspi.Accessible", "org.a11y.atspi.Action"] }
    fn get_attributes(&self) -> HashMap<&str, &str> {
        if self.path == WINDOW { HashMap::from([("alo-surface", "settings")]) } else { HashMap::new() }
    }
}

#[derive(Clone, Copy)]
enum Count { Signed(i32), Unsigned(u32), Boolean(bool) }
struct Action { machine: Vec<String>, localized: Vec<String>, count: Count, calls: Arc<Mutex<Vec<String>>> }
#[zbus::interface(name = "org.a11y.atspi.Action")]
impl Action {
    #[zbus(property, name = "NActions")]
    fn n_actions(&self) -> OwnedValue {
        match self.count { Count::Signed(n) => n.into(), Count::Unsigned(n) => n.into(), Count::Boolean(n) => n.into() }
    }
    fn get_actions(&self) -> Vec<(String, String, String)> {
        self.calls.lock().unwrap().push("localized-list".into());
        self.localized.iter().map(|name| (name.clone(), String::new(), String::new())).collect()
    }
    fn get_name(&self, index: i32) -> String {
        self.calls.lock().unwrap().push(format!("name:{index}"));
        usize::try_from(index).ok().and_then(|at| self.machine.get(at)).cloned().unwrap_or_default()
    }
    fn do_action(&self, index: i32) -> bool {
        self.calls.lock().unwrap().push(format!("do:{index}"));
        true
    }
}

struct Fixture { _service: Connection, tree: AccessibleSession, holder: String, calls: Arc<Mutex<Vec<String>>> }
impl Fixture {
    fn on(bus: &Bus, machine: &[&str], localized: &[&str], count: Count) -> Self {
        let service = Builder::address(bus.address.as_str()).unwrap()
            .name("org.a11y.atspi.Registry").unwrap().build().unwrap();
        let holder = service.unique_name().unwrap().to_string();
        for path in [ROOT, WINDOW, CONTROL] {
            service.object_server().at(path, Node { path, holder: holder.clone() }).unwrap();
        }
        let calls = Arc::new(Mutex::new(Vec::new()));
        service.object_server().at(CONTROL, Action { machine: machine.iter().map(|s| (*s).into()).collect(),
            localized: localized.iter().map(|s| (*s).into()).collect(), count, calls: calls.clone() }).unwrap();
        let property: OwnedValue = service.call_method(Some(holder.as_str()), CONTROL,
            Some("org.freedesktop.DBus.Properties"), "Get", &("org.a11y.atspi.Action", "NActions"))
            .unwrap().body().deserialize().unwrap();
        match count {
            Count::Signed(n) => assert_eq!(i32::try_from(property).unwrap(), n),
            Count::Unsigned(n) => assert_eq!(u32::try_from(property).unwrap(), n),
            Count::Boolean(n) => assert_eq!(bool::try_from(property).unwrap(), n),
        }
        let tree = AccessibleSession::at(&bus.address, Sandboxes::under(bus.directory.join("unused-proc"))).unwrap();
        Self { _service: service, tree, holder, calls }
    }
    fn node(&self) -> NodeAt { NodeAt { holder: self.holder.clone(), object: CONTROL.into() } }
    fn press(&self) -> Result<alo_adapters::PressedByHand, alo_adapters::Unpressed> {
        press_by_hand(&self.tree, Whose::AlosOwn { answers_as: &self.holder }, Some("settings"),
                      Pressable::Button, "Running code")
    }
}

#[test]
fn localized_action_labels_do_not_prevent_the_machine_click_or_change_its_index() {
    for localized in ["Running code", "Exécuter du code"] {
        let bus = Bus::start();
        let fixture = Fixture::on(&bus, &["show-menu", "click"], &["Menu", localized], Count::Signed(2));
        let pressed = fixture.press().expect("the exact machine click must remain usable in every language");
        assert!(!pressed.unanswered());
        assert_eq!(*fixture.calls.lock().unwrap(), ["name:0", "name:1", "do:1"]);
    }
}

#[test]
fn a_localized_click_label_cannot_authorize_an_unknown_machine_action() {
    let bus = Bus::start();
    let fixture = Fixture::on(&bus, &["run-arbitrary-action"], &["click"], Count::Signed(1));
    assert!(fixture.press().is_err());
    assert_eq!(*fixture.calls.lock().unwrap(), ["name:0"]);
}

#[test]
fn action_count_is_a_bounded_signed_wire_integer_before_any_action_is_read_or_done() {
    for count in [Count::Signed(-1), Count::Signed(65), Count::Unsigned(1), Count::Boolean(true)] {
        let bus = Bus::start();
        let fixture = Fixture::on(&bus, &["click"], &["click"], count);
        assert!(fixture.tree.actions(&fixture.node()).is_err());
        assert!(fixture.calls.lock().unwrap().is_empty());
    }
}

#[test]
fn zero_and_the_exact_action_count_limit_are_read_without_truncation() {
    let bus = Bus::start();
    let empty = Fixture::on(&bus, &[], &[], Count::Signed(0));
    assert_eq!(empty.tree.actions(&empty.node()).unwrap(), Vec::<String>::new());
    assert!(empty.calls.lock().unwrap().is_empty());
    drop(empty);
    let names = vec!["focus"; 64];
    let many = Fixture::on(&bus, &names, &names, Count::Signed(64));
    assert_eq!(many.tree.actions(&many.node()).unwrap(), names);
    assert_eq!(many.calls.lock().unwrap().len(), 64);
}
