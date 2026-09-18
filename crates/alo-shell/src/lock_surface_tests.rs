//! Real authentication and refusal at the lock surface, with held messages intact.
#![allow(clippy::panic, clippy::indexing_slicing)]
use crate::lock_testing::*;
use crate::{LockPressed, SignInField, SignInKey, SignInShows};

#[test]
/// The existing authenticator unlocks only the original session and releases held messages.
fn the_existing_authenticator_unlocks_only_the_original_session_and_releases_held_messages() {
    for (name, password) in [
        ("ada", "wrong"),
        ("nobody", "wrong"),
        ("ben", "ben's correct password"),
    ] {
        let mut surface = typed(name, password);
        surface.arrives("private notification".into());
        let LockPressed::Still(surface) = surface.pressed(SignInKey::Enter, || Ok(accounts()))
        else {
            panic!("a refused identity opened the session")
        };
        assert!(matches!(
            surface.shows(),
            SignInShows::Fields {
                name: "",
                password_typed: false,
                field: SignInField::Name,
                said: Some(_)
            }
        ));
        let mut surface = *surface;
        for c in "ada".chars() {
            surface = key(surface, SignInKey::Letter(c));
        }
        surface = key(surface, SignInKey::Enter);
        for c in "correct horse battery staple".chars() {
            surface = key(surface, SignInKey::Letter(c));
        }
        let LockPressed::Opened { seat, held } =
            surface.pressed(SignInKey::Enter, || Ok(accounts()))
        else {
            panic!("correct credentials did not reopen")
        };
        assert_eq!(seat.session(), &session());
        assert!(!seat.is_locked());
        assert_eq!(held, ["private notification"]);
    }
}

#[test]
/// Unreadable accounts forget the fields and keep the screen locked.
fn unreadable_accounts_forget_the_fields_and_keep_the_screen_locked() {
    let LockPressed::Still(surface) =
        typed("ada", "private secret").pressed(SignInKey::Enter, || {
            Err(alo_greeting::NotReadable {
                at: "/private/accounts".into(),
                why: "unreadable".into(),
            })
        })
    else {
        panic!("unreadable accounts opened the seat")
    };
    let SignInShows::Fields {
        name,
        password_typed,
        said: Some(said),
        ..
    } = surface.shows()
    else {
        panic!("refusal not shown")
    };
    assert!(name.is_empty());
    assert!(!password_typed);
    assert!(!said.is_a_bug());
    assert!(!said.text().contains("/private"));
}

#[test]
/// The agent key never reaches a compositor while locked.
fn the_agent_key_never_reaches_a_compositor_while_locked() {
    /// Counts overlay requests without drawing a real surface.
    struct Screen(usize);
    impl alo_overlay::Compositor for Screen {
        /// Honour.
        fn honour(
            &mut self,
            _: alo_overlay::SurfaceRequest,
        ) -> Result<(), alo_overlay::SurfaceRefused> {
            self.0 += 1;
            Ok(())
        }
    }
    let surface = surface();
    let mut screen = Screen(0);
    let mut summoning = alo_overlay::Summoning::closed();
    for _ in 0..3 {
        assert!(
            surface
                .press_the_agents_key(&mut summoning, Some(&mut screen))
                .is_err()
        );
    }
    assert_eq!(screen.0, 0);
    assert!(!summoning.is_open());
    let surface = key(surface, SignInKey::Nothing);
    assert!(!surface.is_asking());
}

/// Moving away forgets unfinished credentials and closes the explicit prompt.
#[test]
fn losing_focus_forgets_unfinished_credentials() {
    let mut surface = typed("ada", "private password");
    surface.lost_focus();
    assert!(!surface.is_asking());
    assert!(matches!(
        surface.shows(),
        SignInShows::Fields {
            name: "",
            password_typed: false,
            said: None,
            ..
        }
    ));
}
