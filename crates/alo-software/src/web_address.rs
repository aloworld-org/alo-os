//! What a web address is, checked before anything is decided about it.
//!
//! `docs/autonomy/v0-5-software-and-the-web-plan.md`, task 3: the browser opens
//! web addresses **from any application**. An address that arrives that way is
//! the least trustworthy value on the machine — it is written by a web page, a
//! message somebody was sent, or a file that came off a disk — and it ends up
//! being handed to an application as an argument. So it is a checked value with
//! a closed list of refusals, the way every argument a verb takes is (ADR 0001
//! §2), and never a string passed along.
//!
//! # Only the open web
//!
//! `http` and `https`, and nothing else. `file:` would turn *open this link*
//! into *read this path*, which is the grant model gone; `javascript:` and
//! `data:` are code and a document written by the asking application, wearing
//! an address; `about:` reaches into the browser's own settings. None of them
//! is the open web, so none of them is here — and each is refused naming the
//! scheme it asked for, because *what tried* is the part worth writing down.
//!
//! # A password in an address is refused rather than passed on
//!
//! `https://anna:hunter2@example.com/` hands a name and a password to whatever
//! opens it, writes them into that application's history, and is the shape a
//! forged address takes when it wants the part before the `@` read as the site.
//! It is refused ([`NotAWebAddress::CarriesAPassword`]), and the person is told
//! to sign in on the site instead.
//!
//! # Nothing that arrived is read back to a person
//!
//! [`crate::words::NOT_A_WEB_ADDRESS`] says *that is not a web address* without
//! repeating what arrived. An address is attacker-written text, and a sentence
//! that quotes it is a sentence whoever wrote it helped compose. The value keeps
//! the scheme — which is short, checked and the only part worth a record — and
//! nothing else.

/// The most bytes a web address may have.
///
/// Longer than any address a person types or a site publishes, and short enough
/// that nothing here is ever handed a megabyte of text to walk. Browsers stop
/// well before this; the number is ours, and refusing is not the browser's job.
pub const LONGEST: usize = 2048;

/// Why what arrived was not a web address.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum NotAWebAddress {
    /// Nothing, or nothing but blank space.
    #[error("no web address was given")]
    Nothing,
    /// No scheme at all: nothing this could be resolved against.
    #[error("that is not a web address: it does not begin with http:// or https://")]
    NoScheme,
    /// A scheme, and not one of the open web's two.
    #[error("a {scheme}: address is not the open web, so it is not opened as a web address")]
    NotAWebScheme {
        /// What it asked for, lower-cased — `file`, `javascript`, `data`.
        scheme: String,
    },
    /// No host, or one no address could name.
    #[error("that web address names no site")]
    NotAHost,
    /// A name and a password written into the address.
    #[error("that web address carries a name and a password in it")]
    CarriesAPassword,
    /// Blank space or a control character: two lines wearing one address.
    #[error("that web address is not one line")]
    NotOneLine,
    /// Longer than [`LONGEST`].
    #[error("that web address is longer than {LONGEST} bytes")]
    TooLong,
}

/// A web address on the open web, checked.
///
/// Built only by [`WebAddress::checked`], so holding one is the proof that the
/// checks at the top of this file were made.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WebAddress {
    /// The address as it arrived, trimmed of the blank space around it.
    address: String,
    /// Where the host begins in it.
    host_at: usize,
    /// How long the host is.
    host_len: usize,
    /// Whether it is `https`.
    secure: bool,
}

impl WebAddress {
    /// A web address, or why what arrived is not one.
    ///
    /// # Errors
    /// [`NotAWebAddress`], naming the first thing that does not hold.
    pub fn checked(address: &str) -> Result<Self, NotAWebAddress> {
        let address = address.trim();
        if address.is_empty() {
            return Err(NotAWebAddress::Nothing);
        }
        if address.len() > LONGEST {
            return Err(NotAWebAddress::TooLong);
        }
        if address
            .chars()
            .any(|letter| letter.is_control() || letter.is_whitespace())
        {
            return Err(NotAWebAddress::NotOneLine);
        }
        let (scheme, rest) = address
            .split_once("://")
            .ok_or_else(|| scheme_of(address))?;
        let secure = if scheme.eq_ignore_ascii_case("https") {
            true
        } else if scheme.eq_ignore_ascii_case("http") {
            false
        } else {
            return Err(NotAWebAddress::NotAWebScheme {
                scheme: scheme.to_lowercase(),
            });
        };
        let authority = match rest.find(['/', '?', '#']) {
            Some(at) => rest.get(..at).ok_or(NotAWebAddress::NotAHost)?,
            None => rest,
        };
        if authority.contains('@') {
            return Err(NotAWebAddress::CarriesAPassword);
        }
        let host = host_of(authority)?;
        let host_at = address.len() - rest.len() + usize::from(numbered(authority));
        Ok(Self {
            address: address.to_owned(),
            host_at,
            host_len: host.len(),
            secure,
        })
    }

    /// The address, as it will be handed to whatever opens it.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.address
    }

    /// The site it names, without the port and without the brackets an address
    /// written to a numbered host has around it.
    ///
    /// What an egress line would name, and what a person reads when they are
    /// asked whether they meant to go there.
    #[must_use]
    pub fn host(&self) -> &str {
        self.address
            .get(self.host_at..self.host_at.saturating_add(self.host_len))
            .unwrap_or_default()
    }

    /// Whether the address is `https`.
    #[must_use]
    pub const fn is_secure(&self) -> bool {
        self.secure
    }
}

/// Whether this authority writes its host as a number in brackets.
fn numbered(authority: &str) -> bool {
    authority.starts_with('[')
}

/// Which refusal an address with no `://` earns: the scheme it did name, or
/// none at all.
fn scheme_of(address: &str) -> NotAWebAddress {
    match address.split_once(':') {
        Some((scheme, _))
            if !scheme.is_empty()
                && scheme.chars().all(|letter| {
                    letter.is_ascii_alphanumeric() || matches!(letter, '+' | '-')
                }) =>
        {
            NotAWebAddress::NotAWebScheme {
                scheme: scheme.to_lowercase(),
            }
        }
        _ => NotAWebAddress::NoScheme,
    }
}

/// The host inside an authority, held to being one — a name, or a number in
/// brackets — with the port, where there is one, digits.
fn host_of(authority: &str) -> Result<&str, NotAWebAddress> {
    let (host, port, in_brackets) = match authority.strip_prefix('[') {
        Some(inside) => {
            let (host, after) = inside.split_once(']').ok_or(NotAWebAddress::NotAHost)?;
            match after.strip_prefix(':') {
                Some(port) => (host, Some(port), true),
                None if after.is_empty() => (host, None, true),
                None => return Err(NotAWebAddress::NotAHost),
            }
        }
        None => match authority.split_once(':') {
            Some((host, port)) => (host, Some(port), false),
            None => (authority, None, false),
        },
    };
    if host.is_empty() {
        return Err(NotAWebAddress::NotAHost);
    }
    let held = if in_brackets {
        host.chars()
            .all(|letter| letter.is_ascii_hexdigit() || matches!(letter, ':' | '.'))
    } else {
        host.chars()
            .all(|letter| letter.is_ascii_alphanumeric() || matches!(letter, '-' | '.' | '_' | '~'))
    };
    if !held {
        return Err(NotAWebAddress::NotAHost);
    }
    match port {
        Some(port) if port.is_empty() || !port.chars().all(|digit| digit.is_ascii_digit()) => {
            Err(NotAWebAddress::NotAHost)
        }
        _ => Ok(host),
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;

    /// **An ordinary address is kept as it arrived**, and its site read off it.
    #[test]
    fn an_address_on_the_open_web_is_kept_as_it_arrived() {
        for (given, host, secure) in [
            ("https://example.com/", "example.com", true),
            ("http://example.com", "example.com", false),
            ("  https://example.com/a?b=c#d  ", "example.com", true),
            ("https://example.com:8443/a", "example.com", true),
            ("HTTPS://example.com/", "example.com", true),
            ("https://[2001:db8::1]:8443/a", "2001:db8::1", true),
            ("https://[::1]", "::1", true),
            ("http://127.0.0.1:1965/", "127.0.0.1", false),
        ] {
            let address = WebAddress::checked(given).unwrap();
            assert_eq!(address.as_str(), given.trim(), "{given}");
            assert_eq!(address.host(), host, "{given}");
            assert_eq!(address.is_secure(), secure, "{given}");
        }
    }

    /// **Nothing but the open web is a web address**, and each refusal names
    /// the scheme that tried rather than the text that carried it.
    #[test]
    fn nothing_but_the_open_web_is_a_web_address() {
        for (given, scheme) in [
            ("file:///etc/shadow", "file"),
            ("FILE:///etc/shadow", "file"),
            ("javascript:alert(1)", "javascript"),
            ("data:text/html,<script>", "data"),
            ("about:config", "about"),
            ("ftp://example.com/", "ftp"),
            ("chrome://settings", "chrome"),
        ] {
            assert_eq!(
                WebAddress::checked(given).unwrap_err(),
                NotAWebAddress::NotAWebScheme {
                    scheme: scheme.to_owned()
                },
                "{given}"
            );
        }
        for given in ["example.com", "/home/anna/march.pdf", "www.example.com/a"] {
            assert_eq!(
                WebAddress::checked(given).unwrap_err(),
                NotAWebAddress::NoScheme,
                "{given}"
            );
        }
    }

    /// **A password written into an address is refused**, wherever the `@` is
    /// in the authority — and an `@` after the site, which is a path and not a
    /// password, is not.
    #[test]
    fn a_password_written_into_an_address_is_refused() {
        for given in [
            "https://anna:hunter2@example.com/",
            "https://anna@example.com/",
            "http://example.com@evil.example/",
        ] {
            assert_eq!(
                WebAddress::checked(given).unwrap_err(),
                NotAWebAddress::CarriesAPassword,
                "{given}"
            );
        }
        assert_eq!(
            WebAddress::checked("https://example.com/mail/anna@example.com")
                .unwrap()
                .host(),
            "example.com"
        );
    }

    /// **Nothing shapeless, empty, over-long or two lines gets through.**
    #[test]
    fn nothing_shapeless_gets_through() {
        for (given, refused) in [
            ("", NotAWebAddress::Nothing),
            ("   ", NotAWebAddress::Nothing),
            ("https://exa mple.com/", NotAWebAddress::NotOneLine),
            ("https://example.com/\nGET /", NotAWebAddress::NotOneLine),
            ("https:///a", NotAWebAddress::NotAHost),
            ("https://:8443/", NotAWebAddress::NotAHost),
            ("https://example.com:/", NotAWebAddress::NotAHost),
            ("https://example.com:http/", NotAWebAddress::NotAHost),
            ("https://[2001:db8::1/", NotAWebAddress::NotAHost),
            ("https://[::1]x/", NotAWebAddress::NotAHost),
            ("https://exa<mple.com/", NotAWebAddress::NotAHost),
        ] {
            assert_eq!(WebAddress::checked(given).unwrap_err(), refused, "{given}");
        }
        let long = format!("https://example.com/{}", "a".repeat(LONGEST));
        assert_eq!(
            WebAddress::checked(&long).unwrap_err(),
            NotAWebAddress::TooLong
        );
        let longest = format!("https://example.com/{}", "a".repeat(LONGEST - 20));
        assert_eq!(longest.len(), LONGEST);
        assert!(WebAddress::checked(&longest).is_ok());
    }
}
