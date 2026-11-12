//! *Which applications could open this kind of link* — the list a person
//! chooses their default browser or mail application from.
//!
//! `docs/design/default-apps-and-open-with.md`, approved by the owner on
//! 2026-11-12: **only installed handlers are listed** — an application whose
//! desktop entry declares `x-scheme-handler/http` or `…/https` for web links,
//! `…/mailto` for email addresses — and **a terminal is never listed**, because
//! no link is handed to one ([`crate::link_opener`]). Settings → Default apps
//! and the *Open with* sheet both draw this list, so the two cannot offer
//! different applications for the same kind.
//!
//! # In the order they declared it
//!
//! The first to declare a kind is listed first, which is
//! [`crate::Declared`]'s own order and the order
//! [`crate::WhatOpensWhat::what_opens_links`] falls back through. A list sorted
//! some other way would put the application that opens links *now* somewhere
//! other than where the rule that chose it would look.
//!
//! # It chooses nothing
//!
//! Which one is ringed is [`crate::WhatOpensWhat::what_opens_links`]'s answer,
//! asked separately: this is the choices, and that is the answer.

use alo_capability::is_a_persons_own;

use crate::application::Application;
use crate::link::LinkKind;
use crate::what_opens::WhatOpensWhat;

impl WhatOpensWhat<'_> {
    /// Every installed application that declares this kind of link and is not
    /// a terminal, first declared first, each once.
    #[must_use]
    pub fn what_could_open_links(&self, kind: LinkKind) -> Vec<&Application> {
        let mut could: Vec<&Application> = Vec::new();
        for declaring in self.declared().declaring_links(kind) {
            if is_a_persons_own(declaring) {
                continue;
            }
            let Some(application) = self.installed().knows(declaring) else {
                continue;
            };
            if !could
                .iter()
                .any(|already| already.identifier() == application.identifier())
            {
                could.push(application);
            }
        }
        could
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use crate::Application;
    use crate::chosen::Chosen;
    use crate::declared::Declared;
    use crate::installed::Installed;
    use crate::link::LinkKind;
    use crate::what_opens::WhatOpensWhat;

    fn called(identifier: &str) -> Application {
        Application::identified(identifier).unwrap()
    }

    fn identifiers(of: &[&Application]) -> Vec<String> {
        of.iter().map(|it| it.identifier().to_owned()).collect()
    }

    /// **Installed declarers only, first declared first, once each — and never
    /// a terminal**, even one that declared web links first.
    #[test]
    fn the_choices_are_installed_declarers_in_order_with_no_terminal() {
        let installed = Installed::holding([
            called("org.mozilla.firefox"),
            called("io.gitlab.librewolf-community"),
            called("org.mozilla.Thunderbird"),
            called("app.devsuite.Ptyxis"),
        ]);
        let mut declared = Declared::nothing();
        declared.declares_media_types(&called("app.devsuite.Ptyxis"), ["x-scheme-handler/http"]);
        declared.declares_media_types(
            &called("io.gitlab.librewolf-community"),
            ["x-scheme-handler/https"],
        );
        declared.declares_media_types(
            &called("org.mozilla.firefox"),
            ["x-scheme-handler/http", "x-scheme-handler/https"],
        );
        // Declared, and not installed: never a choice.
        declared.declares_media_types(&called("com.example.Gone"), ["x-scheme-handler/http"]);
        declared.declares_media_types(
            &called("org.mozilla.Thunderbird"),
            ["x-scheme-handler/mailto"],
        );
        let chosen = Chosen::untouched();
        let asking = WhatOpensWhat::on(&installed, &declared, &chosen);
        assert_eq!(
            identifiers(&asking.what_could_open_links(LinkKind::Web)),
            ["io.gitlab.librewolf-community", "org.mozilla.firefox"]
        );
        assert_eq!(
            identifiers(&asking.what_could_open_links(LinkKind::Mail)),
            ["org.mozilla.Thunderbird"]
        );
    }

    /// **A machine with no handler offers no choice**, which Settings says in
    /// a line rather than drawing an empty card.
    #[test]
    fn nothing_declared_is_no_choice() {
        let installed = Installed::holding([called("org.mozilla.firefox")]);
        let declared = Declared::nothing();
        let chosen = Chosen::untouched();
        let asking = WhatOpensWhat::on(&installed, &declared, &chosen);
        assert!(asking.what_could_open_links(LinkKind::Web).is_empty());
        assert!(asking.what_could_open_links(LinkKind::Mail).is_empty());
    }
}
