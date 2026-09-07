//! Explicit device diagnostic: schema discovery and optional unbound allocation.

#[cfg(target_os = "linux")]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    use std::os::fd::AsFd;
    let mut args = std::env::args_os().skip(1);
    let path = args
        .next()
        .ok_or("usage: atomic_output_check /dev/dri/cardN [--allocate|--test-only]")?;
    let option = args.next();
    let test = option.as_deref().is_some_and(|arg| arg == "--test-only");
    let allocate = match option {
        None => false,
        Some(arg) if arg == "--allocate" || arg == "--test-only" => true,
        Some(_) => {
            return Err(
                "usage: atomic_output_check /dev/dri/cardN [--allocate|--test-only]".into(),
            );
        }
    };
    if args.next().is_some() {
        return Err("usage: atomic_output_check /dev/dri/cardN [--allocate|--test-only]".into());
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
    if allocate {
        let resources = alo_shell::DisplayResources::allocate(file.as_fd(), &result)?;
        println!(
            "unbound framebuffer={} mode_blob={}; no mapping, atomic test or modeset",
            u32::from(resources.framebuffer()),
            resources.mode_blob(),
        );
        if test {
            resources.test_and_release()?;
            println!("atomic TEST_ONLY accepted; no scanout change or reservation");
        } else {
            resources.release()?;
        }
        println!("all display resources released");
    }
    Ok(())
}

#[cfg(not(target_os = "linux"))]
fn main() {
    eprintln!("atomic-output discovery requires Linux");
    std::process::exit(1);
}
