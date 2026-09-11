//! The questions alo OS asks of a service unit, and nothing systemd asks.
//!
//! [`crate::Unit`] is the text; this is the handful of things about a service
//! that two ADRs and a contract turn on — who it runs as, what it holds, what it
//! is started after, and whether it stays after it exits. Every one of them is
//! a sentence somewhere in `docs/` that would otherwise only be prose.
//!
//! # It asks about five settings and deliberately not about the rest
//!
//! A unit has a hundred directives and this reads seven of them. That is not a
//! gap to fill in later: the value of a check is that failing it means
//! something, and a crate that had an opinion about `ProtectHome` would be this
//! repository deciding hardening by accident, in the file that exists to hold it
//! to decisions it already made.

use crate::refusing::NotAService;
use crate::unit::Unit;

/// The section a unit's own ordering and requirements are in.
const UNIT: &str = "Unit";

/// The section a service's own settings are in.
const SERVICE: &str = "Service";

/// The section that says what pulls a unit in at boot.
const INSTALL: &str = "Install";

/// The user a service that runs as nobody in particular runs as.
pub const ROOT: &str = "root";

/// One of alo OS's two services, as its unit file describes it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Service {
    /// The file it was read from, which is what a refusal names.
    called: String,
    /// The sections and keys themselves.
    unit: Unit,
}

impl Service {
    /// This unit, read as a service.
    ///
    /// # Errors
    ///
    /// [`NotAService::NoServiceSection`] and [`NotAService::NothingToStart`] —
    /// a unit with neither is a unit systemd would not start, and checking the
    /// rest of it would be checking a file that never runs.
    pub fn of(called: &str, unit: Unit) -> Result<Self, NotAService> {
        if !unit.has(SERVICE) {
            return Err(NotAService::NoServiceSection {
                called: called.to_owned(),
            });
        }
        if unit.one(SERVICE, "ExecStart").is_none_or(str::is_empty) {
            return Err(NotAService::NothingToStart {
                called: called.to_owned(),
            });
        }
        Ok(Self {
            called: called.to_owned(),
            unit,
        })
    }

    /// The unit file this was read from.
    #[must_use]
    pub fn called(&self) -> &str {
        &self.called
    }

    /// What it starts.
    #[must_use]
    pub fn runs(&self) -> &str {
        self.unit.one(SERVICE, "ExecStart").unwrap_or_default()
    }

    /// How systemd treats the process: `oneshot`, `exec`, and the rest.
    #[must_use]
    pub fn kind(&self) -> Option<&str> {
        self.unit.one(SERVICE, "Type")
    }

    /// Whether the unit stays active once the process has exited.
    ///
    /// The loader's whole shape: it loads, it pins and it stops, and the
    /// machine keeps the boundary because the pin holds it. A `systemctl status`
    /// that went inactive would be telling somebody the opposite.
    #[must_use]
    pub fn stays_after_exiting(&self) -> bool {
        matches!(
            self.unit.one(SERVICE, "RemainAfterExit"),
            Some("yes" | "true" | "on" | "1")
        )
    }

    /// The login it runs as, if the unit says.
    #[must_use]
    pub fn as_login(&self) -> Option<&str> {
        self.unit.one(SERVICE, "User")
    }

    /// The group it runs in, if the unit says.
    #[must_use]
    pub fn in_group(&self) -> Option<&str> {
        self.unit.one(SERVICE, "Group")
    }

    /// The groups it is in beside that one.
    #[must_use]
    pub fn beside_groups(&self) -> Vec<&str> {
        self.unit.listed(SERVICE, "SupplementaryGroups")
    }

    /// The most this service can ever hold.
    #[must_use]
    pub fn bounded_to(&self) -> Vec<&str> {
        self.unit.listed(SERVICE, "CapabilityBoundingSet")
    }

    /// What it is given to start with.
    #[must_use]
    pub fn given(&self) -> Vec<&str> {
        self.unit.listed(SERVICE, "AmbientCapabilities")
    }

    /// Whether the unit *says* it holds nothing, rather than merely not saying
    /// that it does.
    ///
    /// Both lines, both empty. A unit that simply never mentions capabilities
    /// gives a service running as an ordinary person none either — and the
    /// difference is that the next person to edit it can see this one.
    #[must_use]
    pub fn holds_nothing(&self) -> bool {
        self.unit.says(SERVICE, "CapabilityBoundingSet")
            && self.unit.says(SERVICE, "AmbientCapabilities")
            && self.bounded_to().is_empty()
            && self.given().is_empty()
    }

    /// Whether systemd hands this service's own control group over to it.
    ///
    /// A turn is a control group made under the service's own (ADR 0015), and a
    /// service that runs as a person can only make one where systemd has
    /// delegated the subtree — otherwise the unit's cgroup is root's and
    /// `mkdir` answers `EACCES`. `Delegate=` takes a boolean *or* a list of
    /// controller names, so anything but the four words for no is delegation.
    #[must_use]
    pub fn delegates_control_groups(&self) -> bool {
        match self.unit.one(SERVICE, "Delegate") {
            None | Some("" | "no" | "false" | "off" | "0") => false,
            Some(_) => true,
        }
    }

    /// The directories under `/run` systemd makes for this service, in order.
    ///
    /// Relative to `/run`, which is how the setting is written: `alo/1000` is
    /// `/run/alo/1000`.
    #[must_use]
    pub fn runtime_directories(&self) -> Vec<&str> {
        self.unit.listed(SERVICE, "RuntimeDirectory")
    }

    /// The mode it makes them with, as the unit spells it.
    ///
    /// Text rather than a number, because what a check is about is the line
    /// somebody wrote: a unit that says nothing gets systemd's `0755`, and the
    /// difference between that and a chosen mode is the whole question.
    #[must_use]
    pub fn runtime_directory_mode(&self) -> Option<&str> {
        self.unit.one(SERVICE, "RuntimeDirectoryMode")
    }

    /// The units this one will not start without.
    #[must_use]
    pub fn needs(&self) -> Vec<&str> {
        self.unit.listed(UNIT, "Requires")
    }

    /// The units this one is started after.
    #[must_use]
    pub fn after(&self) -> Vec<&str> {
        self.unit.listed(UNIT, "After")
    }

    /// The units this one's life is tied to: stopping one of them stops this.
    ///
    /// Different from [`Service::needs`] in the half that matters here.
    /// `Requires=` says *do not start without it*; `BindsTo=` says *and stop
    /// when it stops*, which is how a service started by signing in is a service
    /// that ends when the last session does.
    #[must_use]
    pub fn bound_to(&self) -> Vec<&str> {
        self.unit.listed(UNIT, "BindsTo")
    }

    /// Every `KEY=value` the unit puts in the service's environment.
    ///
    /// systemd separates several pairs on one line by spaces and accumulates
    /// over repeated assignments, which is [`Unit::listed`] exactly — including
    /// an empty assignment clearing what came before it.
    #[must_use]
    pub fn environment(&self) -> Vec<&str> {
        self.unit.listed(SERVICE, "Environment")
    }

    /// The addresses this service may reach, and be reached from.
    ///
    /// systemd's IP access list is a filter on this unit's own control group,
    /// applied in the kernel in both directions. It is the one setting in a unit
    /// file that makes *it makes no connection of its own* enforced rather than
    /// asserted — a service under `IPAddressDeny=any` does not fail politely at
    /// an update check, it does not leave the machine.
    #[must_use]
    pub fn may_reach(&self) -> Vec<&str> {
        self.unit.listed(SERVICE, "IPAddressAllow")
    }

    /// The addresses it may not.
    #[must_use]
    pub fn may_not_reach(&self) -> Vec<&str> {
        self.unit.listed(SERVICE, "IPAddressDeny")
    }

    /// The units this one is started before.
    #[must_use]
    pub fn before(&self) -> Vec<&str> {
        self.unit.listed(UNIT, "Before")
    }

    /// What pulls this unit in at boot, which is what `systemctl enable` writes.
    #[must_use]
    pub fn wanted_by(&self) -> Vec<&str> {
        self.unit.listed(INSTALL, "WantedBy")
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;

    /// A service unit with everything this file reads in it.
    const A_LOADER: &str = "\
[Unit]
BindsTo=user@1000.service
Before=alo-agentd.service
RequiresMountsFor=/sys/fs/bpf

[Service]
Type=oneshot
RemainAfterExit=yes
User=root
Group=alo-agent
CapabilityBoundingSet=CAP_BPF CAP_SYS_ADMIN
ExecStart=/usr/libexec/alo-boundaryd

[Install]
WantedBy=multi-user.target
";

    /// A service read as one, for a test that wants the value rather than the
    /// reading.
    fn read(called: &str, text: &str) -> Service {
        Service::of(called, Unit::read(text).unwrap()).unwrap()
    }

    /// Everything the unit says comes back as what it says, which is the whole
    /// of what this file is.
    #[test]
    fn a_service_answers_what_its_unit_says() {
        let service = read("alo-boundaryd.service", A_LOADER);

        assert_eq!(service.called(), "alo-boundaryd.service");
        assert_eq!(service.runs(), "/usr/libexec/alo-boundaryd");
        assert_eq!(service.kind(), Some("oneshot"));
        assert!(service.stays_after_exiting());
        assert_eq!(service.as_login(), Some(ROOT));
        assert_eq!(service.in_group(), Some("alo-agent"));
        assert_eq!(service.bounded_to(), vec!["CAP_BPF", "CAP_SYS_ADMIN"]);
        assert_eq!(service.before(), vec!["alo-agentd.service"]);
        assert_eq!(service.bound_to(), vec!["user@1000.service"]);
        assert_eq!(service.wanted_by(), vec!["multi-user.target"]);
    }

    /// **An IP access list is read as two lists, and a unit that says nothing
    /// has neither.** The difference between them is the whole of what makes a
    /// silent service silent: an allow list with no `IPAddressDeny=any` under it
    /// filters nothing at all.
    #[test]
    fn what_a_service_may_reach_is_what_its_unit_says() {
        let bounded = read(
            "bounded.service",
            "[Service]\nExecStart=/usr/bin/x\nIPAddressAllow=localhost\nIPAddressDeny=any\n",
        );
        let open = read("open.service", "[Service]\nExecStart=/usr/bin/x\n");

        assert_eq!(bounded.may_reach(), vec!["localhost"]);
        assert_eq!(bounded.may_not_reach(), vec!["any"]);
        assert!(open.may_reach().is_empty());
        assert!(open.may_not_reach().is_empty());
    }

    /// The environment a unit states comes back pair by pair, accumulated over
    /// the repeated assignments systemd accumulates over.
    #[test]
    fn the_environment_is_every_pair_the_unit_states() {
        let service = read(
            "session.service",
            "[Service]\nExecStart=/usr/bin/x\nEnvironment=XDG_RUNTIME_DIR=/run/user/1000\nEnvironment=DBUS_SESSION_BUS_ADDRESS=unix:path=/run/user/1000/bus\n",
        );

        assert_eq!(
            service.environment(),
            vec![
                "XDG_RUNTIME_DIR=/run/user/1000",
                "DBUS_SESSION_BUS_ADDRESS=unix:path=/run/user/1000/bus",
            ]
        );
        assert!(
            read("bare.service", "[Service]\nExecStart=/usr/bin/x\n")
                .environment()
                .is_empty()
        );
    }

    /// **A unit with no `[Service]` is refused**, because systemd would not
    /// start it and checking the rest would be checking a file that never runs.
    #[test]
    fn a_unit_with_no_service_section_is_refused() {
        let refused = Service::of(
            "nothing.service",
            Unit::read("[Unit]\nDescription=x\n").unwrap(),
        )
        .unwrap_err();
        assert!(
            matches!(&refused, NotAService::NoServiceSection { called } if called == "nothing.service"),
            "{refused}"
        );
    }

    /// **And a service with nothing to start is refused too**, which is the
    /// mistake that really happens: a section somebody wrote and a line they
    /// meant to fill in.
    #[test]
    fn a_service_with_nothing_to_start_is_refused() {
        let refused = Service::of(
            "empty.service",
            Unit::read("[Service]\nType=oneshot\n").unwrap(),
        )
        .unwrap_err();
        assert!(
            matches!(refused, NotAService::NothingToStart { .. }),
            "{refused}"
        );
    }

    /// And an `ExecStart` assigned nothing is the same mistake wearing a line.
    #[test]
    fn an_execstart_assigned_nothing_is_nothing_to_start() {
        let refused = Service::of(
            "blank.service",
            Unit::read("[Service]\nExecStart=\n").unwrap(),
        )
        .unwrap_err();
        assert!(
            matches!(refused, NotAService::NothingToStart { .. }),
            "{refused}"
        );
    }

    /// **Holding nothing is two empty lines, not two missing ones.** A service
    /// that never mentions capabilities holds none either; the difference is
    /// whether the next person to read the unit can see that it was decided.
    #[test]
    fn saying_it_holds_nothing_is_not_the_same_as_not_saying() {
        let said = read(
            "said.service",
            "[Service]\nExecStart=/usr/bin/x\nCapabilityBoundingSet=\nAmbientCapabilities=\n",
        );
        let unsaid = read("unsaid.service", "[Service]\nExecStart=/usr/bin/x\n");

        assert!(said.holds_nothing());
        assert!(!unsaid.holds_nothing());
        assert!(
            unsaid.bounded_to().is_empty(),
            "and it holds nothing either"
        );
    }

    /// A service that is given something does not hold nothing, however the
    /// bounding set reads.
    #[test]
    fn a_service_given_a_capability_does_not_hold_nothing() {
        let service = read(
            "given.service",
            "[Service]\nExecStart=/usr/bin/x\nCapabilityBoundingSet=\nAmbientCapabilities=CAP_NET_ADMIN\n",
        );
        assert!(!service.holds_nothing());
        assert_eq!(service.given(), vec!["CAP_NET_ADMIN"]);
    }

    /// **Delegation is read as systemd reads it**: a boolean or a list of
    /// controllers, and only the words for no mean no. A unit that never says
    /// it is a unit whose service cannot make a control group of its own.
    #[test]
    fn delegating_control_groups_is_said_or_it_is_not_true() {
        let plain = read(
            "yes.service",
            "[Service]\nExecStart=/usr/bin/x\nDelegate=yes\n",
        );
        let listed = read(
            "listed.service",
            "[Service]\nExecStart=/usr/bin/x\nDelegate=pids memory\n",
        );
        let refused = read(
            "no.service",
            "[Service]\nExecStart=/usr/bin/x\nDelegate=no\n",
        );
        let silent = read("silent.service", "[Service]\nExecStart=/usr/bin/x\n");

        assert!(plain.delegates_control_groups());
        assert!(listed.delegates_control_groups());
        assert!(!refused.delegates_control_groups());
        assert!(!silent.delegates_control_groups());
    }

    /// A runtime directory is read relative to `/run`, the way the setting is
    /// written, and the mode comes back as the line rather than as a number.
    #[test]
    fn a_runtime_directory_is_what_the_unit_names() {
        let service = read(
            "door.service",
            "[Service]\nExecStart=/usr/bin/x\nRuntimeDirectory=alo/1000\nRuntimeDirectoryMode=0750\n",
        );

        assert_eq!(service.runtime_directories(), vec!["alo/1000"]);
        assert_eq!(service.runtime_directory_mode(), Some("0750"));
    }

    /// A service that stays after exiting says so in one of the four words
    /// systemd accepts, and a service that does not say it does not stay.
    #[test]
    fn staying_after_exiting_is_said_or_it_is_not_true() {
        let stays = read(
            "stays.service",
            "[Service]\nExecStart=/usr/bin/x\nRemainAfterExit=true\n",
        );
        let goes = read("goes.service", "[Service]\nExecStart=/usr/bin/x\n");

        assert!(stays.stays_after_exiting());
        assert!(!goes.stays_after_exiting());
    }
}
