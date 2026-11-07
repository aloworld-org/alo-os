//! **Writes alo's cursor themes** into a directory, for the image build to
//! install: `alo-cursor-theme /usr/share/icons` writes one theme for each arrow
//! a person may choose in each colour they may choose — `alo/` (the precision
//! kite, navy, as it ships), `alo-classic/`, and their white and black twins
//! (`alo_cursors::theme::the_name_for`).
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
    for (silhouette, colour) in alo_cursors::theme::every_theme() {
        let name = alo_cursors::theme::the_name_for(silhouette, colour);
        let into = directory.join(name);
        let made = alo_cursors::theme::the_theme_in(silhouette, colour)
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
