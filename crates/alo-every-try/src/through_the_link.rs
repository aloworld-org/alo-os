//! **A path through `/home`, where the machine's link leads.**
//!
//! On the image the set is asked of, `/home` is a link to `/var/home`
//! (`image/usr/lib/tmpfiles.d/alo.conf`; walked on 2026-10-26, as
//! `alo_capability::Grant::really` records). Every verb that takes a path has
//! it resolved through every link before anything is done with it
//! (`alo_files::Touching`), and at full access every path is permitted where
//! it really is (`alo_capability::full_access`). So at full access a booted
//! machine takes `/home/alo/hello.txt` as `/var/home/alo/hello.txt`, and so
//! does a try here.
//!
//! **Not at ask each time**: there the grants are asked about the path as it
//! was written first, and a grant over the home is over `/var/home/…`, so a
//! path through the link is held as it always was.

use alo_capability::{Given, Takes, Verbs};
use alo_protocol::{Argument, FromAnAgent};

/// Where the link is.
const THE_LINK: &str = "/home";

/// Where it leads.
const WHERE_IT_LEADS: &str = "/var/home";

/// `asked`, with every path it gives through `/home` given where the
/// link leads instead; everything else, and every other request, as it was.
#[must_use]
pub fn through_the_link(asked: &FromAnAgent, verbs: &Verbs) -> FromAnAgent {
    let (FromAnAgent::Read { verb, given } | FromAnAgent::Propose { verb, given }) = asked else {
        return asked.clone();
    };
    let Some(declared) = verbs.of(verb.trim()) else {
        return asked.clone();
    };
    let a_path = |named: &str| {
        declared
            .args()
            .iter()
            .any(|arg| arg.name() == named && *arg.takes() == Takes::Path)
    };
    let given: Vec<Argument> = given
        .iter()
        .map(|argument| match argument.given() {
            Given::Text(path) if a_path(argument.named()) => match led_to(path) {
                Some(led) => Argument::of(argument.named(), Given::Text(led)),
                None => argument.clone(),
            },
            _ => argument.clone(),
        })
        .collect();
    match asked {
        FromAnAgent::Read { verb, .. } => FromAnAgent::Read {
            verb: verb.clone(),
            given,
        },
        _ => FromAnAgent::Propose {
            verb: verb.clone(),
            given,
        },
    }
}

/// Where `path` leads, when it goes through [`THE_LINK`].
fn led_to(path: &str) -> Option<String> {
    let rest = path.strip_prefix(THE_LINK)?;
    (rest.is_empty() || rest.starts_with('/')).then(|| format!("{WHERE_IT_LEADS}{rest}"))
}

#[cfg(test)]
mod tests {
    use super::led_to;

    #[test]
    fn a_path_through_the_link_leads_under_var_home() {
        assert_eq!(
            led_to("/home/alo/hello.txt").as_deref(),
            Some("/var/home/alo/hello.txt")
        );
        assert_eq!(led_to("/home").as_deref(), Some("/var/home"));
    }

    /// **The refusal case**: a path that only begins with the same letters,
    /// or is already where the link leads, is not the link's.
    #[test]
    fn a_path_not_through_the_link_is_left_alone() {
        assert_eq!(led_to("/homes/alo/hello.txt"), None);
        assert_eq!(led_to("/var/home/alo/hello.txt"), None);
        assert_eq!(led_to("home/alo"), None);
    }
}
