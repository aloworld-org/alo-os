//! **The face a terminal draws its cells in is fixed-pitch, and this checks
//! rather than trusts the filename.**
//!
//! # Why a test and not a line in the README
//!
//! A proportional face in a terminal **does not fail loudly**. It draws a grid
//! whose columns do not line up — and every test that asserts cell dimensions,
//! viewport arithmetic or scrollback geometry still passes, because those are
//! about the cell box and not about what is inside it. The failure is visible
//! only to a person looking at a screen, which is the last place this product
//! should learn about it.
//!
//! This crate has already shipped the neighbouring fault once. `Family::Name`
//! asking for a face no database held gave back, in
//! `crate::window_control_label`'s own words, *a box of the right size with
//! nothing in it* — and a booted machine showed the wallpaper, the monogram's
//! square and the entry field, and **not one word, including the person's own
//! name**. Correct geometry, no glyphs, no error.
//!
//! So: the file is read and asked what it is.
//!
//! # What each assertion is for
//!
//! - **The face is present**, which is the positive control: a path that is
//!   wrong makes every assertion below it unreachable, and a test that cannot
//!   find its subject must say so rather than pass.
//! - **`post.isFixedPitch` is set**, which is the font's own declaration that
//!   every advance is equal. This is the one that catches a well-meaning swap.
//! - **The bytes are the ones whose provenance is recorded**, so the README's
//!   revision and hash describe the file actually in the tree.
//! - **It carries a weight axis**, because a bold cell is weight 700 chosen
//!   through `Attrs::weight`. A static face would need a second file and the
//!   drawing code would silently draw regular where it meant bold.

// **Counted rather than guessed at**, because an unfulfilled `#[expect]` is
// itself an error here and the first version of this block listed
// `clippy::panic`, which nothing in the file uses. Measured: `.expect(` three,
// `panic!` zero, indexing seven, slicing one.
#![expect(
    clippy::expect_used,
    reason = "an unreadable or malformed face is the failure this test reports"
)]
#![expect(
    clippy::indexing_slicing,
    reason = "this file parses a binary format by offset, which is what indexing is for. The \
              bounds are the font's own table directory and the file's hash is asserted beside \
              this, so an index past the end means the bytes are not the face whose provenance \
              is recorded — which is a failure this test exists to report rather than one to \
              handle gracefully."
)]

use std::path::{Path, PathBuf};

/// Where the bundled faces live, from this crate's own manifest directory.
fn fonts() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("fonts")
}

/// The recorded hash, which `fonts/README.md` carries beside the revision it
/// came from.
const THE_RECORDED_HASH: &str = "d00e590b8eb3a59acc329b2d044fd143ae935090b7da33199ebee27cc7de8196";

/// Every table in a TrueType file, by tag, as `(offset, length)`.
///
/// Hand-rolled rather than taken from a font crate, because this test exists to
/// check the file on disk and a parser that normalised it, substituted a
/// fallback or reported a default for a missing table would hide exactly what
/// is being asked.
fn tables(bytes: &[u8]) -> Vec<(String, usize)> {
    let count = u16::from_be_bytes([bytes[4], bytes[5]]) as usize;
    let mut found = Vec::new();
    for table in 0..count {
        let at = 12 + 16 * table;
        let tag = String::from_utf8_lossy(&bytes[at..at + 4]).to_string();
        let offset =
            u32::from_be_bytes([bytes[at + 8], bytes[at + 9], bytes[at + 10], bytes[at + 11]])
                as usize;
        found.push((tag, offset));
    }
    found
}

#[test]
fn the_terminal_draws_its_cells_in_a_fixed_pitch_face() {
    let at = fonts().join("GeistMono.ttf");

    // **The positive control, first.** Everything below reads this file; a
    // wrong path would make the whole test unreachable, and an unreachable
    // assertion passes.
    assert!(
        at.is_file(),
        "no face at {} — a terminal would ask for a monospace family, find nothing, and draw a \
         grid of correctly sized empty boxes. See this crate's `window_control_label`, which \
         records a booted machine doing exactly that.",
        at.display()
    );

    let bytes = std::fs::read(&at).expect("the bundled face is readable");
    let tables = tables(&bytes);

    // **The assertion this file exists for.**
    let (_, post) = tables
        .iter()
        .find(|(tag, _)| tag == "post")
        .expect("a TrueType face carries a `post` table");
    let is_fixed_pitch = u32::from_be_bytes([
        bytes[post + 12],
        bytes[post + 13],
        bytes[post + 14],
        bytes[post + 15],
    ]);
    assert_ne!(
        is_fixed_pitch, 0,
        "the bundled face reports `post.isFixedPitch = 0`, which means it is **proportional**. A \
         terminal drawn in it has columns that do not line up, and no geometry test notices, \
         because the cell box is right and only its contents are wrong. If the face was changed \
         deliberately, change the terminal's own drawing with it."
    );

    // **The weight axis**, because bold is a weight rather than a second file.
    assert!(
        tables.iter().any(|(tag, _)| tag == "fvar"),
        "the bundled face has no `fvar` table, so it is static rather than variable. A bold cell \
         is weight 700 asked for through `Attrs::weight`; against a static face that request is \
         silently satisfied by the regular weight, and bold output stops being bold without \
         anything failing."
    );
}

/// **The bytes are the ones whose provenance `fonts/README.md` records.**
///
/// Not a security control — anybody editing the file can edit the README — but
/// it keeps the two from drifting, which is how a recorded revision comes to
/// describe something that is no longer there.
#[test]
fn the_face_is_the_one_whose_provenance_is_recorded() {
    let bytes = std::fs::read(fonts().join("GeistMono.ttf")).expect("the bundled face is readable");
    let hash = sha256(&bytes);
    assert_eq!(
        hash, THE_RECORDED_HASH,
        "the bundled face is not the one `fonts/README.md` describes. Either the file changed and \
         the record did not, or the record changed and the file did not — and a provenance note \
         that does not describe the bytes beside it is worse than none, because it is believed."
    );
}

/// SHA-256, written out rather than taken from a dependency.
///
/// This crate has no hashing dependency and adding one so a test can read a
/// constant would be the larger change. Thirty lines, from FIPS 180-4.
fn sha256(message: &[u8]) -> String {
    const K: [u32; 64] = [
        0x428a_2f98,
        0x7137_4491,
        0xb5c0_fbcf,
        0xe9b5_dba5,
        0x3956_c25b,
        0x59f1_11f1,
        0x923f_82a4,
        0xab1c_5ed5,
        0xd807_aa98,
        0x1283_5b01,
        0x2431_85be,
        0x550c_7dc3,
        0x72be_5d74,
        0x80de_b1fe,
        0x9bdc_06a7,
        0xc19b_f174,
        0xe49b_69c1,
        0xefbe_4786,
        0x0fc1_9dc6,
        0x240c_a1cc,
        0x2de9_2c6f,
        0x4a74_84aa,
        0x5cb0_a9dc,
        0x76f9_88da,
        0x983e_5152,
        0xa831_c66d,
        0xb003_27c8,
        0xbf59_7fc7,
        0xc6e0_0bf3,
        0xd5a7_9147,
        0x06ca_6351,
        0x1429_2967,
        0x27b7_0a85,
        0x2e1b_2138,
        0x4d2c_6dfc,
        0x5338_0d13,
        0x650a_7354,
        0x766a_0abb,
        0x81c2_c92e,
        0x9272_2c85,
        0xa2bf_e8a1,
        0xa81a_664b,
        0xc24b_8b70,
        0xc76c_51a3,
        0xd192_e819,
        0xd699_0624,
        0xf40e_3585,
        0x106a_a070,
        0x19a4_c116,
        0x1e37_6c08,
        0x2748_774c,
        0x34b0_bcb5,
        0x391c_0cb3,
        0x4ed8_aa4a,
        0x5b9c_ca4f,
        0x682e_6ff3,
        0x748f_82ee,
        0x78a5_636f,
        0x84c8_7814,
        0x8cc7_0208,
        0x90be_fffa,
        0xa450_6ceb,
        0xbef9_a3f7,
        0xc671_78f2,
    ];
    let mut h: [u32; 8] = [
        0x6a09_e667,
        0xbb67_ae85,
        0x3c6e_f372,
        0xa54f_f53a,
        0x510e_527f,
        0x9b05_688c,
        0x1f83_d9ab,
        0x5be0_cd19,
    ];
    let mut padded = message.to_vec();
    let bits = (message.len() as u64) * 8;
    padded.push(0x80);
    while padded.len() % 64 != 56 {
        padded.push(0);
    }
    padded.extend_from_slice(&bits.to_be_bytes());

    for block in padded.chunks(64) {
        let mut w = [0u32; 64];
        for (index, word) in w.iter_mut().enumerate().take(16) {
            let at = index * 4;
            *word = u32::from_be_bytes([block[at], block[at + 1], block[at + 2], block[at + 3]]);
        }
        for index in 16..64 {
            let s0 = w[index - 15].rotate_right(7)
                ^ w[index - 15].rotate_right(18)
                ^ (w[index - 15] >> 3);
            let s1 = w[index - 2].rotate_right(17)
                ^ w[index - 2].rotate_right(19)
                ^ (w[index - 2] >> 10);
            w[index] = w[index - 16]
                .wrapping_add(s0)
                .wrapping_add(w[index - 7])
                .wrapping_add(s1);
        }
        let [mut a, mut b, mut c, mut d, mut e, mut f, mut g, mut hh] = h;
        for index in 0..64 {
            let s1 = e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25);
            let ch = (e & f) ^ ((!e) & g);
            let t1 = hh
                .wrapping_add(s1)
                .wrapping_add(ch)
                .wrapping_add(K[index])
                .wrapping_add(w[index]);
            let s0 = a.rotate_right(2) ^ a.rotate_right(13) ^ a.rotate_right(22);
            let maj = (a & b) ^ (a & c) ^ (b & c);
            let t2 = s0.wrapping_add(maj);
            hh = g;
            g = f;
            f = e;
            e = d.wrapping_add(t1);
            d = c;
            c = b;
            b = a;
            a = t1.wrapping_add(t2);
        }
        for (slot, value) in h.iter_mut().zip([a, b, c, d, e, f, g, hh]) {
            *slot = slot.wrapping_add(value);
        }
    }
    h.iter().map(|word| format!("{word:08x}")).collect()
}

/// **The hashing above is itself checked**, against the empty string's
/// published digest.
///
/// Thirty hand-written lines of bit arithmetic that nothing verifies would be a
/// test whose own instrument could be wrong — and a wrong hash fails the test
/// above for a reason that has nothing to do with the face.
#[test]
fn the_hashing_this_test_does_is_correct() {
    assert_eq!(
        sha256(b""),
        "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
    );
    assert_eq!(
        sha256(b"abc"),
        "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
    );
}
