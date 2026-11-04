//! Run the real frontend and grants writer on an explicitly private test bus.
//!
//! This example is not installed in an image. It uses real process identification
//! and real native backends, so it can test a Flatpak caller rather than a fake
//! `.flatpak-info` fixture. Arguments: private bus address and private state folder.

#[cfg(target_os = "linux")]
/// The Linux-only real frontend assembled for a private verification session.
mod check {
    use alo_portals::{Allowed, Backend, KeepsSecrets, NotKept, Request, Sandboxes};
    use std::os::unix::fs::{DirBuilderExt, MetadataExt};
    use std::{path::PathBuf, sync::Arc, time::SystemTime};

    /// A save probe never provides passwords or a fallback keyring.
    struct NoSecrets;
    impl KeepsSecrets for NoSecrets {
        fn hand_over(
            &self,
            _request: &Request,
            _grants: &alo_capability::Grants,
            _now: SystemTime,
            _into: &mut dyn std::io::Write,
        ) -> Result<Allowed, NotKept> {
            Err(NotKept::Unavailable)
        }
    }

    /// Serve until the private verification session terminates this process.
    pub(super) fn run() -> Result<(), Box<dyn std::error::Error>> {
        let mut args = std::env::args().skip(1);
        let address = args.next().ok_or("a private bus address is required")?;
        let state = PathBuf::from(args.next().ok_or("a private state directory is required")?);
        let application = args.next();
        if !state.is_absolute() || args.next().is_some() {
            return Err("use a bus address and an absolute private state directory".into());
        }
        match std::fs::DirBuilder::new().mode(0o700).create(&state) {
            Ok(()) => {}
            Err(why) if why.kind() == std::io::ErrorKind::AlreadyExists => {}
            Err(why) => return Err(why.into()),
        }
        let about = std::fs::symlink_metadata(&state)?;
        if !about.is_dir()
            || about.mode() & 0o077 != 0
            || about.uid() != alo_remembering::this_person()
        {
            return Err(
                "the probe state directory must already be private and owned by this login".into(),
            );
        }
        if address == "--revoke" {
            let application = application.ok_or("the application to revoke is required")?;
            let (_, removed) = alo_remembering::updated(
                &state.join("grants.toml"),
                SystemTime::now(),
                |grants| {
                    Ok::<_, alo_remembering::NotRemembered>(grants.revoke_everything_for(
                        &alo_capability::Applicant::named(&application).grantee(),
                    ))
                },
            )?;
            if removed == 0 {
                return Err("no selected permission was found to revoke".into());
            }
            println!("native-save-grant-revoked: {removed}");
            return Ok(());
        }
        if application.is_some() {
            return Err("unexpected native frontend arguments".into());
        }
        let machine = alo_portald::ThisMachine::reading(state.join("grants.toml"), state.clone());
        let record = alo_portals::AnswersFile::opened(&state.join("answers.jsonl"))?;
        let _served = Backend::answering_from(
            Arc::new(machine),
            Arc::new(NoSecrets),
            Sandboxes::under("/proc"),
            Arc::new(record),
        )
        .serve_on(&address)
        .map_err(|why| format!("the private frontend could not be served: {why:?}"))?;
        println!("native-save-frontend-ready");
        loop {
            std::thread::park();
        }
    }
}

#[cfg(target_os = "linux")]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    check::run()
}

#[cfg(not(target_os = "linux"))]
fn main() {
    eprintln!("The native save integration probe requires Linux.");
}
