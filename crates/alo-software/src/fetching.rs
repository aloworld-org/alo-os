//! Fetching a maker's release file onto this machine (ADR 0158).
//!
//! **Nothing fetched here is trusted.** A file arrives in a folder of its own
//! and is checked by `crate::installing_a_signed_release` — its checksum
//! against the pin, its signature against the maker's pinned key — before the
//! rented tool is handed it, and deleted whatever the answer.
//!
//! # One redirect, to one host, and why
//!
//! GitHub answers a release address with `302 Found` and a `Location` on
//! `release-assets.githubusercontent.com`, where the bytes are (measured
//! 2026-11-17; `docs/quirks/a-github-release-file-is-served-from-another-host.md`).
//! So one redirect is followed, and only to that host over `https`: a release
//! address that sent this machine anywhere else is refused. The indicator
//! shows the errand to the maker's release host, which is where the person's
//! machine was asked to go; the file host is GitHub's own, said in that quirk.

use std::io::Write as _;
use std::path::Path;

use alo_proxy::Carried;

use crate::tool::Failed;

/// The one host a release address may send this machine on to.
pub const WHERE_GITHUB_KEEPS_RELEASE_FILES: &str = "https://release-assets.githubusercontent.com/";

/// The largest file fetched: a gigabyte, ten times the largest Craft bundle on
/// 2026-11-17 (110 MB). A file larger than this is refused rather than filling
/// a disk.
pub const THE_LARGEST_FILE: u64 = 1 << 30;

/// Something that puts the file at an address into a file on this machine.
pub trait Fetcher {
    /// Fetch `address` into `into`, refusing a file larger than `at_most`.
    ///
    /// # Errors
    /// [`Failed::DidNotAnswer`] for anything that did not end with the whole
    /// file written.
    fn fetch(&self, address: &str, into: &Path, at_most: u64) -> Result<(), Failed>;
}

/// The network, on the road this machine takes out.
/// Not `Clone`, because `alo_proxy::Carried` is deliberately not: it can hold
/// a proxy password.
#[derive(Debug)]
pub struct TheNetwork {
    /// The proxy, or none, as `crate::road` decided it.
    taking: Carried,
}

impl TheNetwork {
    /// The network, through the road `crate::the_road_out` gave.
    #[must_use]
    pub const fn through(taking: Carried) -> Self {
        Self { taking }
    }

    /// The network to a maker's release host, on the road this machine takes
    /// out to install an application — the machine's own proxy, read from its
    /// own file, exactly as the rented tool's road is.
    ///
    /// # Errors
    /// [`crate::NoRoad`], when the way out cannot be decided or signed in to.
    pub fn to(maker: &crate::Maker) -> Result<Self, crate::NoRoad> {
        crate::the_road_out(alo_proxy::Road::InstallingAnApplication, maker.host())
            .map(Self::through)
    }

    /// One answer, with no redirect followed.
    fn asked(&self, address: &str) -> Result<ureq::http::Response<ureq::Body>, Failed> {
        let through = self
            .taking
            .for_a_request()
            .map_err(|_| Failed::DidNotAnswer {
                said: "the way out could not be read as an address".to_owned(),
            })?;
        ureq::get(address)
            .config()
            .max_redirects(0)
            .http_status_as_error(false)
            .proxy(through)
            .build()
            .call()
            .map_err(|why| Failed::DidNotAnswer {
                said: format!("{address} did not answer: {why}"),
            })
    }
}

impl Fetcher for TheNetwork {
    fn fetch(&self, address: &str, into: &Path, at_most: u64) -> Result<(), Failed> {
        let mut answered = self.asked(address)?;
        if answered.status().is_redirection() {
            let onward = answered
                .headers()
                .get("location")
                .and_then(|said| said.to_str().ok())
                .map(str::to_owned)
                .unwrap_or_default();
            if !onward.starts_with(WHERE_GITHUB_KEEPS_RELEASE_FILES) {
                return Err(Failed::DidNotAnswer {
                    said: format!("{address} sent this machine on to {onward:?}"),
                });
            }
            answered = self.asked(&onward)?;
        }
        if answered.status() != ureq::http::StatusCode::OK {
            return Err(Failed::DidNotAnswer {
                said: format!("{address} answered {}", answered.status()),
            });
        }
        let mut reading = answered.body_mut().with_config().limit(at_most).reader();
        let mut file = std::fs::File::create(into).map_err(|why| Failed::DidNotAnswer {
            said: format!("{} could not be written: {why}", into.display()),
        })?;
        std::io::copy(&mut reading, &mut file).map_err(|why| Failed::DidNotAnswer {
            said: format!("{address} was not all fetched: {why}"),
        })?;
        file.flush().map_err(|why| Failed::DidNotAnswer {
            said: format!("{} could not be written: {why}", into.display()),
        })
    }
}

/// **The limit is larger than any bundle the catalogue pins**, held when the
/// crate is built: PhotoCraft 0.23.1's bundle is 104,570,384 bytes.
const _: () = assert!(THE_LARGEST_FILE > 104_570_384);

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use std::io::{BufRead as _, BufReader};
    use std::net::TcpListener;

    use super::*;

    /// A server on this machine that answers each path the way it is told to,
    /// for as many requests as it is given answers, and its address.
    fn a_server(answers: Vec<(&'static str, String)>) -> String {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let address = format!("http://{}", listener.local_addr().unwrap());
        std::thread::spawn(move || {
            for _ in 0..answers.len() {
                let Ok((mut stream, _)) = listener.accept() else {
                    return;
                };
                let mut first = String::new();
                let mut reading = BufReader::new(stream.try_clone().unwrap());
                reading.read_line(&mut first).unwrap();
                loop {
                    let mut line = String::new();
                    if reading.read_line(&mut line).unwrap() == 0 || line == "\r\n" {
                        break;
                    }
                }
                let path = first.split(' ').nth(1).unwrap_or("").to_owned();
                let answer = answers
                    .iter()
                    .find(|(wanted, _)| *wanted == path)
                    .map_or_else(
                        || "HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\n\r\n".to_owned(),
                        |(_, answer)| answer.clone(),
                    );
                stream.write_all(answer.as_bytes()).unwrap();
            }
        });
        address
    }

    fn ok(body: &str) -> String {
        format!(
            "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
            body.len()
        )
    }

    fn straight() -> TheNetwork {
        TheNetwork::through(Carried::straight())
    }

    #[test]
    fn a_file_that_is_there_is_fetched_whole() {
        let at = a_server(vec![("/a.flatpak", ok("the bundle's bytes"))]);
        let folder = tempfile::tempdir().unwrap();
        let into = folder.path().join("a.flatpak");
        straight()
            .fetch(&format!("{at}/a.flatpak"), &into, THE_LARGEST_FILE)
            .unwrap();
        assert_eq!(std::fs::read(&into).unwrap(), b"the bundle's bytes");
    }

    /// **Anything but the file is refused**: an answer that is not *here it
    /// is*, and a file larger than it may be.
    #[test]
    fn a_missing_file_and_one_too_large_are_refused() {
        let at = a_server(vec![("/big", ok("more than four bytes"))]);
        let folder = tempfile::tempdir().unwrap();
        let into = folder.path().join("x");
        assert!(
            straight()
                .fetch(&format!("{at}/not-there"), &into, THE_LARGEST_FILE)
                .is_err(),
            "a 404 was fetched"
        );
        let at = a_server(vec![("/big", ok("more than four bytes"))]);
        assert!(
            straight().fetch(&format!("{at}/big"), &into, 4).is_err(),
            "a file larger than its limit was fetched"
        );
    }

    /// **A release address that sends this machine anywhere but GitHub's own
    /// file host is refused**, and the place it was sent is never asked.
    #[test]
    fn a_redirect_anywhere_else_is_refused() {
        let elsewhere = a_server(vec![("/stolen", ok("somebody else's bytes"))]);
        let at = a_server(vec![(
            "/a.flatpak",
            format!(
                "HTTP/1.1 302 Found\r\nLocation: {elsewhere}/stolen\r\nContent-Length: 0\r\n\r\n"
            ),
        )]);
        let folder = tempfile::tempdir().unwrap();
        let into = folder.path().join("a.flatpak");
        let answer = straight().fetch(&format!("{at}/a.flatpak"), &into, THE_LARGEST_FILE);
        assert!(
            matches!(answer, Err(Failed::DidNotAnswer { ref said }) if said.contains("sent this machine on")),
            "{answer:?}"
        );
        assert!(!into.exists(), "a file was written");
    }
}
