//! The independent reference's signal, sampled at 16 kHz.

/// A marker at seconds 0, 1, 3 and 6 of each seven-second cycle.
#[must_use]
pub fn marker_second(second: u64) -> bool {
    matches!(second % 7, 0 | 1 | 3 | 6)
}

/// Half a second of white at each marker onset, visible at five frames/s.
#[must_use]
pub fn lit_at(time_ns: u64) -> bool {
    marker_second(time_ns / 1_000_000_000) && time_ns % 1_000_000_000 < 500_000_000
}

/// One 16 kHz PCM sample: a 1 kHz tone for the marker's first 200 ms.
///
/// The sixteen rounded samples are one sine period at amplitude 4915. An
/// integer table avoids phase and rounding drift between generated buffers.
#[must_use]
pub fn tone_sample(time_ns: u64) -> i16 {
    const WAVE: [i16; 16] = [
        0, 1881, 3475, 4541, 4915, 4541, 3475, 1881, 0, -1881, -3475, -4541, -4915, -4541, -3475,
        -1881,
    ];
    if !marker_second(time_ns / 1_000_000_000) || time_ns % 1_000_000_000 >= 200_000_000 {
        return 0;
    }
    let index = usize::try_from((time_ns / 62_500) % 16).unwrap_or_default();
    WAVE.get(index).copied().unwrap_or_default()
}
