//! **The walk**, in ordinary Rust over `crate::libatspi`'s safe calls.
//!
//! Find the application whose toolkit is `alo`, let GLib's main loop take in
//! what arrived for two seconds (libatspi's `GetItems` answer among it), turn
//! the cache on if asked, and read every thing in the tree, depth first. One
//! line per thing:
//!
//! ```text
//! depth  role  child-count  index-in-parent  parent|stray  name
//! ```
//!
//! separated by tabs. `stray` is a child whose parent is not the thing that
//! listed it, or whose index is not where it was listed. A child libatspi
//! gives as nothing is `missing`, and anything libatspi refused is `refused`
//! with what it said. The test reading this decides what is right; this only
//! reports what libatspi read.

use std::time::{Duration, Instant};

use crate::libatspi::{Libatspi, Thing};

/// How long GLib's main loop is given before the walk, so that libatspi's
/// `GetItems` — asked when it first met the application — is answered and
/// taken in. libatspi gives that call two seconds itself.
const SETTLING: Duration = Duration::from_secs(2);

/// How the tree is read.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum How {
    /// libatspi's cache off: every question is asked over the bus.
    OneQuestionAtATime,
    /// libatspi's cache on, so what `GetItems` said is what is read.
    Cached,
}

/// Walk alo's tree, returning the lines to print.
///
/// # Errors
/// What stopped the walk before it reached the tree: libatspi not starting,
/// no desktop, or no application drawn by `alo` on it.
pub(crate) fn walked(how: How) -> Result<Vec<String>, String> {
    let atspi = Libatspi::started()?;
    let desktop = atspi.desktop()?;
    let mut ours = None;
    for index in 0..desktop.child_count()? {
        let Ok(application) = desktop.child_at(index) else {
            continue;
        };
        if application.toolkit_name().is_ok_and(|name| name == "alo") {
            ours = Some(application);
        }
    }
    let ours = ours.ok_or("no application with the toolkit alo on the desktop")?;
    let started = Instant::now();
    while started.elapsed() < SETTLING {
        if !atspi.take_in_what_arrived() {
            std::thread::sleep(Duration::from_millis(10));
        }
    }
    if how == How::Cached {
        ours.cache_everything_it_can();
    }
    let mut lines = Vec::new();
    read(&ours, 0, None, &mut lines);
    Ok(lines)
}

/// One thing and everything under it, onto `lines`.
fn read(
    thing: &Thing<'_>,
    depth: usize,
    listed: Option<(&Thing<'_>, i32)>,
    lines: &mut Vec<String>,
) {
    let said = |what: Result<i32, String>| {
        what.map_or_else(|why| format!("refused {why}"), |n| n.to_string())
    };
    let count = thing.child_count();
    let index = thing.index_in_parent();
    let where_it_hangs = match listed {
        None => "parent".to_owned(),
        Some((by, at)) => {
            let under = thing.parent().ok().flatten();
            let here = under.as_ref().is_some_and(|parent| parent.is(by))
                && index.as_ref().ok() == Some(&at);
            if here { "parent" } else { "stray" }.to_owned()
        }
    };
    let name = thing.name().unwrap_or_else(|why| format!("refused {why}"));
    lines.push(format!(
        "{depth}\t{}\t{}\t{}\t{where_it_hangs}\t{name}",
        said(thing.role()),
        said(count.clone()),
        said(index)
    ));
    let Ok(count) = count else { return };
    for at in 0..count {
        match thing.child_at(at) {
            Ok(child) => read(&child, depth + 1, Some((thing, at)), lines),
            Err(_) => lines.push(format!("missing\t{}\t{at}", depth + 1)),
        }
    }
}
