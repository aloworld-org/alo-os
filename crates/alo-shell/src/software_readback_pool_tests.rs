//! A live or concurrent reader must never see a reused mutable allocation.

use super::*;

#[test]
fn storage_is_reused_only_after_every_strong_reader_releases_it()
-> Result<(), Box<dyn std::error::Error>> {
    let mut pool = Buffers::default();
    let frame = Arc::new(vec![7; 16]);
    let identity = frame.as_ptr();
    pool.put(frame.clone());
    assert!(pool.take(16).is_none());
    assert_eq!(frame.as_slice(), &[7; 16]);
    let weak = Arc::downgrade(&frame);
    drop(frame);
    let mut reused = pool.take(16).ok_or("released storage was not retained")?;
    assert_eq!(reused.as_ptr(), identity);
    reused.fill(9);
    assert!(
        weak.upgrade().is_none(),
        "old weak readers cannot observe new pixels"
    );
    assert_eq!(pool.bytes, 0);
    Ok(())
}

#[test]
fn capacity_budget_and_oversized_bypass_preserve_existing_storage() {
    let mut pool = Buffers {
        limit: 48,
        ..Buffers::default()
    };
    let first = Arc::new(vec![1; 16]);
    let oldest = Arc::downgrade(&first);
    pool.put(first);
    pool.put(Arc::new(vec![2; 16]));
    pool.put(Arc::new(vec![3; 32]));
    assert!(oldest.upgrade().is_none());
    assert_eq!(pool.bytes, 48);
    pool.put(Arc::new(vec![4; 64]));
    assert_eq!(pool.bytes, 48);
    assert!(pool.take(64).is_none());
    assert_eq!(pool.take(16).as_deref(), Some([2; 16].as_slice()));
    assert_eq!(pool.take(32).as_deref(), Some([3; 32].as_slice()));
    assert_eq!(pool.bytes, 0);
}

#[test]
fn returned_frames_are_bounded_separately_from_their_bytes() {
    let mut pool = Buffers::default();
    let first = Arc::new(vec![0; 4]);
    let oldest = Arc::downgrade(&first);
    pool.put(first);
    for _ in 0..MAX_FRAMES {
        pool.put(Arc::new(vec![1; 4]));
    }
    assert_eq!(pool.frames.len(), MAX_FRAMES);
    assert_eq!(pool.bytes, MAX_FRAMES * 4);
    assert!(oldest.upgrade().is_none());
    assert!(pool.take(8).is_none());
}
