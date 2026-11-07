//! What alo's cursor theme holds, and that what it writes is what a toolkit
//! finds.

use super::*;

/// The theme as it ships.
fn shipped() -> Theme {
    the_theme(Silhouette::default()).unwrap_or_else(|why| unreachable!("{why}"))
}

/// **Every name the protocol defines has its own file**, holding every size,
/// and every older name points at one of them.
#[test]
fn every_name_has_a_file_and_every_old_name_points_at_one() {
    let theme = shipped();
    for icon in EVERY_SHAPE {
        let Some(file) = theme.cursors.get(icon.name()) else {
            unreachable!("{} has no cursor file", icon.name());
        };
        let Some(images) = xcursor::parser::parse_xcursor(file) else {
            unreachable!("{}'s file does not read", icon.name());
        };
        assert_eq!(
            images.iter().map(|image| image.size).collect::<Vec<_>>(),
            SIZES,
            "{}",
            icon.name()
        );
    }
    for (alias, target) in &theme.aliases {
        assert!(
            theme.cursors.contains_key(target),
            "{alias} points at {target}, which is not in the theme"
        );
        assert!(
            !theme.cursors.contains_key(alias),
            "{alias} is a file and an alias"
        );
    }
}

/// **The old names toolkits actually ask for reach the right cursor**, and an
/// old name that is also a CSS name is the CSS cursor rather than an alias.
#[test]
fn the_old_names_reach_the_right_cursor() {
    let theme = shipped();
    for (old, new) in [
        ("left_ptr", "default"),
        ("xterm", "text"),
        ("hand2", "pointer"),
        ("watch", "wait"),
        ("left_ptr_watch", "progress"),
        ("h_double_arrow", "ew-resize"),
        ("v_double_arrow", "ns-resize"),
        ("crossed_circle", "not-allowed"),
        ("closedhand", "grabbing"),
    ] {
        assert_eq!(
            theme.aliases.get(old).map(String::as_str),
            Some(new),
            "{old}"
        );
    }
    for both in ["copy", "move"] {
        assert!(theme.cursors.contains_key(both) && !theme.aliases.contains_key(both));
    }
}

/// **The index names the theme**, which is what `XCURSOR_THEME=alo` finds.
#[test]
fn the_index_names_the_theme() {
    let index = shipped().index;
    assert!(index.starts_with("[Icon Theme]\n"));
    assert!(index.contains(&format!("\nName={THE_NAME}\n")));
}

/// **The familiar arrow's theme differs from the shipped one only where the
/// arrow is**: its own name, a different arrow, and the same text cursor, hands
/// and resize arrows.
#[test]
fn the_classic_theme_changes_only_the_arrow() {
    let kite = shipped();
    let classic = the_theme(Silhouette::Classic).unwrap_or_else(|why| unreachable!("{why}"));
    assert!(
        classic
            .index
            .contains(&format!("\nName={THE_CLASSIC_NAME}\n"))
    );
    assert_eq!(the_name_of(Silhouette::Kite), THE_NAME);
    assert_eq!(the_name_of(Silhouette::Classic), THE_CLASSIC_NAME);
    assert_ne!(kite.cursors.get("default"), classic.cursors.get("default"));
    for same in ["text", "pointer", "grab", "ew-resize", "not-allowed"] {
        assert_eq!(kite.cursors.get(same), classic.cursors.get(same), "{same}");
    }
    assert_eq!(kite.aliases, classic.aliases);
}

/// **What is written is what a toolkit finds**: the index, every file, and an
/// old name that opens to the same bytes as the name it points at.
#[test]
fn what_is_written_is_what_a_toolkit_finds() -> Result<(), NotMade> {
    let theme = shipped();
    let directory = std::env::temp_dir().join(format!(
        "alo-cursors-theme-{}-{}",
        std::process::id(),
        line!()
    ));
    write_into(&theme, &directory)?;
    // Twice, as a rebuilt image layer would: an alias already there is replaced.
    write_into(&theme, &directory)?;
    let read = |path: &str| std::fs::read(directory.join(path)).map_err(NotMade::Io);
    assert_eq!(read("index.theme")?, theme.index.as_bytes());
    for (name, bytes) in &theme.cursors {
        assert_eq!(&read(&format!("cursors/{name}"))?, bytes, "{name}");
    }
    assert_eq!(read("cursors/left_ptr")?, read("cursors/default")?);
    std::fs::remove_dir_all(&directory).map_err(NotMade::Io)?;
    Ok(())
}

/// **A place that cannot be written is refused**, not half-written silently.
#[test]
fn a_place_that_cannot_be_written_is_refused() {
    let theme = shipped();
    let file = std::env::temp_dir().join(format!("alo-cursors-not-a-dir-{}", std::process::id()));
    assert!(std::fs::write(&file, b"a file, not a directory").is_ok());
    assert!(matches!(write_into(&theme, &file), Err(NotMade::Io(_))));
    assert!(std::fs::remove_file(&file).is_ok());
}
