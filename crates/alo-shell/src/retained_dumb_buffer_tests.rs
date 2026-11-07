//! The production mapping slot admits one allocation and retains refusal ownership.

use super::cached;
use std::{cell::Cell, io, rc::Rc};

/// Observe retirement without borrowing any real hardware.
struct Mapping {
    /// CPU bytes written by the same callback boundary as production.
    pixels: Vec<u8>,
    /// Exactly one destructor per successful mapping.
    retired: Rc<Cell<usize>>,
}

impl Drop for Mapping {
    fn drop(&mut self) {
        self.retired.set(self.retired.get() + 1);
    }
}

#[test]
fn repeated_uploads_reuse_one_mapping_and_retire_it_once() -> io::Result<()> {
    let maps = Cell::new(0);
    let retired = Rc::new(Cell::new(0));
    let mut slot = None;
    for colour in [4, 9, 17] {
        cached(
            &mut slot,
            || {
                maps.set(maps.get() + 1);
                Ok(Mapping {
                    pixels: vec![0; 16],
                    retired: retired.clone(),
                })
            },
            |mapping| {
                mapping.pixels.fill(colour);
                Ok(())
            },
        )?;
        assert!(
            slot.as_ref()
                .is_some_and(|mapping| mapping.pixels.iter().all(|pixel| *pixel == colour))
        );
        assert_eq!(retired.get(), 0);
    }
    assert_eq!(maps.get(), 1);
    drop(slot);
    assert_eq!(retired.get(), 1);
    Ok(())
}

#[test]
fn a_mapping_refusal_is_not_cached_and_never_runs_the_upload() -> io::Result<()> {
    let mut slot: Option<Vec<u8>> = None;
    let called = Cell::new(false);
    let result = cached(
        &mut slot,
        || Err(io::Error::from_raw_os_error(12)),
        |_| {
            called.set(true);
            Ok(())
        },
    );
    assert_eq!(
        result.err().and_then(|error| error.raw_os_error()),
        Some(12)
    );
    assert!(slot.is_none());
    assert!(!called.get());
    cached(
        &mut slot,
        || Ok(vec![0; 4]),
        |pixels| {
            pixels.fill(7);
            Ok(())
        },
    )?;
    assert_eq!(slot, Some(vec![7; 4]));
    Ok(())
}

#[test]
fn an_upload_refusal_retains_the_mapping_until_its_owner_retires_it() {
    let retired = Rc::new(Cell::new(0));
    let mut slot = None;
    let result = cached(
        &mut slot,
        || {
            Ok(Mapping {
                pixels: vec![0; 4],
                retired: retired.clone(),
            })
        },
        |_| Err::<(), _>(io::Error::from_raw_os_error(5)),
    );
    assert_eq!(result.err().and_then(|error| error.raw_os_error()), Some(5));
    assert!(slot.is_some());
    assert_eq!(retired.get(), 0);
    drop(slot);
    assert_eq!(retired.get(), 1);
}
