//! Compiling the half that runs inside the kernel, and putting it where the
//! half that runs here can find it.
//!
//! `crates/alo-bounding-kernel` targets `bpfel-unknown-none`: a virtual machine
//! inside the kernel, with no prebuilt `core` to link against, which is why the
//! build below is nightly with `-Z build-std=core`. None of that reaches
//! anybody building alo OS — they run `cargo build`, and this happens.
//!
//! # Two things this deliberately does not do
//!
//! **It does not run on a host that is not Linux.** A BPF object is of no use
//! to a machine that cannot load one, the crate around it compiles to nothing
//! there, and requiring a nightly compiler and a BPF linker on a developer's
//! laptop to build a crate that will be empty is a cost with nothing on the
//! other side of it.
//!
//! **It does not inherit this build's own settings.** A nested `cargo` picks up
//! `RUSTFLAGS`, the toolchain and the target from the environment it is started
//! in, and every one of those is wrong for it — the flags are for this machine,
//! the toolchain is stable, and the target is x86. They are removed rather than
//! overridden, so a flag added to the outer build later cannot quietly arrive
//! here.

use std::{env, error::Error, fs, path::PathBuf, process::Command};

/// What the compiled program is called, in `OUT_DIR` and in the kernel half's
/// own target directory.
const OBJECT: &str = "alo-bounding-kernel";

/// The package that is compiled, relative to this crate.
const KERNEL_HALF: &str = "../alo-bounding-kernel";

/// The environment a nested cargo must not inherit.
///
/// Everything cargo tells a build script about *this* compilation, which is a
/// different target, a different channel and a different set of flags from the
/// one being started.
const NOT_INHERITED: [&str; 8] = [
    "CARGO_ENCODED_RUSTFLAGS",
    "RUSTFLAGS",
    "RUSTC",
    "RUSTDOC",
    "RUSTC_WORKSPACE_WRAPPER",
    "RUSTC_WRAPPER",
    "RUSTUP_TOOLCHAIN",
    "CARGO_BUILD_TARGET",
];

fn main() -> Result<(), Box<dyn Error>> {
    println!("cargo::rerun-if-changed={KERNEL_HALF}/src");
    println!("cargo::rerun-if-changed={KERNEL_HALF}/Cargo.toml");
    println!("cargo::rerun-if-changed=../alo-bounding-map/src");

    let out = PathBuf::from(env::var_os("OUT_DIR").ok_or("cargo did not set OUT_DIR")?);
    let landing = out.join(OBJECT);

    if env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("linux") {
        // Nothing here can be loaded, and `lib.rs` compiles to nothing. The
        // file is still written, because an empty file is a clearer thing for
        // the next person to find than a missing one.
        fs::write(&landing, [])?;
        return Ok(());
    }

    let target = out.join("kernel");
    let mut cargo = Command::new("cargo");
    for setting in NOT_INHERITED {
        cargo.env_remove(setting);
    }
    // Started *in* the kernel half's own directory rather than pointed at its
    // manifest, because that is what makes its `rust-toolchain.toml` the one
    // that decides: rustup picks a toolchain from where it was invoked, and
    // that file is the pinned nightly whose LLVM matches the `bpf-linker` on
    // the machine. Naming a channel here instead would silently overrule it.
    let built = cargo
        .current_dir(KERNEL_HALF)
        .args(["build", "--release"])
        .args(["--target", "bpfel-unknown-none"])
        .args(["-Z", "build-std=core"])
        .arg("--target-dir")
        .arg(&target)
        .status()?;
    if !built.success() {
        return Err(
            "the half that runs in the kernel did not compile. It needs a nightly toolchain \
             with `rust-src`, and `bpf-linker` on the path; `docs/autonomy/LOOP.md` has what \
             installing those actually takes."
                .into(),
        );
    }

    fs::copy(
        target.join("bpfel-unknown-none/release").join(OBJECT),
        &landing,
    )?;
    Ok(())
}
