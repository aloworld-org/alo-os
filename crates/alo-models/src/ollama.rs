//! The Ollama adapter — **the only file in this repository that knows Ollama
//! exists**.
//!
//! [ADR 0006](../../../docs/decisions/0006-the-pinned-model-runtime.md) pins
//! Ollama as the runtime alo OS ships and puts [`ModelRuntime`] in front of it.
//! That promise is only worth anything if it is kept literally: every endpoint
//! path, every field name and every naming convention Ollama has lives here, so
//! a reviewer asking "what would replacing the runtime cost?" can read one file
//! and know.
//!
//! Two things this deliberately does **not** do.
//!
//! It does not offer a model the catalogue does not list. Ollama's own library
//! is not curated and states no licences; `docs/features.md` promises a curated
//! catalogue with licences, and that promise would be hollow if any name could
//! be passed through. So [`Ollama::fetch`] refuses an id the catalogue does not
//! carry, with [`RuntimeError::NotOffered`].
//!
//! And it exposes no way to send an arbitrary request. The trait forbids it
//! (law 2), and an adapter that quietly added one would put the escape hatch
//! back where the trait had removed it.

use std::time::Duration;

use serde::Deserialize;

use crate::catalogue::Catalogue;
use crate::runtime::{Installed, Loaded, ModelRuntime, Progress, ProgressSink, RuntimeError};

/// Where Ollama listens by default. The same endpoint `alo-workplace`'s
/// `AiConfig` has documented since 2025, which is why pointing the agents at a
/// local model is configuration rather than new code.
pub const DEFAULT_ENDPOINT: &str = "http://127.0.0.1:11434";

/// How long to wait on a call that should be quick. Listing what is installed
/// is a local read; if it has not answered in this long, the runtime is not
/// well, and saying so beats hanging a user interface.
const QUICK_TIMEOUT: Duration = Duration::from_secs(10);

/// How long a deliberately loaded model stays in video memory without use.
/// Long enough that a person who loaded it on purpose does not find it gone by
/// the time they have finished typing; short enough that a forgotten model
/// eventually gives the card back.
const DEFAULT_KEEP_ALIVE: &str = "30m";

/// Ollama, reached over its HTTP API.
#[derive(Debug, Clone)]
pub struct Ollama {
    /// Where the runtime listens, without a trailing slash.
    endpoint: String,
    /// What alo OS offers. Held here because it is the gate on `fetch`.
    catalogue: Catalogue,
}

// Ollama's wire shapes. Every field is `#[serde(default)]` on purpose: a
// runtime that adds, renames or omits a field between versions should cost us
// a missing value, never a failed parse of an otherwise good response.

/// `/api/tags` — what is on disk.
#[derive(Deserialize)]
struct TagsResponse {
    /// One entry per set of weights the runtime holds.
    #[serde(default)]
    models: Vec<TagEntry>,
}

/// One installed model, as `/api/tags` describes it.
#[derive(Deserialize)]
struct TagEntry {
    /// Ollama's own `family:tag` name, which never escapes this file.
    #[serde(default)]
    name: String,
    /// Bytes on disk, as the runtime measures them.
    #[serde(default)]
    size: u64,
    /// Present on recent versions; absent on older ones.
    #[serde(default)]
    details: Option<TagDetails>,
}

/// The nested detail block of a `/api/tags` entry.
#[derive(Deserialize)]
struct TagDetails {
    /// The quantisation actually installed, spelled Ollama's way.
    #[serde(default)]
    quantization_level: Option<String>,
}

/// `/api/ps` — what is in video memory now.
#[derive(Deserialize)]
struct PsResponse {
    /// One entry per loaded model.
    #[serde(default)]
    models: Vec<PsEntry>,
}

/// One loaded model, as `/api/ps` describes it.
#[derive(Deserialize)]
struct PsEntry {
    /// Ollama's `family:tag` name.
    #[serde(default)]
    name: String,
    /// Video memory held. The same object also carries the disk size, which is
    /// the wrong number for this question.
    #[serde(default)]
    size_vram: u64,
}

/// One line of `/api/pull`'s streamed response.
#[derive(Deserialize)]
struct PullLine {
    /// Bytes fetched so far, absent on lines that only report status.
    #[serde(default)]
    completed: Option<u64>,
    /// Bytes expected, which the runtime does not always know at the start.
    #[serde(default)]
    total: Option<u64>,
    /// Set when the download failed. Its text is never repeated to a caller.
    #[serde(default)]
    error: Option<String>,
}

impl Ollama {
    /// An adapter pointed at the default endpoint.
    #[must_use]
    pub fn new(catalogue: Catalogue) -> Self {
        Self::at(DEFAULT_ENDPOINT, catalogue)
    }

    /// An adapter pointed somewhere else — a test server, or a runtime on a
    /// paired machine once shared inference arrives (ADR 0003).
    #[must_use]
    pub fn at(endpoint: &str, catalogue: Catalogue) -> Self {
        Self {
            endpoint: endpoint.trim_end_matches('/').to_owned(),
            catalogue,
        }
    }

    /// Ollama names a model `family:tag`; our catalogue ids are our own. The
    /// mapping is one line today and may not stay that way, but it stays *here*
    /// either way.
    fn runtime_name(id: &str) -> String {
        if id.contains(':') {
            id.to_owned()
        } else {
            format!("{id}:latest")
        }
    }

    /// …and back, so what the runtime reports can be matched to the catalogue.
    fn catalogue_id(runtime_name: &str) -> String {
        runtime_name
            .strip_suffix(":latest")
            .unwrap_or(runtime_name)
            .to_owned()
    }

    /// One GET against the runtime, read to a string. Only for the small
    /// replies — a download is streamed, not buffered.
    fn get(&self, path: &str) -> Result<String, RuntimeError> {
        ureq::get(format!("{}{path}", self.endpoint))
            .config()
            .timeout_global(Some(QUICK_TIMEOUT))
            .build()
            .call()
            .map_err(|_| RuntimeError::Unreachable)?
            .body_mut()
            .read_to_string()
            .map_err(|_| RuntimeError::Unusable)
    }
}

impl ModelRuntime for Ollama {
    fn installed(&self) -> Result<Vec<Installed>, RuntimeError> {
        let body = self.get("/api/tags")?;
        let tags: TagsResponse = serde_json::from_str(&body).map_err(|_| RuntimeError::Unusable)?;
        Ok(tags
            .models
            .into_iter()
            .map(|m| Installed {
                id: Self::catalogue_id(&m.name),
                bytes_on_disk: m.size,
                quantisation: m.details.and_then(|d| d.quantization_level),
            })
            .collect())
    }

    fn loaded(&self) -> Result<Vec<Loaded>, RuntimeError> {
        let body = self.get("/api/ps")?;
        let ps: PsResponse = serde_json::from_str(&body).map_err(|_| RuntimeError::Unusable)?;
        Ok(ps
            .models
            .into_iter()
            .map(|m| Loaded {
                id: Self::catalogue_id(&m.name),
                vram_bytes: m.size_vram,
            })
            .collect())
    }

    fn fetch(&self, id: &str, progress: &mut dyn ProgressSink) -> Result<(), RuntimeError> {
        // The catalogue gate. Without it, any name could be pulled and the
        // licence promise in docs/features.md would mean nothing.
        if self.catalogue.get(id).is_none() {
            return Err(RuntimeError::NotOffered(id.to_owned()));
        }

        let body = serde_json::json!({ "model": Self::runtime_name(id), "stream": true });
        let response = ureq::post(format!("{}/api/pull", self.endpoint))
            .send_json(&body)
            .map_err(|_| RuntimeError::Unreachable)?;

        // The response is a stream of JSON objects, one per line, for as long as
        // the download runs. Read it line by line rather than to a string: these
        // are gigabytes, and a progress report that arrives at the end is not a
        // progress report.
        let reader = std::io::BufReader::new(response.into_body().into_reader());
        let mut saw_error = None;
        {
            use std::io::BufRead as _;
            for line in reader.lines() {
                let Ok(line) = line else {
                    return Err(RuntimeError::Unreachable);
                };
                if line.trim().is_empty() {
                    continue;
                }
                let Ok(parsed) = serde_json::from_str::<PullLine>(&line) else {
                    // A line we cannot read is not a reason to abandon a
                    // multi-gigabyte download; the next one usually parses.
                    continue;
                };
                if parsed.error.is_some() {
                    saw_error = Some(());
                    continue;
                }
                if let Some(done) = parsed.completed {
                    progress.advanced(Progress {
                        done_bytes: done,
                        total_bytes: parsed.total,
                    });
                }
            }
        }
        if saw_error.is_some() {
            // The runtime's own words are not repeated: this crate's errors
            // never carry a backend response body.
            return Err(RuntimeError::Refused("the download did not complete"));
        }
        Ok(())
    }

    fn remove(&self, id: &str) -> Result<(), RuntimeError> {
        // Ollama deletes with a DELETE *carrying a body*, which is unusual
        // enough that ureq's `delete()` builder cannot express it — its DELETE
        // is body-less. Building the request directly is the way through, and
        // is exactly the kind of runtime-shaped awkwardness this file exists to
        // absorb.
        let body = serde_json::json!({ "model": Self::runtime_name(id) }).to_string();
        let request = ureq::http::Request::builder()
            .method("DELETE")
            .uri(format!("{}/api/delete", self.endpoint))
            .header("content-type", "application/json")
            .body(body)
            .map_err(|_| RuntimeError::Unusable)?;
        let response = ureq::Agent::new_with_defaults().run(request);
        match response {
            Ok(_) => Ok(()),
            // Ollama answers 404 for a model it does not have. That is not a
            // failure to report as one — the caller asked for the disk back and
            // the disk is already back.
            Err(ureq::Error::StatusCode(404)) => Err(RuntimeError::NotInstalled(id.to_owned())),
            Err(_) => Err(RuntimeError::Unreachable),
        }
    }

    fn load(&self, id: &str) -> Result<(), RuntimeError> {
        // Ollama has no load call either: an empty generate with a non-zero
        // keep-alive brings the weights into video memory and leaves them
        // there. Same endpoint as unload, opposite keep-alive — which is
        // precisely why both belong in this file and neither belongs in the
        // trait's vocabulary.
        self.keep_alive(id, DEFAULT_KEEP_ALIVE)
    }

    fn unload(&self, id: &str) -> Result<(), RuntimeError> {
        self.keep_alive(id, "0")
    }
}

impl Ollama {
    /// Ask the runtime to hold a model in video memory for `keep_alive`, or to
    /// let it go when that is `"0"`.
    fn keep_alive(&self, id: &str, keep_alive: &str) -> Result<(), RuntimeError> {
        let body = serde_json::json!({
            "model": Self::runtime_name(id),
            "keep_alive": keep_alive,
        });
        let response = ureq::post(format!("{}/api/generate", self.endpoint))
            .config()
            .timeout_global(Some(QUICK_TIMEOUT))
            .build()
            .send_json(&body);
        match response {
            Ok(_) => Ok(()),
            Err(ureq::Error::StatusCode(404)) => Err(RuntimeError::NotInstalled(id.to_owned())),
            Err(_) => Err(RuntimeError::Unreachable),
        }
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    clippy::indexing_slicing,
    reason = "in a test, a panic on a bad index or a None is the failure being reported"
)]
mod tests {
    use std::io::{BufRead as _, Read as _, Write as _};
    use std::net::TcpListener;
    use std::thread;

    use super::*;

    /// A stand-in Ollama: one request, one canned reply, on a real socket.
    ///
    /// A real server rather than a mocked HTTP client, because the thing worth
    /// testing is what goes out on the wire and what we make of what comes
    /// back — which a mock at the client boundary would assume rather than
    /// check.
    fn serving(response_body: &'static str, status: u16) -> (String, thread::JoinHandle<String>) {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        let handle = thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            // Read the request head, and the body if one was announced.
            let mut reader = std::io::BufReader::new(stream.try_clone().unwrap());
            let mut head = String::new();
            let mut length = 0usize;
            loop {
                let mut line = String::new();
                if reader.read_line(&mut line).unwrap() == 0 {
                    break;
                }
                if let Some(v) = line.to_ascii_lowercase().strip_prefix("content-length:") {
                    length = v.trim().parse().unwrap_or(0);
                }
                let done = line == "\r\n" || line == "\n";
                head.push_str(&line);
                if done {
                    break;
                }
            }
            let mut body = vec![0u8; length];
            if length > 0 {
                reader.read_exact(&mut body).unwrap();
            }
            let reply = format!(
                "HTTP/1.1 {status} OK\r\nContent-Length: {}\r\nContent-Type: application/json\r\nConnection: close\r\n\r\n{response_body}",
                response_body.len()
            );
            stream.write_all(reply.as_bytes()).unwrap();
            stream.flush().unwrap();
            head + &String::from_utf8_lossy(&body)
        });
        (format!("http://127.0.0.1:{port}"), handle)
    }

    fn catalogue() -> Catalogue {
        Catalogue::built_in().unwrap()
    }

    /// Compare request bodies without depending on how the HTTP client chooses
    /// to lay JSON out — it pretty-prints today and may not tomorrow, and a
    /// test that fails over a space is a test that will be silenced.
    fn without_spaces(s: &str) -> String {
        s.chars().filter(|c| !c.is_whitespace()).collect()
    }

    #[test]
    fn installed_reads_what_is_on_disk_and_drops_the_latest_tag() {
        let (url, server) = serving(
            r#"{"models":[{"name":"mistral-7b-instruct:latest","size":4370000000,"details":{"quantization_level":"Q4_K_M"}}]}"#,
            200,
        );
        let got = Ollama::at(&url, catalogue()).installed().unwrap();
        server.join().unwrap();
        assert_eq!(
            got,
            vec![Installed {
                // ":latest" is Ollama's convention, not ours, and must not
                // escape this file into catalogue ids.
                id: "mistral-7b-instruct".to_owned(),
                bytes_on_disk: 4_370_000_000,
                quantisation: Some("Q4_K_M".to_owned()),
            }]
        );
    }

    #[test]
    fn loaded_reads_video_memory_not_disk() {
        let (url, server) = serving(
            r#"{"models":[{"name":"teuken-7b-instruct:latest","size":4600000000,"size_vram":6100000000}]}"#,
            200,
        );
        let got = Ollama::at(&url, catalogue()).loaded().unwrap();
        server.join().unwrap();
        assert_eq!(
            got,
            vec![Loaded {
                id: "teuken-7b-instruct".to_owned(),
                // The disk size is right there in the same object and is the
                // wrong number: what a loaded model costs is VRAM.
                vram_bytes: 6_100_000_000,
            }]
        );
    }

    /// The catalogue gate, which is what makes the licence promise real. No
    /// request should even be attempted for a model we do not offer.
    #[test]
    fn fetching_a_model_the_catalogue_does_not_offer_is_refused_without_asking_the_runtime() {
        // Port 1 is not listening; if the gate leaked we would get Unreachable
        // instead of NotOffered, and the test would say so.
        let ollama = Ollama::at("http://127.0.0.1:1", catalogue());
        let err = ollama
            .fetch("some-uncurated-model", &mut Progress::ignored())
            .unwrap_err();
        assert!(
            matches!(&err, RuntimeError::NotOffered(id) if id == "some-uncurated-model"),
            "{err:?}"
        );
    }

    #[test]
    fn fetching_reports_progress_as_the_download_advances() {
        let (url, server) = serving(
            "{\"status\":\"pulling\",\"completed\":100,\"total\":400}\n\
             {\"status\":\"pulling\",\"completed\":400,\"total\":400}\n\
             {\"status\":\"success\"}\n",
            200,
        );
        let mut seen: Vec<Progress> = Vec::new();
        let result = Ollama::at(&url, catalogue()).fetch("mistral-7b-instruct", &mut |p| {
            seen.push(p);
        });
        let request = server.join().unwrap();
        assert!(result.is_ok(), "{result:?}");
        assert_eq!(seen.len(), 2, "{seen:?}");
        assert_eq!(seen[0].fraction(), Some(0.25));
        assert_eq!(seen[1].fraction(), Some(1.0));
        // The name that went out is Ollama's convention, applied here and
        // nowhere else.
        assert!(request.contains("mistral-7b-instruct:latest"), "{request}");
    }

    /// A malformed line mid-download must not abandon several gigabytes of
    /// progress. The next line almost always parses.
    #[test]
    fn a_line_that_does_not_parse_does_not_end_the_download() {
        let (url, server) = serving(
            "{\"completed\":100,\"total\":400}\n\
             not json at all\n\
             {\"completed\":400,\"total\":400}\n",
            200,
        );
        let mut seen = 0;
        let result = Ollama::at(&url, catalogue()).fetch("mistral-7b-instruct", &mut |_p| {
            seen += 1;
        });
        server.join().unwrap();
        assert!(result.is_ok(), "{result:?}");
        assert_eq!(seen, 2);
    }

    /// An error line in the stream is a failed download, and our error says so
    /// without repeating whatever the runtime wrote.
    #[test]
    fn an_error_in_the_stream_fails_the_fetch_without_quoting_the_runtime() {
        let (url, server) = serving(
            r#"{"error":"pull model manifest: file does not exist"}"#,
            200,
        );
        let err = Ollama::at(&url, catalogue())
            .fetch("mistral-7b-instruct", &mut Progress::ignored())
            .unwrap_err();
        server.join().unwrap();
        assert!(matches!(err, RuntimeError::Refused(_)), "{err:?}");
        assert!(
            !err.to_string().contains("manifest"),
            "the runtime's own words must not travel: {err}"
        );
    }

    #[test]
    fn removing_something_that_is_not_there_says_so_rather_than_failing_vaguely() {
        let (url, server) = serving(r#"{"error":"model not found"}"#, 404);
        let err = Ollama::at(&url, catalogue())
            .remove("mistral-7b-instruct")
            .unwrap_err();
        server.join().unwrap();
        assert!(
            matches!(&err, RuntimeError::NotInstalled(id) if id == "mistral-7b-instruct"),
            "{err:?}"
        );
    }

    #[test]
    fn a_runtime_that_is_not_listening_is_unreachable_rather_than_a_panic() {
        let ollama = Ollama::at("http://127.0.0.1:1", catalogue());
        assert!(matches!(
            ollama.installed().unwrap_err(),
            RuntimeError::Unreachable
        ));
    }

    /// Loading and unloading are the same Ollama endpoint with opposite
    /// keep-alives — it has neither a load nor an unload call. What these two
    /// tests hold is that the difference is a keep-alive and that it never
    /// leaves this file.
    #[test]
    fn loading_asks_the_runtime_to_hold_the_model_in_video_memory() {
        let (url, server) = serving("{}", 200);
        let result = Ollama::at(&url, catalogue()).load("mistral-7b-instruct");
        let request = server.join().unwrap();
        assert!(result.is_ok(), "{result:?}");
        assert!(request.contains("mistral-7b-instruct:latest"), "{request}");
        assert!(
            without_spaces(&request).contains(&format!(r#""keep_alive":"{DEFAULT_KEEP_ALIVE}""#)),
            "loading must ask for a non-zero keep-alive: {request}"
        );
    }

    #[test]
    fn unloading_asks_the_runtime_to_stop_holding_video_memory() {
        let (url, server) = serving("{}", 200);
        let result = Ollama::at(&url, catalogue()).unload("mistral-7b-instruct");
        let request = server.join().unwrap();
        assert!(result.is_ok(), "{result:?}");
        assert!(request.contains("mistral-7b-instruct:latest"), "{request}");
        assert!(
            without_spaces(&request).contains(r#""keep_alive":"0""#),
            "unloading must ask for a zero keep-alive: {request}"
        );
    }

    #[test]
    fn loading_something_not_installed_says_so_rather_than_failing_vaguely() {
        let (url, server) = serving(r#"{"error":"model not found"}"#, 404);
        let err = Ollama::at(&url, catalogue())
            .load("mistral-7b-instruct")
            .unwrap_err();
        server.join().unwrap();
        assert!(
            matches!(&err, RuntimeError::NotInstalled(id) if id == "mistral-7b-instruct"),
            "{err:?}"
        );
    }
}
