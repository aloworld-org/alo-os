//! The departures and the words the egress indicator's tests are written
//! against.
//!
//! The departures are made the way a verb makes one, and permitted by a real
//! `alo_egress::Indicator`, so a test about a lit status area is a test about
//! something that really left. The words are the machine's one vocabulary as
//! `alo-saying` collects it — what a shell really holds — so a line nobody
//! collected fails here rather than on somebody's screen.
//!
//! Nothing here is compiled into the crate: it exists under `cfg(test)` only.

#![expect(
    clippy::unwrap_used,
    reason = "in a fixture, a panic on an unexpected None or Err is the failure being reported"
)]

use std::time::{Duration, SystemTime};

use alo_capability::Grantee;
use alo_egress::{Destination, Leaving, Why};
use alo_models::{InferenceSource, Region};
use alo_strings::Strings;

/// Everything this machine can say, with nothing translated.
pub(crate) fn words() -> Strings {
    Strings::of(alo_saying::everything_this_machine_can_say().unwrap())
}

/// The moment every test here is written against.
pub(crate) fn noon() -> SystemTime {
    SystemTime::UNIX_EPOCH + Duration::from_secs(1_760_000_000)
}

/// The agent asking the questions below.
pub(crate) fn the_agent() -> Grantee {
    Grantee::named("@mail")
}

/// A question put to a provider.
pub(crate) fn asking_a_provider() -> Leaving {
    Leaving::asking(&the_agent(), &a_provider()).unwrap()
}

/// A provider, somewhere other than this machine.
pub(crate) fn a_provider() -> InferenceSource {
    InferenceSource::Hosted {
        provider: "alo".to_owned(),
        region: Region::Declared("the EU".to_owned()),
    }
}

/// The machine down the corridor, paired.
pub(crate) fn the_machine_down_the_corridor() -> InferenceSource {
    InferenceSource::PairedMachine {
        machine: "workstation-2".to_owned(),
    }
}

/// An agent fetching something from a host a verb named.
pub(crate) fn fetching() -> Leaving {
    Leaving::because(
        &Grantee::named("@files"),
        Why::Fetching,
        Destination::at("alo.example").unwrap(),
    )
}
