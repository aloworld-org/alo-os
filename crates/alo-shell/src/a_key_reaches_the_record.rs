//! **A key reaches the record window while it is open.**
//!
//! `crate::record_seat` reads a key as the window understands it, and
//! `RecordWindow::pressed` does what it says; nothing on the real display road
//! handed one to the other until 2026-10-27, because nothing on that road ever
//! opened the window. `crate::a_chord_opens_the_record` opens it now, so this
//! carries the keys: Up, Down, Home and End move through the account, F5 reads
//! the record again, and Escape closes it.
//!
//! **No key reaches an application while the window is open**, which is
//! `crate::record_seat`'s own rule: the record seat intercepts every key.

use crate::{InputError, InputUpdate, RecordKey, RecordOpened, Server};

/// Route one seat update while the record window is open.
///
/// Keys go to the window and reach no client. Everything else — a pointer, a
/// device going away — takes the road it always took.
///
/// # Errors
/// Whatever the seat refuses about the key, and whatever the ordinary road
/// refuses about everything else. A code outside evdev's range is **not** an
/// error here, for `crate::a_key_reaches_settings`' reason.
pub(crate) fn a_key_reaches_the_record(
    server: &mut Server,
    update: InputUpdate<'_>,
    extent: (i32, i32),
) -> Result<(), InputError> {
    let InputUpdate::Event(event) = update else {
        return server.libinput_update(update, extent);
    };
    let Some(crate::DirectSeatEvent::Key(key, _)) = crate::libinput_routing::translate(event)?
    else {
        return server.libinput_update(InputUpdate::Event(event), extent);
    };
    the_key_the_record_took(server, key)
}

/// One key, from the evdev code inward. Split so a test can press one.
///
/// # Errors
/// Whatever the seat refuses about the key.
pub(crate) fn the_key_the_record_took(
    server: &mut Server,
    key: crate::DirectKeyEvent,
) -> Result<(), InputError> {
    match server.record_key(key.code, key.state, key.time) {
        Ok(Some(press)) => {
            the_press_is_carried_out(server, press);
            Ok(())
        }
        Ok(None) | Err(InputError::InvalidKey) => Ok(()),
        Err(error) => Err(error),
    }
}

/// Hand one press to the window.
///
/// **A window opened on a machine that could not say where its record is has
/// no recounting to read again**, so the only key that does anything there is
/// Escape: it closes, and every other key does nothing rather than invent a
/// record to read.
fn the_press_is_carried_out(server: &mut Server, press: RecordKey) {
    match server.recounting.clone() {
        Some(Ok(recounting)) => {
            if let Some(RecordOpened::NowhereToShow(why)) =
                server.record.pressed(press, &recounting)
            {
                eprintln!("alo-shell: the record could not be shown again: {why:?}");
            }
        }
        Some(Err(_)) | None => {
            if press == RecordKey::Close {
                server.record.closed();
            }
        }
    }
}

#[cfg(test)]
#[expect(
    clippy::panic,
    reason = "in a test, a panic on a window showing the wrong thing is the failure being reported"
)]
#[path = "a_key_reaches_the_record_tests.rs"]
mod tests;
