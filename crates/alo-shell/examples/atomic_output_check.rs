//! Explicit device diagnostic: capability/schema discovery, never a modeset.

#[cfg(target_os = "linux")]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    use std::os::fd::AsFd;
    let mut args = std::env::args_os().skip(1);
    let path = args
        .next()
        .ok_or("usage: atomic_output_check /dev/dri/cardN")?;
    if args.next().is_some() {
        return Err("usage: atomic_output_check /dev/dri/cardN".into());
    }
    // Developer fixture only. Opening a card can implicitly acquire DRM master;
    // production must use DirectSession::with_device instead of this direct open.
    let file = std::fs::File::open(path)?;
    let result = alo_shell::discover_atomic_output(file.as_fd())?;
    println!(
        "atomic schema connector={} crtc={} primary={} formats={:?}; no atomic test or modeset",
        u32::from(result.output.connector),
        u32::from(result.output.crtc),
        u32::from(result.plane),
        result.formats
    );
    Ok(())
}

#[cfg(not(target_os = "linux"))]
fn main() {
    eprintln!("atomic-output discovery requires Linux");
    std::process::exit(1);
}
