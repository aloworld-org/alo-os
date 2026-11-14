//! `alo-walk`: boot a written alo disk in qemu, walk it, and keep what it saw.
//!
//! ```text
//! alo-walk --disk alo.raw --firmware OVMF_CODE_4M.fd --variables vars.fd \
//!          --out the-walk --scratch walk-scratch [--no-kvm]
//! ```
//!
//! Into `--out`, which is what a workflow uploads: one picture per look,
//! `report.txt`, and `serial.log` — the machine's serial log **with every
//! secret the walk typed taken out**. The raw log stays in `--scratch`, which
//! is never uploaded. The report is also printed, and appended as a table to
//! `$GITHUB_STEP_SUMMARY` where that is set.
//!
//! Exits 0 when the walk passed, 1 when it did not, and 2 when it could not
//! walk at all.

use std::path::PathBuf;
use std::process::ExitCode;

use alo_walking::qemu::Qemu;
use alo_walking::redacting;
use alo_walking::{Booting, Pace, the_first_walk, walk};

/// What the command line named.
struct Asked {
    /// The machine to boot.
    booting: Booting,
    /// Where what is kept goes.
    out: PathBuf,
    /// Where what is not kept goes.
    scratch: PathBuf,
}

/// Boot, walk, keep, and say.
fn main() -> ExitCode {
    let asked = match asked(std::env::args().skip(1).collect()) {
        Ok(asked) => asked,
        Err(why) => {
            eprintln!("alo-walk: {why}");
            return ExitCode::from(2);
        }
    };
    for directory in [&asked.out, &asked.scratch] {
        if let Err(why) = std::fs::create_dir_all(directory) {
            eprintln!("alo-walk: {} could not be made: {why}", directory.display());
            return ExitCode::from(2);
        }
    }
    let the_walk = the_first_walk();
    let secret = |name: &str| std::env::var(name).ok().filter(|it| !it.is_empty());
    let mut machine = match Qemu::boot(&asked.booting, &asked.out, &asked.scratch) {
        Ok(machine) => machine,
        Err(why) => {
            eprintln!("alo-walk: {why}");
            return ExitCode::from(2);
        }
    };
    let report = walk(&mut machine, &the_walk, &secret, Pace::UNDER_KVM);
    machine.stop();

    let secrets: Vec<String> = the_walk.secrets().into_iter().filter_map(&secret).collect();
    let mut kept_everything = true;
    match std::fs::read(&asked.booting.serial) {
        Ok(raw) => {
            let cleaned = redacting::without(&raw, &secrets);
            if let Err(why) = std::fs::write(asked.out.join("serial.log"), cleaned) {
                eprintln!("alo-walk: the serial log was not kept: {why}");
                kept_everything = false;
            }
        }
        Err(why) => {
            eprintln!("alo-walk: the serial log could not be read: {why}");
            kept_everything = false;
        }
    }
    let text = report.as_text();
    print!("{text}");
    if let Err(why) = std::fs::write(asked.out.join("report.txt"), &text) {
        eprintln!("alo-walk: the report was not kept: {why}");
        kept_everything = false;
    }
    if let Some(summary) = std::env::var_os("GITHUB_STEP_SUMMARY") {
        let appended = std::fs::OpenOptions::new()
            .append(true)
            .create(true)
            .open(summary)
            .and_then(|mut file| {
                std::io::Write::write_all(&mut file, report.as_markdown().as_bytes())
            });
        if let Err(why) = appended {
            eprintln!("alo-walk: the run's summary was not written: {why}");
        }
    }
    if report.passed() && kept_everything {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}

/// Read the command line.
fn asked(arguments: Vec<String>) -> Result<Asked, String> {
    let mut disk = None;
    let mut firmware = None;
    let mut variables = None;
    let mut out = None;
    let mut scratch = None;
    let mut kvm = true;
    let mut given = arguments.into_iter();
    while let Some(argument) = given.next() {
        let mut value = || {
            given
                .next()
                .map(PathBuf::from)
                .ok_or_else(|| format!("{argument} needs a value"))
        };
        match argument.as_str() {
            "--disk" => disk = Some(value()?),
            "--firmware" => firmware = Some(value()?),
            "--variables" => variables = Some(value()?),
            "--out" => out = Some(value()?),
            "--scratch" => scratch = Some(value()?),
            "--no-kvm" => kvm = false,
            other => return Err(format!("{other} is not something alo-walk takes")),
        }
    }
    let needed = |it: Option<PathBuf>, name: &str| it.ok_or_else(|| format!("{name} is needed"));
    let scratch = needed(scratch, "--scratch")?;
    let absolute = |it: PathBuf| std::path::absolute(&it).unwrap_or(it);
    let scratch = absolute(scratch);
    Ok(Asked {
        booting: Booting {
            disk: absolute(needed(disk, "--disk")?),
            firmware: absolute(needed(firmware, "--firmware")?),
            variables: absolute(needed(variables, "--variables")?),
            serial: scratch.join("serial.raw.log"),
            qmp_port: 4444,
            kvm,
            memory: 6144,
            processors: 4,
        },
        out: absolute(needed(out, "--out")?),
        scratch,
    })
}
