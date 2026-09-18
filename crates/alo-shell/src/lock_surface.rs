//! Locked input and the existing sign-in entry, without a second authenticator.
use crate::sign_in_entry::SignInEntry;
use crate::{SignInKey, SignInShows};
use alo_accounts::Accounts;
use alo_greeting::NotReadable;
use alo_locking::{Seat, Unlocking};
use alo_strings::{Said, Strings};

/// A locked session and its optional unlock fields. Never formats credentials.
pub struct LockSurface<N> {
    /// The authoritative lock state and held private notifications.
    seat: Seat<N>,
    /// Shared bounded, zeroing sign-in credential entry.
    entry: SignInEntry,
    /// Whether Enter has explicitly opened the fields.
    asking: bool,
    /// Vocabulary for existing policy refusals.
    strings: Strings,
    /// Last refusal, with no credentials or account-store path.
    said: Option<Said>,
}

/// One key either leaves the lock up or returns the original session and held messages.
pub enum LockPressed<N> {
    /// Still locked, including after every refusal.
    Still(Box<LockSurface<N>>),
    /// Authenticated by the existing lock/greeting composition.
    Opened {
        /// The original session, now unlocked.
        seat: Seat<N>,
        /// Notifications held while locked, in arrival order.
        held: Vec<N>,
    },
}

impl<N> LockSurface<N> {
    /// Consume an already locked seat. An open seat is returned untouched.
    /// # Errors
    /// Returns the supplied seat if it was not locked.
    pub fn of(seat: Seat<N>, strings: Strings) -> Result<Self, Seat<N>> {
        match seat {
            Seat::Locked(locked) => Ok(Self {
                seat: Seat::Locked(locked),
                entry: SignInEntry::empty(),
                asking: false,
                strings,
                said: None,
            }),
            open => Err(open),
        }
    }

    /// Notifications remain private until the lock composition releases them.
    pub fn arrives(&mut self, notification: N) {
        let _ = self.seat.arrives(notification);
    }

    /// The policy's privacy-limited view; held notifications cannot enter it.
    pub fn snapshot(
        &self,
        now: std::time::SystemTime,
        appearance: &alo_appearance::Appearance,
        display: &alo_appearance::DisplayId,
        battery: Option<alo_locking::Battery>,
        indicator: &alo_egress::Indicator,
    ) -> Option<alo_locking::LockScreen> {
        self.seat
            .lock_screen(now, appearance, display, battery, indicator)
    }

    /// Delegate the agent key to lock policy before any compositor can be reached.
    pub fn press_the_agents_key(
        &self,
        summoning: &mut alo_overlay::Summoning,
        compositor: Option<&mut dyn alo_overlay::Compositor>,
    ) -> Result<alo_overlay::Pressed, alo_locking::NotWhileLocked> {
        self.seat.press_the_agents_key(summoning, compositor)
    }

    /// Forget unfinished input when the nested window loses focus.
    pub(crate) fn lost_focus(&mut self) {
        self.entry.forgotten();
        self.asking = false;
        self.said = None;
    }

    /// Whether the person explicitly asked to type their credentials.
    pub const fn is_asking(&self) -> bool {
        self.asking
    }

    /// The same password-free view used by the sign-in rasterizer.
    pub fn shows(&self) -> SignInShows<'_> {
        SignInShows::Fields {
            name: self.entry.name(),
            password_typed: self.entry.has_password(),
            field: self.entry.field(),
            said: self.said.as_ref(),
        }
    }

    /// One intercepted key. Enter opens the fields; nothing starts on a timer.
    /// Accounts are re-read only at submission, through the supplied reader.
    /// Every submission forgets both fields, including an unreadable store.
    pub fn pressed(
        mut self,
        key: SignInKey,
        read: impl FnOnce() -> Result<Accounts, NotReadable>,
    ) -> LockPressed<N> {
        if !self.asking {
            self.asking = key == SignInKey::Enter;
            return LockPressed::Still(Box::new(self));
        }
        if !self.entry.pressed(key) {
            return LockPressed::Still(Box::new(self));
        }
        let accounts = match read() {
            Ok(accounts) => accounts,
            Err(why) => {
                self.entry.forgotten();
                self.said = Some(why.said(&self.strings));
                return LockPressed::Still(Box::new(self));
            }
        };
        let answer = self
            .seat
            .unlocks(accounts, self.entry.name(), self.entry.password());
        self.entry.forgotten();
        match answer {
            Unlocking::Unlocked { seat, held } => LockPressed::Opened { seat, held },
            Unlocking::StillLocked {
                seat: Seat::Locked(locked),
                refused,
            } => {
                self.seat = Seat::Locked(locked);
                self.said = refused.said(&self.strings);
                LockPressed::Still(Box::new(self))
            }
            // Fail closed even if a future composition changes its outcomes.
            Unlocking::WasNotLocked { seat } | Unlocking::StillLocked { seat, .. } => {
                self.seat = seat.locked(&mut alo_overlay::Summoning::closed());
                LockPressed::Still(Box::new(self))
            }
        }
    }
}
