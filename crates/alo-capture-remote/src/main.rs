//! Descriptor-only worker: called by the portal's media adapter, never by a model.

#[cfg(target_os = "linux")]
fn main() -> std::process::ExitCode {
    use std::os::fd::AsFd;
    let result = (|| -> Result<(), Box<dyn std::error::Error>> {
        let mut args = std::env::args().skip(1);
        let id = args
            .next()
            .ok_or("a selected node ID is required")?
            .parse()?;
        let serial = args
            .next()
            .ok_or("a selected node serial is required")?
            .parse()?;
        let application = args
            .next()
            .ok_or("an authenticated application ID is required")?;
        if args.next().is_some() {
            return Err("unexpected capture arguments".into());
        }
        let descriptor = alo_capture_remote::restricted_remote(
            alo_capture_protocol::MediaNode { id, serial },
            &application,
        )?;
        alo_capture_protocol::send_descriptor(&std::io::stdout(), descriptor.as_fd())?;
        Ok(())
    })();
    match result {
        Ok(()) => std::process::ExitCode::SUCCESS,
        Err(why) => {
            eprintln!("alo-capture-remote: {why}");
            std::process::ExitCode::FAILURE
        }
    }
}

#[cfg(not(target_os = "linux"))]
fn main() -> std::process::ExitCode {
    eprintln!("The capture media helper requires Linux.");
    std::process::ExitCode::FAILURE
}
