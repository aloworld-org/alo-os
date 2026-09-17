//! The broker, as a machine runs it: root, in the person's group, holding no
//! capability, with one door.
//!
//! `alo_brokerd` is every decision; this is the process that makes them in
//! order. It is deliberately thin, because the only thing here a test cannot
//! reach is the machine's own paths and the group this process was started in.
//!
//! # It says one kind of thing, to a service log
//!
//! Nothing written here is read by the person using the machine. What they read
//! is said by the surface that asked, in their own language, from the one-word
//! answer the door gave. The log gets that word, and the reason a start was
//! refused, in English for whoever stands the machine up.

/// What this process does on the machine alo OS is for.
#[cfg(unix)]
mod running {
    use std::path::Path;
    use std::process::ExitCode;

    use alo_broker::our_group;
    use alo_brokerd::{Carriers, Network, Places, Proxy, Started, started};
    use alo_networks::network_manager::OnThisMachine;
    use alo_networks::proxy_file::{THE_MACHINES_PROXY, THE_WANTED_PROXY};

    /// Open the door and answer whoever knocks, until this service is stopped.
    ///
    /// `FAILURE` is a broker that did not open its door, and the log says why.
    /// A request refused is not a failure of this process: it is an answer, and
    /// it was written down before it was given.
    pub fn main() -> ExitCode {
        let places = Places::on_this_machine();
        let carriers = |logins: &alo_brokerd::Logins| {
            Carriers::of(
                Network::against(OnThisMachine),
                Proxy::handed_over(
                    Path::new(THE_WANTED_PROXY),
                    Path::new(THE_MACHINES_PROXY),
                    logins.person,
                ),
            )
        };
        let Started {
            listening,
            mut broker,
        } = match started(&places, our_group(), carriers) {
            Ok(started) => started,
            Err(why) => {
                eprintln!("alo-brokerd: the broker's door did not open: {why}");
                return ExitCode::FAILURE;
            }
        };

        eprintln!(
            "alo-brokerd: the door at {} is open to group {} and hears one user; this process \
             holds no capability, and what it can do is its closed list",
            places.door.display(),
            our_group()
        );

        loop {
            match listening.answer_one(&mut broker) {
                Ok(answer) => eprintln!("alo-brokerd: answered {}", answer.written()),
                // A caller that went away before its answer. What it asked was
                // written down; the next knock is the one that matters.
                Err(why) => eprintln!("alo-brokerd: a caller was not answered: {why}"),
            }
        }
    }
}

/// The broker, on the machine it is for.
#[cfg(unix)]
fn main() -> std::process::ExitCode {
    running::main()
}

/// Anywhere else, and it says so rather than pretending.
#[cfg(not(unix))]
fn main() -> std::process::ExitCode {
    eprintln!(
        "alo-brokerd listens at a Unix socket and there is none on this host: alo OS is Linux \
         (ADR 0011)"
    );
    std::process::ExitCode::FAILURE
}
