//! Developer-only native graphics prerequisite and nested rendering probe.
//!
//! This is not installed in the OS image and exposes no agent capability.

#[cfg(target_os = "linux")]
mod prerequisites;
#[cfg(target_os = "linux")]
mod rendering;

/// Report a failed probe as a nonzero exit, without claiming a working desktop.
fn main() -> std::process::ExitCode {
    match run() {
        Ok(()) => std::process::ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("graphics check failed: {error}");
            std::process::ExitCode::FAILURE
        }
    }
}

/// Check the session first; drawing is an explicit, bounded developer action.
#[cfg(target_os = "linux")]
fn run() -> Result<(), String> {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    let render = match args.as_slice() {
        [mode] if mode == "check" => false,
        [mode] if mode == "render" => true,
        _ => return Err("usage: alo-graphics-check check|render".into()),
    };
    prerequisites::native_libraries()?;
    let socket = prerequisites::session_socket(
        std::env::var_os("XDG_RUNTIME_DIR").as_deref(),
        std::env::var_os("WAYLAND_DISPLAY").as_deref(),
    )?;
    prerequisites::connect(&socket)?;
    println!("Wayland socket reachable: {}", socket.display());
    if render {
        rendering::frame()?;
        println!("Smithay 0.7.0: nested GLES frame submitted; physical display unverified");
    }
    Ok(())
}

/// Linux native dependencies are deliberately absent on other build hosts.
#[cfg(not(target_os = "linux"))]
fn run() -> Result<(), String> {
    Err("this development probe requires Linux and a Wayland session".into())
}
