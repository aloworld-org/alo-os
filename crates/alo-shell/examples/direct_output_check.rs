//! Read-only diagnostic of one explicitly supplied DRM card, never a modeset.

#[cfg(target_os = "linux")]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    use std::os::fd::AsFd;
    let mut args = std::env::args_os().skip(1);
    let path = args
        .next()
        .ok_or("usage: direct_output_check /dev/dri/cardN")?;
    if args.next().is_some() {
        return Err("usage: direct_output_check /dev/dri/cardN".into());
    }
    // Production receives its fd from the session manager. This developer probe
    // only queries resources; opening a primary node may implicitly make its first
    // opener DRM master, so run on a development device owned by this session.
    let file = std::fs::File::open(path)?;
    let output = alo_shell::discover_output(file.as_fd())?;
    println!(
        "discovered connector={} crtc={} size={:?}; no modeset performed",
        u32::from(output.connector),
        u32::from(output.crtc),
        output.mode.size()
    );
    Ok(())
}

#[cfg(not(target_os = "linux"))]
fn main() {
    eprintln!("direct-output discovery requires Linux");
    std::process::exit(1);
}
