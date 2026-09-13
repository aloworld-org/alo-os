//! Handing a file on this disk to the runtime, as the pinned runtime takes one.
//!
//! [`crate::ModelRuntime::bring`] used to tell Ollama about a brought file with
//! a one-line Modelfile — `{"modelfile": "FROM /path/to/weights.gguf"}` — and
//! every test in this crate agreed, because every test answered with a socket
//! this repository wrote. The runtime the image pins, **Ollama 0.34.0**, was
//! asked on 2026-09-13 and answered `400 {"error":"neither 'from' or 'files' was
//! specified"}`: that field is gone from its create API. *Point alo OS at
//! weights you already have and it runs them* had never been walked against the
//! program it depends on.
//!
//! # What 0.34.0 takes instead, measured rather than read
//!
//! Two requests, both on loopback:
//!
//! 1. **The file goes into the runtime's store under its own digest.**
//!    `HEAD /api/blobs/sha256:<digest>` answers `200` when the runtime already
//!    holds those bytes and `404` when it does not; `POST` to the same address
//!    with the file as the body answers `201`, and a body whose digest is not the
//!    one named is refused `400` with *digest mismatch*. So the runtime checks
//!    the bytes it was given against the name it was given, and this crate
//!    cannot hand it a different file by mistake.
//! 2. **The model is created from that blob by name.**
//!    `POST /api/create` with `{"model": <id>, "files": {<file name>:
//!    "sha256:<digest>"}}` answers `200 {"status":"success"}`, and afterwards the
//!    runtime answers questions put to `<id>`. A file that is not weights is
//!    refused there, `500 {"error":"unexpected EOF"}`.
//!
//! **`from` naming a path is refused** — `400 {"error":"invalid model name"}` —
//! which is the half of this that matters for law 1: `from` is how a create
//! names a model to *fetch*, and there is no spelling of it that turns a path on
//! this disk into a download. The road below never sends it.
//!
//! # The cost, stated
//!
//! The runtime keeps its own copy: a brought file of five gigabytes is five more
//! gigabytes in the runtime's store, once. The digest is read off the whole file
//! first, which on the measuring machine is a few seconds a gigabyte. Neither is
//! egress, and both happen once per file — a second `bring` of the same bytes
//! finds them already held and sends nothing but the create.

use std::fs::File;
use std::io::{BufReader, Read as _};
use std::path::Path;
use std::time::Duration;

use crate::runtime::RuntimeError;

/// How much of a file is read at a time while its digest is taken.
const A_READ: usize = 1 << 20;

/// **The file's SHA-256, as the runtime's store names blobs**: sixty-four
/// lowercase hexadecimal characters.
///
/// # Errors
/// [`RuntimeError::NotAPathOnThisDisk`] when the file cannot be read to the end
/// — it was there when the door checked and is not now, which is the same fact
/// for the person who pointed at it.
pub(crate) fn digest_of(file: &Path) -> Result<String, RuntimeError> {
    let unreadable = || RuntimeError::NotAPathOnThisDisk(file.to_path_buf());
    let mut reading = BufReader::with_capacity(A_READ, File::open(file).map_err(|_| unreadable())?);
    let mut digesting = ring::digest::Context::new(&ring::digest::SHA256);
    let mut chunk = vec![0u8; A_READ];
    loop {
        let read = reading.read(&mut chunk).map_err(|_| unreadable())?;
        let Some(read_bytes) = chunk.get(..read) else {
            return Err(unreadable());
        };
        if read_bytes.is_empty() {
            break;
        }
        digesting.update(read_bytes);
    }
    Ok(digesting
        .finish()
        .as_ref()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect())
}

/// **The file, in the runtime's store under that digest** — sent only when the
/// runtime does not already hold those bytes.
///
/// # Errors
/// [`RuntimeError::Unreachable`] when nothing answers,
/// [`RuntimeError::TookTooLong`] when copying the file outlasts `wait`,
/// [`RuntimeError::Unusable`] when the runtime refuses the bytes — its digest
/// check failing, which means the file changed between being read and being
/// sent — and [`RuntimeError::NotAPathOnThisDisk`] when the file cannot be
/// opened to send.
pub(crate) fn handed_over(
    endpoint: &str,
    file: &Path,
    digest: &str,
    wait: Duration,
) -> Result<(), RuntimeError> {
    let blob = format!("{endpoint}/api/blobs/sha256:{digest}");
    match ureq::head(&blob)
        .config()
        .timeout_global(Some(wait))
        .build()
        .call()
    {
        Ok(_) => return Ok(()),
        Err(ureq::Error::StatusCode(404)) => {}
        Err(ureq::Error::Timeout(_)) => return Err(RuntimeError::TookTooLong),
        Err(ureq::Error::StatusCode(_)) => return Err(RuntimeError::Unusable),
        Err(_) => return Err(RuntimeError::Unreachable),
    }
    let body =
        File::open(file).map_err(|_| RuntimeError::NotAPathOnThisDisk(file.to_path_buf()))?;
    match ureq::post(&blob)
        .config()
        .timeout_global(Some(wait))
        .build()
        .send(body)
    {
        Ok(_) => Ok(()),
        Err(ureq::Error::Timeout(_)) => Err(RuntimeError::TookTooLong),
        Err(ureq::Error::StatusCode(_)) => Err(RuntimeError::Unusable),
        Err(_) => Err(RuntimeError::Unreachable),
    }
}

/// **The create request 0.34.0 takes**: the id to answer to, and the one file
/// it is made from, by name and digest.
///
/// Nothing else is in it — no template, no system prompt, no parameters —
/// because anything further would be this machine putting words in a model
/// somebody else brought.
pub(crate) fn the_create(model: &str, file_name: &str, digest: &str) -> serde_json::Value {
    serde_json::json!({
        "model": model,
        "files": { file_name: format!("sha256:{digest}") },
        "stream": false,
    })
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;

    /// A file on this disk with these bytes.
    fn a_file_holding(called: &str, bytes: &[u8]) -> std::path::PathBuf {
        let at =
            std::env::temp_dir().join(format!("alo-handing-over-{called}-{}", std::process::id()));
        std::fs::write(&at, bytes).unwrap();
        at
    }

    /// **The digest is SHA-256, spelled as the runtime's store spells it** —
    /// checked against the published vector for `abc` and the empty input, so
    /// a wrong algorithm or a wrong spelling cannot pass.
    #[test]
    fn the_digest_is_the_sha256_the_store_names_blobs_by() {
        let abc = a_file_holding("abc", b"abc");
        assert_eq!(
            digest_of(&abc).unwrap(),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
        let empty = a_file_holding("empty", b"");
        assert_eq!(
            digest_of(&empty).unwrap(),
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
        // Longer than one read, so the loop is walked more than once.
        let long = a_file_holding("long", &vec![b'a'; A_READ * 2 + 3]);
        assert_eq!(digest_of(&long).unwrap().len(), 64);
        for file in [abc, empty, long] {
            drop(std::fs::remove_file(file));
        }
    }

    /// A file that is not there cannot be digested, and says which path.
    #[test]
    fn a_file_that_went_away_is_refused_naming_it() {
        let gone = std::env::temp_dir().join("alo-handing-over-never-there.gguf");
        assert_eq!(
            digest_of(&gone).unwrap_err(),
            RuntimeError::NotAPathOnThisDisk(gone)
        );
    }

    /// **The create names the file by digest and nothing else**, and never
    /// carries `from` — the field that names a model to fetch.
    #[test]
    fn the_create_carries_the_file_by_digest_and_never_a_model_to_fetch() {
        let create = the_create(
            "theirs.gguf:latest",
            "theirs.gguf",
            "ab".repeat(32).as_str(),
        );
        assert_eq!(create.get("model").unwrap(), "theirs.gguf:latest");
        assert_eq!(
            create.pointer("/files/theirs.gguf").unwrap(),
            &format!("sha256:{}", "ab".repeat(32))
        );
        assert_eq!(create.get("stream").unwrap(), false);
        assert!(create.get("from").is_none(), "{create}");
        assert!(create.get("modelfile").is_none(), "{create}");
        assert_eq!(create.as_object().unwrap().len(), 3, "{create}");
    }
}
