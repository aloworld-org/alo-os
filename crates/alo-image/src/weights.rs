//! Where a person's own weights live, and that the image ships none of its own.
//!
//! This file used to be 822 lines holding the image **to carrying** a model:
//! the weights pinned by content, checked before anything read them, pruned to
//! the manifest that named them, matched against the catalogue's recommendation
//! for the certified laptop, and held to a licence we may redistribute under.
//!
//! [ADR 0095](../../../docs/decisions/0095-the-release-carries-no-model-and-a-person-brings-their-own.md)
//! removed the thing all of that was about. **The release carries no weights.**
//! A person brings weights they already have, or uses a provider, or works
//! without one, and alo OS chooses none of those for them.
//!
//! # Why those rules were deleted rather than left standing
//!
//! Every one of them had become a guard that cannot fire: there is nothing to
//! pin, nothing to verify, nothing to prune and no licence to redistribute.
//! `docs/misreadings/a-guard-that-cannot-fire-is-a-comment.md` is this
//! repository's own lesson about what that does — a check that cannot fail
//! reads as diligence and gets quoted as evidence, and is neither.
//!
//! # The two questions that replaced them, and why each earns its place
//!
//! **Does the recipe land weights?** If it ever does again, that is now the
//! fault. 4.87 GiB can return in one `COPY` line, and nobody reviewing a recipe
//! notices a layer getting bigger.
//!
//! **Is the store somewhere a person can write?** This is the one that nearly
//! shipped wrong. The store was `/usr/share/alo/models`, which is right for
//! weights that arrive with the machine and **impossible** for weights a person
//! brings: `/usr` is the read-only half of a bootc machine. Taking the weights
//! out without moving the store would have left every sentence about bringing
//! your own reading correctly while the thing itself could not happen.

/// Where the model runtime serves from, and where a person's own weights go.
///
/// Inside the model service's own state directory, which systemd makes before
/// the process starts, owns, and keeps at `0700`
/// (`image/usr/lib/systemd/system/alo-modeld.service`). **Writable**, which is
/// the whole point: see this module's header.
pub const THE_STORE_IS_AT: &str = "/var/lib/alo-model/models";

/// Where weights landed while the image carried them.
///
/// Kept so that a recipe putting them back is caught **by name**, and whoever
/// reads the failure is told where to look rather than handed a rule and left
/// to find the line.
pub const WHERE_WEIGHTS_USED_TO_LAND: &str = "/usr/share/alo/models";

/// The halves of a bootc machine that can be written after the image is built.
///
/// `/usr` is the image. `/var` is the machine's own and is kept across an
/// update; `/etc` is the person's. Anything added after installing lands in one
/// of the two that are not the image, and weights are now something the person
/// adds.
const WHAT_A_MACHINE_MAY_WRITE: [&str; 2] = ["/var/", "/etc/"];

/// Whether this path is somewhere the machine may write after it is installed.
///
/// A prefix, and nothing cleverer. A relative path, or one that climbs out with
/// `..`, is **not** writable by this answer — not because it could not be, but
/// because a path like that has not said where it is, and this question may
/// only be answered yes by something that has.
#[must_use]
pub fn a_machine_may_write(path: &str) -> bool {
    !path.contains("..")
        && WHAT_A_MACHINE_MAY_WRITE
            .iter()
            .any(|half| path.starts_with(half))
}

/// Everywhere the recipe copies something out of a build stage into a model
/// store, which should be nowhere.
///
/// Matched on the landing place rather than on the stage's name, because a
/// stage can be called anything and the question is what arrives on the
/// machine. Any landing place with `models` in it counts: the store has moved
/// once already, and a rule naming only the old place would miss weights put
/// back in the new one.
#[must_use]
pub fn where_the_recipe_lands_weights(recipe: &str) -> Vec<String> {
    recipe
        .lines()
        .map(str::trim)
        .filter(|line| line.starts_with("COPY") && line.contains("--from="))
        .filter_map(|line| line.split_whitespace().next_back())
        .filter(|landing| landing.contains("models"))
        .map(str::to_owned)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **The recipe this repository ships lands no weights.**
    #[test]
    fn the_shipped_recipe_lands_no_weights() {
        let recipe =
            std::fs::read_to_string(std::path::Path::new(crate::THE_IMAGE).join("Containerfile"))
                .unwrap_or_default();
        assert!(!recipe.is_empty(), "the recipe reads");
        assert_eq!(
            where_the_recipe_lands_weights(&recipe),
            Vec::<String>::new()
        );
    }

    /// **A recipe that puts them back is caught**, in the old place or a new
    /// one.
    #[test]
    fn a_recipe_that_puts_weights_back_is_caught() {
        for landing in [
            "/usr/share/alo/models/",
            "/var/lib/alo-model/models",
            "/opt/somebody-elses/models/",
        ] {
            let recipe =
                format!("FROM scratch AS weights\nCOPY --from=weights /models/ {landing}\n");
            assert_eq!(
                where_the_recipe_lands_weights(&recipe),
                vec![landing.to_owned()],
                "{landing}"
            );
        }
    }

    /// A copy that is not out of a stage, or lands somewhere else, is not
    /// weights.
    #[test]
    fn a_copy_that_is_not_weights_is_not_caught() {
        let recipe = "COPY image/usr/lib/os-release /usr/lib/os-release\n\
                      COPY --from=built /alo-agentd /usr/libexec/alo-agentd\n";
        assert_eq!(where_the_recipe_lands_weights(recipe), Vec::<String>::new());
    }

    /// **The store the service is pointed at is one the machine may write**,
    /// and the place weights used to land is not.
    #[test]
    fn the_store_is_writable_and_the_old_place_is_not() {
        assert!(a_machine_may_write(THE_STORE_IS_AT));
        assert!(!a_machine_may_write(WHERE_WEIGHTS_USED_TO_LAND));
        assert!(a_machine_may_write("/etc/alo/models"));
    }

    /// A path that has not said where it is cannot answer yes.
    #[test]
    fn a_path_that_has_not_said_where_it_is_is_not_writable() {
        assert!(!a_machine_may_write("var/lib/alo-model/models"));
        assert!(!a_machine_may_write("/var/lib/../usr/share/alo/models"));
        assert!(!a_machine_may_write(""));
    }
}
