//! Explicit session-mediated discovery diagnostic; never modesets or switches VT.

#[cfg(target_os = "linux")]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args_os().skip(1);
    let path = args
        .next()
        .ok_or("usage: session_device_check /dev/dri/cardN")?;
    if args.next().is_some() {
        return Err("usage: session_device_check /dev/dri/cardN".into());
    }
    let mut session = alo_shell::DirectSession::new(path.into())?;
    let result = session.with_device(alo_shell::discover_output);
    session.shutdown()?;
    let output = result??;
    println!(
        "session acquired and released; connector={} size={:?}; no modeset",
        u32::from(output.connector),
        output.mode.size()
    );
    Ok(())
}

#[cfg(not(target_os = "linux"))]
fn main() {
    eprintln!("display sessions require Linux");
    std::process::exit(1);
}
