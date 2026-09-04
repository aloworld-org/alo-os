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
        assert_eq!(service.wanted_by(), vec!["multi-user.target"]);
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
