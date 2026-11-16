//! **What `the-shipped-names.txt` should hold**, read from the desktop entries
//! the shipped packages exported, and whether the committed file holds it.
//!
//! ```text
//! alo-shipped-names --identifiers
//! alo-shipped-names <applications folder>...
//! ```
//!
//! The first prints each shipped application's source and identifier, one a
//! line, for `.github/workflows/shipped-names.yml` to install. The second reads
//! the entries in each folder given, earlier folders winning, with the
//! machine's own reader (`alo_installed::read`); prints what the file should
//! hold; and exits 1 when the file built into this crate holds anything else.
//! **It never writes the file**: a person copies what it printed.

use std::path::PathBuf;
use std::process::ExitCode;

use alo_software::shipped_names::THE_NAMES;
use alo_software::{Shipped, as_their_entries_name_them};

/// Prints what the file should hold, and exits 1 where it holds anything else.
fn main() -> ExitCode {
    match run() {
        Ok(code) => code,
        Err(why) => {
            eprintln!("alo-shipped-names: {why}");
            ExitCode::from(2)
        }
    }
}

/// What was asked for, done: the exit code, or why nothing could be read.
fn run() -> Result<ExitCode, String> {
    let shipped = Shipped::decided().map_err(|why| format!("shipped.toml: {why:?}"))?;
    let given: Vec<String> = std::env::args().skip(1).collect();
    if given.iter().map(String::as_str).eq(["--identifiers"]) {
        for pinned in shipped.every() {
            println!(
                "{} {}",
                pinned.source().as_str(),
                pinned.application().identifier()
            );
        }
        return Ok(ExitCode::SUCCESS);
    }
    if given.is_empty() || given.iter().any(|one| one.starts_with('-')) {
        return Err("give --identifiers, or the folders the entries are in".to_owned());
    }
    let folders: Vec<PathBuf> = given.into_iter().map(PathBuf::from).collect();
    let installed = alo_installed::read(&folders);
    let made = as_their_entries_name_them(&shipped, &installed).map_err(|why| why.to_string())?;
    if made == THE_NAMES {
        println!("crates/alo-software/the-shipped-names.txt holds the packages' own names.");
        return Ok(ExitCode::SUCCESS);
    }
    println!(
        "crates/alo-software/the-shipped-names.txt is not what the packages say. It should hold, \
         between the lines:\n----\n{made}----"
    );
    Ok(ExitCode::from(1))
}
