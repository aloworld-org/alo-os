//! The reference uses irregular onsets, long enough to see at five fps.

#[path = "../../../apps/alo-studio/verification/reference_signal.rs"]
mod signal;

#[test]
fn markers_repeat_an_irregular_seven_second_pattern() {
    for second in 0..21 {
        assert_eq!(
            signal::marker_second(second),
            matches!(second % 7, 0 | 1 | 3 | 6)
        );
    }
}

#[test]
fn visual_onsets_match_the_tone_and_remain_visible_for_half_a_second() {
    for second in 0..14 {
        let at = second * 1_000_000_000;
        assert_eq!(signal::lit_at(at), signal::marker_second(second));
        assert_eq!(
            signal::lit_at(at + 499_999_999),
            signal::marker_second(second)
        );
        assert!(!signal::lit_at(at + 500_000_000));
    }
}

#[test]
fn a_one_kilohertz_tone_is_present_for_only_the_first_two_hundred_milliseconds() {
    assert_eq!(signal::tone_sample(0), 0);
    assert_eq!(signal::tone_sample(62_500), 1881);
    assert_eq!(signal::tone_sample(250_000), 4915);
    assert_eq!(signal::tone_sample(750_000), -4915);
    assert_eq!(signal::tone_sample(1_062_500), 1881);
    assert_ne!(signal::tone_sample(199_062_500), 0);
    assert_eq!(signal::tone_sample(200_062_500), 0);
    assert_eq!(signal::tone_sample(2_000_062_500), 0);
    assert_eq!(signal::tone_sample(3_000_062_500), 1881);
}
