//! The compositor's half of `alo-recounting`: the account on the screen.
//!
//! `alo_recounting::Compositor` is the port whatever owns the screen implements
//! to be handed an account. This is that implementation, and it is
//! deliberately only a place to keep what it was handed: one `Account`, read
//! off the disk by `alo_recounting::Recounting` a moment before, and nothing
//! else. There is no way to put an account here except to be handed one through
//! that port, and an `Account` cannot be made outside `alo-recounting` — so
//! there is no way to draw an account that was not read off the record.

use alo_recounting::{Account, Compositor, SurfaceRefused};

/// The account on the screen, as the compositor was last handed it.
#[derive(Debug)]
pub(crate) struct RecordShown {
    /// Whether this compositor has an output to put an account on.
    output: bool,
    /// The account it was handed, until the window is closed or reads again.
    account: Option<Account>,
}

impl RecordShown {
    /// A compositor drawing on an output.
    pub(crate) fn on_an_output() -> Self {
        Self {
            output: true,
            account: None,
        }
    }

    /// A compositor with nothing to put an account on: a headless session.
    pub(crate) fn with_no_output() -> Self {
        Self {
            output: false,
            account: None,
        }
    }

    /// The account on the screen, if one is.
    pub(crate) fn account(&self) -> Option<&Account> {
        self.account.as_ref()
    }

    /// Take the account down: the window closed, or is about to show a refusal.
    pub(crate) fn taken_down(&mut self) {
        self.account = None;
    }
}

impl Compositor for RecordShown {
    fn show(&mut self, account: Account) -> Result<(), SurfaceRefused> {
        if !self.output {
            self.account = None;
            return Err(SurfaceRefused::NothingToShowOn);
        }
        self.account = Some(account);
        Ok(())
    }
}
