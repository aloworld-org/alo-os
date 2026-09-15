//! A printing service on this machine's loopback, speaking the protocol the
//! real one speaks, and remembering every request it was sent.
//!
//! **What it is and what it is not.** It reads a request exactly as the
//! printing service would — HTTP around an IPP message, read with the crate's
//! own decoder — and answers with whatever the test says, so both sides of every
//! exchange can be watched: what this crate sent, and what it made of the
//! answer. It is not CUPS, and a green run here says nothing about a printer on
//! a certified machine. That is owed, and the report says so.
//!
//! Shared by every test file through `mod serving;`.

#![allow(
    dead_code,
    reason = "each test file uses a different part of the service"
)]
#![expect(
    clippy::expect_used,
    reason = "in a fixture, a panic on an unexpected None or Err is the failure being reported"
)]

use std::io::{Read, Write};
use std::net::{Ipv4Addr, SocketAddr, TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread;

use alo_printing::PrintingService;
use alo_printing::ipp::{Group, Message, Value};

/// What the service answers one request with.
pub enum Answer {
    /// An IPP message, sent back under the request's own identifier.
    Ipp(Message),
    /// The same, in the chunks a long answer comes in.
    Chunked(Message),
    /// An HTTP status and nothing else.
    Http(u16),
    /// The connection closed with nothing said.
    HangUp,
}

/// A service that is running, and what it has been sent.
pub struct Serving {
    /// Where it listens.
    address: SocketAddr,
    /// Every request, in the order it arrived.
    heard: Arc<Mutex<Vec<Message>>>,
}

impl Serving {
    /// This machine's printing service, as this crate is given it.
    pub fn service(&self) -> PrintingService {
        PrintingService::at_this_machines_address(self.address).expect("loopback is this machine")
    }

    /// Every request it has been sent.
    pub fn heard(&self) -> Vec<Message> {
        self.heard.lock().expect("the list of requests").clone()
    }

    /// The operation of every request it has been sent.
    pub fn operations(&self) -> Vec<u16> {
        self.heard().iter().map(Message::code).collect()
    }
}

/// A service answering each request with what `answering` says.
pub fn a_printing_service(answering: impl Fn(&Message) -> Answer + Send + 'static) -> Serving {
    let listener = TcpListener::bind((Ipv4Addr::LOCALHOST, 0)).expect("a port on loopback");
    let address = listener.local_addr().expect("the port it has");
    let heard = Arc::new(Mutex::new(Vec::new()));
    let keeping = Arc::clone(&heard);
    thread::spawn(move || {
        for stream in listener.incoming() {
            let Ok(stream) = stream else { continue };
            serve(stream, &answering, &keeping);
        }
    });
    Serving { address, heard }
}

/// An address on this machine with nothing listening at it.
pub fn nothing_listening() -> PrintingService {
    let listener = TcpListener::bind((Ipv4Addr::LOCALHOST, 0)).expect("a port on loopback");
    let address = listener.local_addr().expect("the port it has");
    drop(listener);
    PrintingService::at_this_machines_address(address).expect("loopback is this machine")
}

/// One connection: one request read, one answer written.
fn serve(
    mut stream: TcpStream,
    answering: &impl Fn(&Message) -> Answer,
    heard: &Mutex<Vec<Message>>,
) {
    let mut bytes = Vec::new();
    let mut chunk = [0_u8; 8192];
    let head_end = loop {
        if let Some(at) = bytes.windows(4).position(|w| w == b"\r\n\r\n") {
            break at;
        }
        match stream.read(&mut chunk) {
            Ok(0) | Err(_) => return,
            Ok(read) => bytes.extend(chunk.get(..read).unwrap_or_default()),
        }
    };
    let head = String::from_utf8_lossy(bytes.get(..head_end).unwrap_or_default()).into_owned();
    let length: usize = head
        .lines()
        .find_map(|line| line.strip_prefix("Content-Length: "))
        .and_then(|length| length.trim().parse().ok())
        .unwrap_or(0);
    let mut body = bytes.get(head_end + 4..).unwrap_or_default().to_vec();
    while body.len() < length {
        match stream.read(&mut chunk) {
            Ok(0) | Err(_) => return,
            Ok(read) => body.extend(chunk.get(..read).unwrap_or_default()),
        }
    }
    let request = Message::read(&body).expect("this crate sends messages that read");
    heard
        .lock()
        .expect("the list of requests")
        .push(request.clone());

    let written = match answering(&request) {
        Answer::HangUp => return,
        Answer::Http(status) => {
            format!("HTTP/1.1 {status} No\r\nContent-Length: 0\r\n\r\n").into_bytes()
        }
        Answer::Ipp(message) => {
            let body = under(&request, message);
            let mut written =
                format!("HTTP/1.1 200 OK\r\nContent-Length: {}\r\n\r\n", body.len()).into_bytes();
            written.extend(body);
            written
        }
        Answer::Chunked(message) => {
            let body = under(&request, message);
            let mut written = b"HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\n\r\n".to_vec();
            for piece in body.chunks(7) {
                written.extend(format!("{:x}\r\n", piece.len()).into_bytes());
                written.extend(piece);
                written.extend(b"\r\n");
            }
            written.extend(b"0\r\n\r\n");
            written
        }
    };
    let _ = stream.write_all(&written);
}

/// An answer's bytes, under the identifier of the request it answers.
fn under(request: &Message, answer: Message) -> Vec<u8> {
    let mut rebuilt =
        Message::answer(answer.code(), request.request_id()).carrying(answer.data().to_vec());
    for (group, attributes) in answer.groups() {
        rebuilt = rebuilt.beginning(group);
        for attribute in attributes {
            rebuilt = rebuilt.with(group, attribute.name(), attribute.values().to_vec());
        }
    }
    rebuilt.written().expect("an answer that writes")
}

/// Success, with nothing in it.
pub fn ok() -> Message {
    Message::answer(0, 0).beginning(Group::Operation)
}

/// A status, with nothing in it.
pub fn status(code: u16) -> Message {
    Message::answer(code, 0).beginning(Group::Operation)
}

/// One device in an answer listing devices.
pub fn with_device(message: Message, uri: &str, make_and_model: &str) -> Message {
    message
        .beginning(Group::Printer)
        .with(
            Group::Printer,
            "device-class",
            vec![Value::keyword("network")],
        )
        .with(
            Group::Printer,
            "device-info",
            vec![Value::text(make_and_model)],
        )
        .with(
            Group::Printer,
            "device-make-and-model",
            vec![Value::text(make_and_model)],
        )
        .with(Group::Printer, "device-uri", vec![Value::uri(uri)])
}

/// An answer naming the printer this machine prints on.
pub fn the_default(queue: &str, uri: &str, info: &str) -> Message {
    ok().beginning(Group::Printer)
        .with(Group::Printer, "printer-name", vec![Value::name(queue)])
        .with(Group::Printer, "device-uri", vec![Value::uri(uri)])
        .with(Group::Printer, "printer-info", vec![Value::text(info)])
}

/// An answer describing how a printer is.
pub fn a_printer_that_is(state: i32, reasons: &[&str], accepting: bool) -> Message {
    ok().beginning(Group::Printer)
        .with(Group::Printer, "printer-state", vec![Value::Enum(state)])
        .with(
            Group::Printer,
            "printer-state-reasons",
            reasons
                .iter()
                .map(|reason| Value::keyword(reason))
                .collect(),
        )
        .with(
            Group::Printer,
            "printer-is-accepting-jobs",
            vec![Value::Boolean(accepting)],
        )
}

/// The operation that lists devices.
pub const GET_DEVICES: u16 = 0x400b;
/// The operation that adds a printer.
pub const ADD_MODIFY_PRINTER: u16 = 0x4003;
/// The operation that makes a printer the default.
pub const SET_DEFAULT: u16 = 0x400a;
/// The operation that asks which printer is the default.
pub const GET_DEFAULT: u16 = 0x4001;
/// The operation that asks how a printer is.
pub const GET_PRINTER_ATTRIBUTES: u16 = 0x000b;
/// The operation that prints a document.
pub const PRINT_JOB: u16 = 0x0002;
