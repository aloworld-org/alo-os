//! A script, as Windows PowerShell takes one without reading it as a command line.
//!
//! `powershell.exe -Command` receives its script through the process's command
//! line, where quotes are re-parsed by two different sets of rules before
//! PowerShell sees them. `-EncodedCommand` takes the script as Base64 of its
//! UTF-16LE text instead, so what runs is byte for byte what `crate::program`
//! wrote, and no character in it — a quote, a semicolon, a space — means
//! anything to anybody on the way.

/// The Base64 alphabet (RFC 4648 §4).
const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

/// The script, as `-EncodedCommand` takes it.
#[must_use]
pub fn for_powershell(script: &str) -> String {
    let bytes: Vec<u8> = script.encode_utf16().flat_map(u16::to_le_bytes).collect();
    base64(&bytes)
}

/// Standard, padded Base64.
fn base64(bytes: &[u8]) -> String {
    let mut out = String::with_capacity(bytes.len().div_ceil(3) * 4);
    for chunk in bytes.chunks(3) {
        let [a, b, c] = match *chunk {
            [a] => [a, 0, 0],
            [a, b] => [a, b, 0],
            [a, b, c] => [a, b, c],
            _ => continue,
        };
        let joined = (u32::from(a) << 16) | (u32::from(b) << 8) | u32::from(c);
        for (position, shift) in [18_u32, 12, 6, 0].into_iter().enumerate() {
            if position <= chunk.len() {
                let index = usize::try_from((joined >> shift) & 0x3f).unwrap_or(0);
                out.push(char::from(ALPHABET.get(index).copied().unwrap_or(b'A')));
            } else {
                out.push('=');
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    /// RFC 4648 §10's vectors.
    #[test]
    fn base64_is_the_rfcs() {
        for (plain, encoded) in [
            ("", ""),
            ("f", "Zg=="),
            ("fo", "Zm8="),
            ("foo", "Zm9v"),
            ("foob", "Zm9vYg=="),
            ("fooba", "Zm9vYmE="),
            ("foobar", "Zm9vYmFy"),
        ] {
            assert_eq!(base64(plain.as_bytes()), encoded, "{plain}");
        }
    }

    /// The script is UTF-16LE before it is Base64, as PowerShell reads it.
    #[test]
    fn a_script_is_utf16_little_endian() {
        // `powershell -EncodedCommand ZABpAHIA` runs `dir`.
        assert_eq!(for_powershell("dir"), "ZABpAHIA");
    }
}
