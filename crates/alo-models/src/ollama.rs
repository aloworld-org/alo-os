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
//! It does not *offer* a model the catalogue does not list. Ollama's own
//! library is not curated and states no licences; `docs/features.md` promises a
//! curated catalogue with licences, and that promise would be hollow if any
//! name could be downloaded through here. So [`Ollama::fetch`] refuses an id the
//! catalogue does not carry, with [`RuntimeError::NotOffered`] — and
//! [`Ollama::answers`] deliberately does not, because offering is downloading:
//! a model already on somebody's disk is theirs, and refusing to ask it
//! anything would be alo OS overruling the owner of the machine about their own
//! hardware rather than keeping a promise about what we ship.
//!
//! And it exposes no way to send an arbitrary request. The trait forbids it
//! (law 2), and an adapter that quietly added one would put the escape hatch
//! back where the trait had removed it. [`Ollama::answers`] is not that hatch
//! and is worth saying so about: it carries a question to a model, which is
//! text for a model to read, never a command for this machine to run.
//!
//! # What a question does to this file, and what it does not
//!
//! A question is the only thing here that is somebody's own words, and it is
//! borrowed for the length of one request. It goes into one JSON body and
//! nowhere else: not into a `RuntimeError`, which has no field for it, and not
//! into anything this file keeps. ADR 0001 §7 is what that is for, and
//! `a_question_goes_into_the_body_and_nowhere_else` is the test.

use std::time::Duration;

use serde::{Deserialize, Serialize};

use crate::catalogue::Catalogue;
use crate::runtime::{Installed, Loaded, ModelRuntime, Progress, ProgressSink, RuntimeError};

/// Where Ollama listens by default. The same endpoint `alo-workplace`'s
/// `AiConfig` has documented since 2025, which is why pointing the agents at a
/// local model is configuration rather than new code.
pub const DEFAULT_ENDPOINT: &str = "http://127.0.0.1:11434";

/// How long to wait on a call that should be quick. Listing what is installed
/// is a local read; if it has not answered in this long, the runtime is not
/// well, and saying so beats hanging a user interface.
///
/// A timeout here stays [`RuntimeError::Unreachable`] rather than becoming
/// [`RuntimeError::TookTooLong`], and the difference is real: a listing that
/// takes ten seconds means something is wrong, while a model that takes five
/// minutes means it is thinking. Only [`Ollama::answers`] can say the second.
const QUICK_TIMEOUT: Duration = Duration::from_secs(10);

/// How long this machine waits for a model on it to answer.
///
/// Longer than the two minutes `alo_asking::hosted` waits for a provider, and
/// deliberately: ADR 0007 makes the CPU the default, and a model thinking on a
/// CPU is slower than the same question put to somebody's service on a card
/// they own. **This machine waits longer for itself than for anybody else.**
/// Finite all the same — a wait with no end is indistinguishable from a hang,
/// and [`RuntimeError::TookTooLong`] is a sentence a person can act on.
const WHILE_A_MODEL_THINKS: Duration = Duration::from_secs(300);

/// The most of an answer that is read.
///
/// The reply is on this machine and is still not read without a bound: no model
/// writes a megabyte in one answer, so this costs nothing real, and an answer
/// past it is [`RuntimeError::Unusable`] rather than half of what the model
/// said. `alo_asking::hosted` holds a provider to the same number for the same
/// reason.
const MOST_OF_AN_ANSWER: u64 = 1_000_000;

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

/// `/api/chat` — one question, put to a model on this machine.
///
/// Ollama's own chat call rather than its OpenAI-compatible one. Both work; this
/// is the runtime speaking its own language, which is what keeps ADR 0006's
/// promise literal — the file that knows Ollama exists uses Ollama's API, and
/// nothing about our shape is chosen to look like somebody else's.
#[derive(Serialize)]
struct ChatRequest<'a> {
    /// Ollama's `family:tag` name, applied here and nowhere else.
    model: String,
    /// The conversation, which is one message: alo OS composes no preamble of
    /// its own and sends no previous turn.
    messages: [ChatMessage<'a>; 1],
    /// Whole answers only. Nothing in this repository has decided what a
    /// half-arrived answer is, so none is asked for.
    stream: bool,
}

/// One message in that call.
#[derive(Serialize)]
struct ChatMessage<'a> {
    /// Who is speaking. Always the person.
    role: &'a str,
    /// What they asked. Borrowed, and this is the only place it goes.
    content: &'a str,
}

/// What `/api/chat` answers with.
#[derive(Deserialize)]
struct ChatResponse {
    /// What the model wrote, absent on a reply that is not an answer.
    #[serde(default)]
    message: Option<ChatSaid>,
}

/// The message a model wrote.
#[derive(Deserialize)]
struct ChatSaid {
    /// Its text.
    #[serde(default)]
    content: String,
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
            // never carry a backend response body, and since item 9f they
            // cannot — the reason is a variant with a string of its own rather
            // than a sentence this file wrote.
            return Err(RuntimeError::DownloadIncomplete);
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

    fn answers(&self, question: &str, of_model: &str) -> Result<String, RuntimeError> {
        let body = ChatRequest {
            model: Self::runtime_name(of_model),
            messages: [ChatMessage {
                role: "user",
                content: question,
            }],
            stream: false,
        };
        let response = ureq::post(format!("{}/api/chat", self.endpoint))
            .config()
            .timeout_global(Some(WHILE_A_MODEL_THINKS))
            .build()
            .send_json(&body);
        let mut response = match response {
            Ok(response) => response,
            // Ollama answers 404 for a model it does not hold. The person asked
            // a model that is not on this machine, which is a different thing
            // from the runtime not being there.
            Err(ureq::Error::StatusCode(404)) => {
                return Err(RuntimeError::NotInstalled(of_model.to_owned()));
            }
            // It is there and it is thinking. ADR 0007's CPU default makes this
            // ordinary rather than broken, and saying "nothing was running"
            // would send somebody to look at a runtime that is busy.
            Err(ureq::Error::Timeout(_)) => return Err(RuntimeError::TookTooLong),
            // Something answered, and not with an answer. Its own words are not
            // repeated: this crate's errors never carry a backend response body.
            Err(ureq::Error::StatusCode(_)) => return Err(RuntimeError::Unusable),
            Err(_) => return Err(RuntimeError::Unreachable),
        };
        let body = response
            .body_mut()
            .with_config()
            .limit(MOST_OF_AN_ANSWER)
            .read_to_string()
            .map_err(|_| RuntimeError::Unusable)?;
        let said: ChatResponse = serde_json::from_str(&body).map_err(|_| RuntimeError::Unusable)?;
        let said = said.message.map(|m| m.content).unwrap_or_default();
        if said.trim().is_empty() {
            // A reply with no answer in it is not an empty answer to show
            // somebody: they asked something and nothing came back.
            return Err(RuntimeError::Unusable);
        }
        Ok(said)
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
    use super::*;
    use crate::testing::serving;

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
        assert_eq!(err, RuntimeError::DownloadIncomplete);
        // The runtime's own words must not travel, and there is now nowhere
        // for them to travel in: the variant carries nothing.
        let said = err.said(&crate::testing::in_english());
        assert!(!said.text().contains("manifest"), "{said}");
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

    /// **The path the zero-egress promise is about.** A question goes to a
    /// model on this machine, the answer comes back as the model wrote it, and
    /// the whole of what went out is the model, the one message and
    /// `stream: false` — no preamble alo OS composed, no previous turn, nothing
    /// identifying the machine or the person.
    #[test]
    fn a_question_goes_into_the_body_and_nowhere_else() {
        let (url, server) = serving(
            r#"{"model":"mistral-7b-instruct:latest","message":{"role":"assistant","content":"No, not without written consent."},"done":true}"#,
            200,
        );
        let answer = Ollama::at(&url, catalogue())
            .answers("may the tenant sublet?", "mistral-7b-instruct")
            .unwrap();
        let request = server.join().unwrap();

        assert_eq!(answer, "No, not without written consent.");
        assert!(request.starts_with("POST /api/chat "), "{request}");
        let body = request.split("\r\n\r\n").nth(1).unwrap_or_default();
        let sent: serde_json::Value = serde_json::from_str(body).unwrap();
        assert_eq!(
            sent,
            serde_json::json!({
                // Ollama's naming convention, applied here and nowhere else.
                "model": "mistral-7b-instruct:latest",
                "messages": [{"role": "user", "content": "may the tenant sublet?"}],
                "stream": false,
            }),
            "{request}"
        );
    }

    /// **A question is somebody's own words and this crate keeps none of
    /// them.** It goes into one body; nothing that comes back out of this file
    /// carries it, because no error here has a field it could travel in.
    #[test]
    fn a_question_never_comes_back_out_in_anything_this_file_answers_with() {
        let (url, server) = serving(r#"{"error":"model not found"}"#, 404);
        let err = Ollama::at(&url, catalogue())
            .answers("what is in this contract?", "mistral-7b-instruct")
            .unwrap_err();
        server.join().unwrap();
        assert!(
            matches!(&err, RuntimeError::NotInstalled(id) if id == "mistral-7b-instruct"),
            "{err:?}"
        );
        let rendered = format!("{err:?}");
        assert!(!rendered.contains("contract"), "{rendered}");
        let said = err.said(&crate::testing::in_english()).text().to_owned();
        assert!(!said.contains("contract"), "{said}");
    }

    /// **Asking is not gated by the catalogue, and downloading is.** A model on
    /// somebody's own disk is theirs; the licence promise is about what alo OS
    /// offers to fetch, and refusing to use what is already installed would be
    /// this system overruling the owner of the machine.
    #[test]
    fn a_model_the_catalogue_does_not_list_can_still_be_asked_if_it_is_installed() {
        let (url, server) = serving(
            r#"{"message":{"role":"assistant","content":"it answered anyway"}}"#,
            200,
        );
        let ollama = Ollama::at(&url, catalogue());
        let answer = ollama.answers("anything?", "some-uncurated-model").unwrap();
        let request = server.join().unwrap();
        assert_eq!(answer, "it answered anyway");
        assert!(request.contains("some-uncurated-model:latest"), "{request}");

        // And the gate that is a gate is still shut, on a runtime nothing is
        // listening on — so a leak would read as Unreachable rather than pass.
        let err = Ollama::at("http://127.0.0.1:1", catalogue())
            .fetch("some-uncurated-model", &mut Progress::ignored())
            .unwrap_err();
        assert!(matches!(err, RuntimeError::NotOffered(_)), "{err:?}");
    }

    /// Something answered, and it was not an answer. Four shapes of the same
    /// sentence to the person who asked: they asked, and nothing came back.
    #[test]
    fn a_reply_that_is_not_an_answer_is_unusable_rather_than_shown() {
        for (body, status) in [
            ("<!doctype html><title>Welcome</title>", 200),
            (r#"{"done":true}"#, 200),
            (r#"{"message":{"role":"assistant","content":"   "}}"#, 200),
            (r#"{"error":"something at that end"}"#, 500),
        ] {
            let (url, server) = serving(body, status);
            let err = Ollama::at(&url, catalogue())
                .answers("may the tenant sublet?", "mistral-7b-instruct")
                .unwrap_err();
            server.join().unwrap();
            assert_eq!(err, RuntimeError::Unusable, "{body}");
            // Whatever it said about itself does not travel into what a person
            // reads: there is nowhere in the variant for it to travel.
            let said = err.said(&crate::testing::in_english()).text().to_owned();
            assert!(!said.contains("something at that end"), "{said}");
        }
    }

    /// A runtime that is not running is not a model that is slow, and the two
    /// sentences send a person to different places.
    #[test]
    fn nothing_listening_is_unreachable_and_not_a_model_thinking() {
        let err = Ollama::at("http://127.0.0.1:1", catalogue())
            .answers("may the tenant sublet?", "mistral-7b-instruct")
            .unwrap_err();
        assert_eq!(err, RuntimeError::Unreachable);
        assert_ne!(err, RuntimeError::TookTooLong);
        let said = RuntimeError::TookTooLong.said(&crate::testing::in_english());
        assert!(said.text().contains("did not answer in the time"), "{said}");
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
