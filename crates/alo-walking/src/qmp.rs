//! **Talking to qemu's machine protocol (QMP)**: one JSON object per line each
//! way, a greeting first, and asynchronous events mixed in with the answers.
//!
//! The old shell helpers spoke to the human monitor, and learned two things
//! the hard way that this keeps: **hold the connection and read the answer**
//! (a command written and closed in one breath was never read, and a
//! `screendump` came back as a 0-byte file with no error), and **an answer is
//! the line that says `return` or `error`** — anything else on the wire, an
//! event qemu volunteers, is set aside rather than mistaken for it.
//!
//! Generic over the reader and the writer so the tests can speak both halves
//! without a qemu.

use std::io::{BufRead, Write};

use serde_json::{Value, json};

/// What went wrong talking to qemu.
#[derive(Debug, thiserror::Error)]
pub enum QmpError {
    /// The connection itself failed.
    #[error("the connection to qemu failed: {0}")]
    Io(#[from] std::io::Error),
    /// qemu closed the connection.
    #[error("qemu closed the connection")]
    Closed,
    /// A line from qemu was not JSON.
    #[error("qemu said something that is not JSON: {0}")]
    NotJson(String),
    /// qemu did not greet as QMP does.
    #[error("qemu did not greet with QMP: {0}")]
    NoGreeting(String),
    /// qemu answered a command with an error.
    #[error("qemu refused {command}: {said}")]
    Refused {
        /// The command it refused.
        command: String,
        /// What it said.
        said: String,
    },
}

/// A QMP session, past its greeting and with commands enabled.
pub struct Qmp<R, W> {
    /// Where qemu's lines come from.
    from: R,
    /// Where commands go.
    to: W,
    /// Events qemu volunteered while a command waited for its answer.
    events: Vec<Value>,
}

impl<R: BufRead, W: Write> Qmp<R, W> {
    /// Read the greeting and leave capabilities negotiation, so commands may
    /// be sent.
    ///
    /// # Errors
    ///
    /// When the greeting is not QMP's, or qemu refuses `qmp_capabilities`.
    pub fn open(from: R, to: W) -> Result<Self, QmpError> {
        let mut qmp = Self {
            from,
            to,
            events: Vec::new(),
        };
        let greeting = qmp.line()?;
        if greeting.get("QMP").is_none() {
            return Err(QmpError::NoGreeting(greeting.to_string()));
        }
        qmp.execute(&json!({ "execute": "qmp_capabilities" }))?;
        Ok(qmp)
    }

    /// Send one command and wait for its answer, setting aside any events.
    ///
    /// # Errors
    ///
    /// When the connection fails, or qemu answers with an error.
    pub fn execute(&mut self, command: &Value) -> Result<Value, QmpError> {
        let mut line = command.to_string();
        line.push('\n');
        self.to.write_all(line.as_bytes())?;
        self.to.flush()?;
        loop {
            let said = self.line()?;
            if let Some(answer) = said.get("return") {
                return Ok(answer.clone());
            }
            if let Some(error) = said.get("error") {
                return Err(QmpError::Refused {
                    command: command
                        .get("execute")
                        .and_then(Value::as_str)
                        .unwrap_or("a command")
                        .to_owned(),
                    said: error.to_string(),
                });
            }
            self.events.push(said);
        }
    }

    /// The events qemu volunteered so far, taken.
    pub fn events(&mut self) -> Vec<Value> {
        std::mem::take(&mut self.events)
    }

    /// One line from qemu, as JSON.
    fn line(&mut self) -> Result<Value, QmpError> {
        let mut text = String::new();
        if self.from.read_line(&mut text)? == 0 {
            return Err(QmpError::Closed);
        }
        serde_json::from_str(text.trim()).map_err(|_| QmpError::NotJson(text.trim().to_owned()))
    }
}

#[cfg(test)]
#[path = "qmp_tests.rs"]
mod tests;
