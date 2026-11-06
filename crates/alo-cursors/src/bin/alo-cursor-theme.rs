//! **Writes alo's cursor theme** into a directory, for the image build to
//! install: `alo-cursor-theme /usr/share/icons/alo`.
//!
//! Run once, at build time, by `image/Containerfile`. It draws every cursor from
//! the vector source the compositor draws from, so the theme an application reads
//! and the pointer alo draws cannot drift apart. Nothing runs it on a person's
//! machine.

use std::process::ExitCode;

fn main() -> ExitCode {
    let mut arguments = std::env::args_os().skip(1);
    let (Some(directory), None) = (arguments.next(), arguments.next()) else {
        eprintln!("usage: alo-cursor-theme <directory to write the theme into>");
        return ExitCode::from(2);
    };
    let directory = std::path::PathBuf::from(directory);
    let made = alo_cursors::theme::the_theme(alo_cursors::Silhouette::default())
        .and_then(|theme| alo_cursors::theme::write_into(&theme, &directory).map(|()| theme));
    match made {
        Ok(theme) => {
            println!(
                "wrote the {} cursor theme into {}: {} cursors, {} older names",
                alo_cursors::theme::THE_NAME,
                directory.display(),
                theme.cursors.len(),
                theme.aliases.len()
            );
            ExitCode::SUCCESS
        }
        Err(why) => {
            eprintln!("{why}");
            ExitCode::FAILURE
        }
    }
}
