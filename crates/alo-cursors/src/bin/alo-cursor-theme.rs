//! **Writes alo's cursor themes** into a directory, for the image build to
//! install: `alo-cursor-theme /usr/share/icons` writes `alo/`, with the precision
//! kite, and `alo-classic/`, with the familiar arrow a person may choose instead.
//!
//! Run once, at build time, by `image/Containerfile`. It draws every cursor from
//! the vector source the compositor draws from, so the theme an application reads
//! and the pointer alo draws cannot drift apart. Nothing runs it on a person's
//! machine.

use std::process::ExitCode;

fn main() -> ExitCode {
    let mut arguments = std::env::args_os().skip(1);
    let (Some(directory), None) = (arguments.next(), arguments.next()) else {
        eprintln!("usage: alo-cursor-theme <directory to write the themes into>");
        return ExitCode::from(2);
    };
    let directory = std::path::PathBuf::from(directory);
    for silhouette in alo_cursors::Silhouette::ALL {
        let name = alo_cursors::theme::the_name_of(silhouette);
        let into = directory.join(name);
        let made = alo_cursors::theme::the_theme(silhouette)
            .and_then(|theme| alo_cursors::theme::write_into(&theme, &into).map(|()| theme));
        match made {
            Ok(theme) => println!(
                "wrote the {name} cursor theme into {}: {} cursors, {} older names",
                into.display(),
                theme.cursors.len(),
                theme.aliases.len()
            ),
            Err(why) => {
                eprintln!("{name}: {why}");
                return ExitCode::FAILURE;
            }
        }
    }
    ExitCode::SUCCESS
}
