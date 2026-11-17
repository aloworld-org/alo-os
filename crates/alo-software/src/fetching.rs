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
