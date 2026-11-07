//! A native-libc worker; the public portal daemon never links a media engine.

#[cfg(target_os = "linux")]
mod private_engine;
#[cfg(target_os = "linux")]
mod publishing;

#[cfg(target_os = "linux")]
fn main() -> std::process::ExitCode {
    match publishing::run() {
        Ok(()) => std::process::ExitCode::SUCCESS,
        Err(why) => {
            eprintln!("alo-capture-provider: {why}");
            std::process::ExitCode::FAILURE
        }
    }
}

#[cfg(not(target_os = "linux"))]
fn main() -> std::process::ExitCode {
    std::process::ExitCode::FAILURE
}
