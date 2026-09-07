//! Explicit session-owned black scanout diagnostic; disables before returning.

#[cfg(target_os = "linux")]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args_os().skip(1);
    let path = args
        .next()
        .ok_or("usage: scanout_check /dev/dri/cardN --enable-and-disable")?;
    if !args.next().is_some_and(|arg| arg == "--enable-and-disable") || args.next().is_some() {
        return Err("usage: scanout_check /dev/dri/cardN --enable-and-disable".into());
    }
    let mut session = alo_shell::DirectSession::new(std::path::PathBuf::from(path))?;
    let result = session.with_device(|fd| -> Result<(), Box<dyn std::error::Error>> {
        let output = alo_shell::discover_atomic_output(fd)?;
        let resources = alo_shell::DisplayResources::allocate(fd, &output)?;
        let active = resources.activate()?;
        println!("blocking atomic enable accepted; initialized black buffer retained");
        active.disable()?;
        println!("blocking atomic disable accepted; resources released");
        Ok(())
    });
    let shutdown = session.shutdown();
    let operation = match result {
        Ok(result) => result,
        Err(error) => Err(error.into()),
    };
    match (operation, shutdown) {
        (Ok(()), Ok(())) => Ok(()),
        (Err(error), Ok(())) => Err(error),
        (Ok(()), Err(error)) => Err(error.into()),
        (Err(error), Err(cleanup)) => Err(format!("{error}; session cleanup: {cleanup}").into()),
    }
}

#[cfg(not(target_os = "linux"))]
fn main() {
    eprintln!("scanout diagnostic requires Linux");
    std::process::exit(1);
}
