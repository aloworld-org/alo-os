//! How far apart two colours are to look at, measured the way the standard
//! measures it.
//!
//! **This is here because a colour set is a claim about legibility.** ADR 0010
//! offers five accents, each with a value for a light ground and one for a dark,
//! and says outright that the values are a designer's proposal rather than a
//! measurement. A proposal nobody measures is how a hue that reads beautifully
//! in a design tool reaches somebody who cannot read it — so the measurement is
//! here, and [`crate::accent`] holds every accent to it as a test.
//!
//! **The numbers are EN 301 549's**, by way of WCAG 2.1 §1.4.3 and §1.4.11,
//! which it points at: ordinary text needs [`ENOUGH_FOR_TEXT`] against what is
//! behind it, and a shape carrying meaning — a focus ring, the fill of a
//! selected row — needs [`ENOUGH_FOR_A_SHAPE`]. EN 301 549 is what an EU
//! public-sector desktop is procured against, so these are a requirement rather
//! than a preference.
//!
//! What this does **not** do is answer whether two colours can be told apart by
//! somebody who cannot distinguish a hue. Contrast is a ratio of luminance and
//! says nothing about colour blindness, which is precisely why ADR 0010 puts a
//! mark and a word beside the agent's colour rather than trusting the colour.

use crate::colour::Colour;

/// What ordinary text has to reach against what is behind it, from WCAG 2.1
/// §1.4.3 by way of EN 301 549.
pub const ENOUGH_FOR_TEXT: f64 = 4.5;

/// What a shape that carries meaning has to reach — a focus ring, a filled
/// state, an icon that is the only thing saying something — from WCAG 2.1
/// §1.4.11.
pub const ENOUGH_FOR_A_SHAPE: f64 = 3.0;

impl Colour {
    /// How much light this colour sends back, 0 for black and 1 for white.
    ///
    /// The sRGB relative luminance the standard defines, which is not the same
    /// as how bright a person would call it: green counts for far more than blue
    /// because an eye is built that way.
    #[must_use]
    pub fn relative_luminance(self) -> f64 {
        let red = straightened(self.red());
        let green = straightened(self.green());
        let blue = straightened(self.blue());
        0.2126f64.mul_add(red, 0.7152f64.mul_add(green, 0.0722 * blue))
    }

    /// How far apart these two are to look at: 1 for the same colour, 21 for
    /// black against white.
    ///
    /// Which one is in front and which behind does not change the answer, so a
    /// caller never has to decide the order.
    #[must_use]
    pub fn contrast_with(self, other: Self) -> f64 {
        let one = self.relative_luminance();
        let two = other.relative_luminance();
        let (lighter, darker) = if one >= two { (one, two) } else { (two, one) };
        (lighter + 0.05) / (darker + 0.05)
    }
}

/// One channel, 0 to 255, as the fraction of light it stands for.
///
/// The curve is the standard's: a screen does not emit twice the light for twice
/// the number, so the number is straightened before any of it is added up.
fn straightened(channel: u8) -> f64 {
    let part = f64::from(channel) / 255.0;
    if part <= 0.03928 {
        part / 12.92
    } else {
        ((part + 0.055) / 1.055).powf(2.4)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::token::Token;

    /// How close two measurements have to be to count as the same. The standard
    /// quotes contrast to one decimal place, so this is finer than anything the
    /// answer is used for.
    const CLOSE_ENOUGH: f64 = 0.001;

    /// **The two ends are the ones the standard names.** Black against white is
    /// 21:1 and a colour against itself is 1:1 — if either of those moves, the
    /// arithmetic is wrong and every accent measured with it is meaningless.
    #[test]
    fn the_ends_of_the_scale_are_where_the_standard_puts_them() {
        let black = Colour::of(0, 0, 0);
        let white = Colour::of(255, 255, 255);
        assert!((black.contrast_with(white) - 21.0).abs() < CLOSE_ENOUGH);
        assert!((black.relative_luminance() - 0.0).abs() < CLOSE_ENOUGH);
        assert!((white.relative_luminance() - 1.0).abs() < CLOSE_ENOUGH);
        for token in Token::ALL {
            let colour = token.colour();
            assert!(
                (colour.contrast_with(colour) - 1.0).abs() < CLOSE_ENOUGH,
                "{} against itself",
                token.word().says()
            );
        }
    }

    /// The order of the two colours is not part of the question, because a
    /// caller asking *does this read* should not have to know which is brighter.
    #[test]
    fn which_one_is_in_front_does_not_change_the_answer() {
        let navy = Token::Navy.colour();
        let cream = Token::Cream.colour();
        assert!((navy.contrast_with(cream) - cream.contrast_with(navy)).abs() < CLOSE_ENOUGH);
    }

    /// **The palette's own text pair is measured, not assumed.** `text/primary`
    /// on `bg/canvas` is what the design puts most of the words on, and it
    /// clears ordinary text by a distance — 13.70:1, checked against the same
    /// arithmetic every contrast checker in the world implements.
    ///
    /// **13.56:1 until 2026-10-04**, when the canvas took the design file's
    /// `#FAF7F2` in place of `#F8F6F2` (ADR 0093). A slightly lighter ground
    /// reads slightly better, and the number is re-measured rather than
    /// adjusted — the arithmetic was reproduced against the old value first and
    /// agreed to five places.
    #[test]
    fn navy_on_cream_reads_and_the_number_is_the_published_one() {
        let measured = Token::Navy.colour().contrast_with(Token::Cream.colour());
        assert!(
            (measured - 13.7013).abs() < 0.001,
            "navy on cream measured {measured}"
        );
        assert!(measured >= ENOUGH_FOR_TEXT);
    }

    /// **Deep teal on the canvas clears both thresholds** — 5.84:1, over the
    /// 4.5 a word needs and the 3.0 a shape carrying meaning needs.
    /// **Terracotta**, which it replaced, reached **2.87:1** and reached
    /// neither, so alo's colour has gone from unreadable as text on the reading
    /// ground to comfortably readable. This is one of the three measurements
    /// ADR 0067 said were owed.
    ///
    /// **This sentence said "Deep teal, which it replaced" until 2026-10-04**,
    /// which named deep teal as its own predecessor and made the comparison
    /// unreadable. ADR 0067 replaced *terracotta* with deep teal.
    ///
    /// **5.78:1 until 2026-10-04**, against cream `#F8F6F2`; the canvas now
    /// carries the design file's `#FAF7F2` (ADR 0093) and the pair measures
    /// 5.84:1. Terracotta against the same new ground is 2.89:1 — still under
    /// both thresholds, so nothing ADR 0067 concluded moves.
    ///
    /// **And the mark and the word are still not optional**, which is the part
    /// worth keeping straight. Their reason has changed: it used to be that the
    /// colour could not be read at all, and now it is the durable one — **hue is
    /// never a signal**, because around one man in twelve cannot rely on it and
    /// EN 301 549 has said so for twenty years. A colour that passes contrast is
    /// still a colour somebody cannot distinguish from navy, which is why
    /// `docs/design/who-is-acting.md` asks whether a person can say who is
    /// acting **without naming the colour**.
    #[test]
    fn deep_teal_on_cream_is_readable_and_still_not_a_signal_on_its_own() {
        let measured = Token::DeepTeal
            .colour()
            .contrast_with(Token::Cream.colour());
        assert!(
            (measured - 5.8375).abs() < 0.001,
            "deep teal on cream measured {measured}"
        );
        assert!(measured >= ENOUGH_FOR_A_SHAPE);
        assert!(measured >= ENOUGH_FOR_TEXT);
    }
}
