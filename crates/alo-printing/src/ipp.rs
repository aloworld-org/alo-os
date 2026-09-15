//! The messages this machine's printing service reads and writes: IPP, as
//! RFC 8010 encodes it.
//!
//! **Rented protocol, our own few hundred lines.** The printing service is
//! CUPS, pinned and unpatched (ADR 0011), and the interface it offers a program
//! that wants to add a printer, ask how one is, or hand it a document is this
//! encoding carried over HTTP. Speaking it here means nothing in this crate
//! starts a program — no administration tool, no print command — and that every byte sent to the
//! service is one somebody can read in this file.
//!
//! A message is a version, an operation (in a request) or a status (in a
//! response), a request identifier, groups of attributes, and then whatever
//! document follows. An attribute is a name and one or more values, each value
//! tagged with its type.
//!
//! **Reading is defensive and deliberately incomplete.** A value of a type this
//! crate never reads — a date, a resolution, a collection's members — is kept
//! as bytes under its tag rather than interpreted, so a service that answers
//! with more than was asked for is not a service this crate fails to read. What
//! is refused is a message that does not hold together: cut short, a value
//! with no attribute to belong to, or more attributes than any answer here
//! could need.
//!
//! Nothing here has a `Display`. A person never reads this format, and what a
//! malformed answer means to them is `crate::Tried`'s sentence.

/// A request and a response carry the version of the protocol first.
pub const VERSION: [u8; 2] = [2, 0];

/// The value tag for a signed 32-bit integer.
pub const INTEGER: u8 = 0x21;
/// The value tag for a boolean.
pub const BOOLEAN: u8 = 0x22;
/// The value tag for an enumerated value.
pub const ENUM: u8 = 0x23;
/// The value tag for text with its language beside it.
pub const TEXT_WITH_LANGUAGE: u8 = 0x35;
/// The value tag for a name with its language beside it.
pub const NAME_WITH_LANGUAGE: u8 = 0x36;
/// The value tag for text.
pub const TEXT: u8 = 0x41;
/// The value tag for a name.
pub const NAME: u8 = 0x42;
/// The value tag for a keyword.
pub const KEYWORD: u8 = 0x44;
/// The value tag for a URI.
pub const URI: u8 = 0x45;
/// The value tag for a character set.
pub const CHARSET: u8 = 0x47;
/// The value tag for a natural language.
pub const LANGUAGE: u8 = 0x48;
/// The value tag for a media type.
pub const MIME: u8 = 0x49;
/// The value tag meaning an attribute has no value.
pub const NO_VALUE: u8 = 0x13;

/// The tag that ends the attributes, after which the document begins.
const END_OF_ATTRIBUTES: u8 = 0x03;

/// The most attributes one message is read with.
///
/// The largest answer this crate asks for is a list of devices, a handful of
/// attributes each; a service answering with more than this is not answering a
/// question this crate asked.
const MOST_ATTRIBUTES: usize = 20_000;

/// Which group of a message an attribute is in.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Group {
    /// About the operation itself.
    Operation,
    /// About a job.
    Job,
    /// About a printer — or, in a list of devices, one device.
    Printer,
    /// Attributes the other side did not support.
    Unsupported,
    /// A group this crate never reads, by its tag.
    Other(u8),
}

impl Group {
    /// The tag this group is written with.
    #[must_use]
    pub fn tag(self) -> u8 {
        match self {
            Self::Operation => 0x01,
            Self::Job => 0x02,
            Self::Printer => 0x04,
            Self::Unsupported => 0x05,
            Self::Other(tag) => tag,
        }
    }

    /// The group written with this tag.
    fn of_tag(tag: u8) -> Self {
        match tag {
            0x01 => Self::Operation,
            0x02 => Self::Job,
            0x04 => Self::Printer,
            0x05 => Self::Unsupported,
            other => Self::Other(other),
        }
    }
}

/// One value of an attribute.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Value {
    /// A signed integer.
    Integer(i32),
    /// A boolean.
    Boolean(bool),
    /// An enumerated value.
    Enum(i32),
    /// Any of the string types, by its tag: text, a name, a keyword, a URI, a
    /// character set, a language or a media type.
    Text {
        /// Which string type it is.
        tag: u8,
        /// The string.
        text: String,
    },
    /// A value of a type this crate does not interpret, as it arrived.
    Other {
        /// Its tag.
        tag: u8,
        /// Its bytes.
        bytes: Vec<u8>,
    },
}

impl Value {
    /// A keyword.
    #[must_use]
    pub fn keyword(text: &str) -> Self {
        Self::string(KEYWORD, text)
    }

    /// A URI.
    #[must_use]
    pub fn uri(text: &str) -> Self {
        Self::string(URI, text)
    }

    /// A name.
    #[must_use]
    pub fn name(text: &str) -> Self {
        Self::string(NAME, text)
    }

    /// Text.
    #[must_use]
    pub fn text(text: &str) -> Self {
        Self::string(TEXT, text)
    }

    /// A media type.
    #[must_use]
    pub fn mime(text: &str) -> Self {
        Self::string(MIME, text)
    }

    /// A string of this type.
    #[must_use]
    pub fn string(tag: u8, text: &str) -> Self {
        Self::Text {
            tag,
            text: text.to_owned(),
        }
    }

    /// The string, if this is one.
    #[must_use]
    pub fn as_text(&self) -> Option<&str> {
        match self {
            Self::Text { text, .. } => Some(text),
            _ => None,
        }
    }

    /// The number, if this is an integer or an enumerated value.
    #[must_use]
    pub fn as_integer(&self) -> Option<i32> {
        match self {
            Self::Integer(number) | Self::Enum(number) => Some(*number),
            _ => None,
        }
    }

    /// The tag this value is written with.
    fn tag(&self) -> u8 {
        match self {
            Self::Integer(_) => INTEGER,
            Self::Boolean(_) => BOOLEAN,
            Self::Enum(_) => ENUM,
            Self::Text { tag, .. } | Self::Other { tag, .. } => *tag,
        }
    }

    /// The bytes this value is written as.
    fn bytes(&self) -> Vec<u8> {
        match self {
            Self::Integer(number) | Self::Enum(number) => number.to_be_bytes().to_vec(),
            Self::Boolean(yes) => vec![u8::from(*yes)],
            Self::Text { tag, text }
                if *tag == TEXT_WITH_LANGUAGE || *tag == NAME_WITH_LANGUAGE =>
            {
                let mut bytes = vec![0, 0];
                bytes.extend(u16::try_from(text.len()).unwrap_or(u16::MAX).to_be_bytes());
                bytes.extend(text.as_bytes());
                bytes
            }
            Self::Text { text, .. } => text.as_bytes().to_vec(),
            Self::Other { bytes, .. } => bytes.clone(),
        }
    }

    /// The value these bytes are, under this tag.
    fn read(tag: u8, bytes: &[u8]) -> Self {
        let as_is = || Self::Other {
            tag,
            bytes: bytes.to_vec(),
        };
        match tag {
            INTEGER | ENUM => match <[u8; 4]>::try_from(bytes) {
                Ok(four) if tag == INTEGER => Self::Integer(i32::from_be_bytes(four)),
                Ok(four) => Self::Enum(i32::from_be_bytes(four)),
                Err(_) => as_is(),
            },
            BOOLEAN => match bytes {
                [one] => Self::Boolean(*one != 0),
                _ => as_is(),
            },
            TEXT_WITH_LANGUAGE | NAME_WITH_LANGUAGE => {
                with_language(bytes).map_or_else(as_is, |text| Self::Text { tag, text })
            }
            0x40..=0x49 => match std::str::from_utf8(bytes) {
                Ok(text) => Self::Text {
                    tag,
                    text: text.to_owned(),
                },
                Err(_) => as_is(),
            },
            _ => as_is(),
        }
    }
}

/// The text inside a value written with its language beside it.
fn with_language(bytes: &[u8]) -> Option<String> {
    let mut reading = Reader { bytes, at: 0 };
    let language = usize::from(reading.u16().ok()?);
    reading.take(language).ok()?;
    let length = usize::from(reading.u16().ok()?);
    let text = reading.take(length).ok()?;
    std::str::from_utf8(text).ok().map(str::to_owned)
}

/// One attribute: a name and its values.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Attribute {
    /// Its name.
    name: String,
    /// Its values, at least one.
    values: Vec<Value>,
}

impl Attribute {
    /// Its name.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Its values.
    #[must_use]
    pub fn values(&self) -> &[Value] {
        &self.values
    }

    /// Every value that is a string.
    pub fn texts(&self) -> impl Iterator<Item = &str> {
        self.values.iter().filter_map(Value::as_text)
    }

    /// The first value, if it is a string.
    #[must_use]
    pub fn text(&self) -> Option<&str> {
        self.values.first().and_then(Value::as_text)
    }

    /// The first value, if it is a number.
    #[must_use]
    pub fn integer(&self) -> Option<i32> {
        self.values.first().and_then(Value::as_integer)
    }

    /// The first value, if it is a boolean.
    #[must_use]
    pub fn boolean(&self) -> Option<bool> {
        match self.values.first() {
            Some(Value::Boolean(yes)) => Some(*yes),
            _ => None,
        }
    }
}

/// Why a message could not be read.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Malformed {
    /// It ended part way through something, or before its attributes did.
    CutShort,
    /// A version this crate does not speak.
    AnotherVersion,
    /// An attribute before any group had begun.
    OutsideAGroup,
    /// A second value before any attribute it could belong to.
    AValueWithNoAttribute,
    /// More attributes than any answer this crate asks for.
    TooMany,
}

/// Why a message could not be written.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Unwritable {
    /// A name or a value longer than the encoding can say.
    TooLong,
}

/// One message: a request, or the response to one.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Message {
    /// The version it was written in.
    version: [u8; 2],
    /// The operation, in a request; the status, in a response.
    code: u16,
    /// Which request this is, or answers.
    request_id: u32,
    /// The groups, in order.
    groups: Vec<(Group, Vec<Attribute>)>,
    /// What follows the attributes.
    data: Vec<u8>,
}

impl Message {
    /// A request for this operation, carrying the two attributes every request
    /// begins with.
    ///
    /// The natural language named here is the language of the *attributes*
    /// this crate writes, which are keywords and addresses — never anything a
    /// person reads, which is always a sentence from `crate::words`.
    #[must_use]
    pub fn request(operation: u16, request_id: u32) -> Self {
        Self::answer(operation, request_id)
            .with(
                Group::Operation,
                "attributes-charset",
                vec![Value::string(CHARSET, "utf-8")],
            )
            .with(
                Group::Operation,
                "attributes-natural-language",
                vec![Value::string(LANGUAGE, "en")],
            )
    }

    /// A message with this code and nothing in it yet — how a response is
    /// begun, and what a request is built on.
    #[must_use]
    pub fn answer(code: u16, request_id: u32) -> Self {
        Self {
            version: VERSION,
            code,
            request_id,
            groups: Vec::new(),
            data: Vec::new(),
        }
    }

    /// The same message with this attribute added to the last group, when
    /// that group is of this kind, or to a new group of this kind.
    #[must_use]
    pub fn with(mut self, group: Group, name: &str, values: Vec<Value>) -> Self {
        let attribute = Attribute {
            name: name.to_owned(),
            values,
        };
        match self.groups.last_mut() {
            Some((last, attributes)) if *last == group => attributes.push(attribute),
            _ => self.groups.push((group, vec![attribute])),
        }
        self
    }

    /// The same message with a new, empty group of this kind begun — how two
    /// devices in one answer are told apart.
    #[must_use]
    pub fn beginning(mut self, group: Group) -> Self {
        self.groups.push((group, Vec::new()));
        self
    }

    /// The same message with a document after its attributes.
    #[must_use]
    pub fn carrying(mut self, data: Vec<u8>) -> Self {
        self.data = data;
        self
    }

    /// The operation, in a request; the status, in a response.
    #[must_use]
    pub fn code(&self) -> u16 {
        self.code
    }

    /// Whether this response says the request succeeded.
    #[must_use]
    pub fn succeeded(&self) -> bool {
        self.code < 0x0100
    }

    /// Which request this is, or answers.
    #[must_use]
    pub fn request_id(&self) -> u32 {
        self.request_id
    }

    /// What follows the attributes.
    #[must_use]
    pub fn data(&self) -> &[u8] {
        &self.data
    }

    /// Every group, in order.
    pub fn groups(&self) -> impl Iterator<Item = (Group, &[Attribute])> {
        self.groups
            .iter()
            .map(|(group, attributes)| (*group, attributes.as_slice()))
    }

    /// The first attribute of this name in any group of this kind.
    #[must_use]
    pub fn attribute(&self, group: Group, name: &str) -> Option<&Attribute> {
        self.groups()
            .filter(|(kind, _)| *kind == group)
            .flat_map(|(_, attributes)| attributes.iter())
            .find(|attribute| attribute.name == name)
    }

    /// The message's attributes as bytes, without the document.
    ///
    /// # Errors
    /// [`Unwritable::TooLong`] for a name or value the encoding cannot carry.
    pub fn head(&self) -> Result<Vec<u8>, Unwritable> {
        let mut bytes = Vec::with_capacity(256);
        bytes.extend(self.version);
        bytes.extend(self.code.to_be_bytes());
        bytes.extend(self.request_id.to_be_bytes());
        for (group, attributes) in &self.groups {
            bytes.push(group.tag());
            for attribute in attributes {
                write_attribute(&mut bytes, attribute)?;
            }
        }
        bytes.push(END_OF_ATTRIBUTES);
        Ok(bytes)
    }

    /// The whole message as bytes: its attributes, then its document.
    ///
    /// # Errors
    /// [`Unwritable::TooLong`] for a name or value the encoding cannot carry.
    pub fn written(&self) -> Result<Vec<u8>, Unwritable> {
        let mut bytes = self.head()?;
        bytes.extend(&self.data);
        Ok(bytes)
    }

    /// The message these bytes are.
    ///
    /// # Errors
    /// [`Malformed`] for bytes that do not hold together as a message.
    pub fn read(bytes: &[u8]) -> Result<Self, Malformed> {
        let mut reading = Reader { bytes, at: 0 };
        let version = [reading.u8()?, reading.u8()?];
        if !matches!(version, [1 | 2, _]) {
            return Err(Malformed::AnotherVersion);
        }
        let code = reading.u16()?;
        let request_id = reading.u32()?;
        let mut message = Self {
            version,
            code,
            request_id,
            groups: Vec::new(),
            data: Vec::new(),
        };
        let mut counted = 0_usize;
        loop {
            let tag = reading.u8()?;
            if tag == END_OF_ATTRIBUTES {
                message.data = reading.rest().to_vec();
                return Ok(message);
            }
            if tag < 0x10 {
                message.groups.push((Group::of_tag(tag), Vec::new()));
                continue;
            }
            if tag == 0x7f {
                // An extended tag names its real type in the next four bytes;
                // nothing here reads one, so it is kept under the tag it came
                // with.
                reading.u32()?;
            }
            let name_length = usize::from(reading.u16()?);
            let name = reading.take(name_length)?;
            let value_length = usize::from(reading.u16()?);
            let value = Value::read(tag, reading.take(value_length)?);
            counted += 1;
            if counted > MOST_ATTRIBUTES {
                return Err(Malformed::TooMany);
            }
            let Some((_, attributes)) = message.groups.last_mut() else {
                return Err(Malformed::OutsideAGroup);
            };
            if name.is_empty() {
                let Some(last) = attributes.last_mut() else {
                    return Err(Malformed::AValueWithNoAttribute);
                };
                last.values.push(value);
            } else {
                attributes.push(Attribute {
                    name: String::from_utf8_lossy(name).into_owned(),
                    values: vec![value],
                });
            }
        }
    }
}

/// One attribute, every value after the first written with no name.
fn write_attribute(bytes: &mut Vec<u8>, attribute: &Attribute) -> Result<(), Unwritable> {
    let name = u16::try_from(attribute.name.len()).map_err(|_| Unwritable::TooLong)?;
    if attribute.values.is_empty() {
        bytes.push(NO_VALUE);
        bytes.extend(name.to_be_bytes());
        bytes.extend(attribute.name.as_bytes());
        bytes.extend(0_u16.to_be_bytes());
        return Ok(());
    }
    for (index, value) in attribute.values.iter().enumerate() {
        let written = value.bytes();
        let length = u16::try_from(written.len()).map_err(|_| Unwritable::TooLong)?;
        bytes.push(value.tag());
        if index == 0 {
            bytes.extend(name.to_be_bytes());
            bytes.extend(attribute.name.as_bytes());
        } else {
            bytes.extend(0_u16.to_be_bytes());
        }
        bytes.extend(length.to_be_bytes());
        bytes.extend(written);
    }
    Ok(())
}

/// Bytes, read from the front.
struct Reader<'a> {
    /// All of them.
    bytes: &'a [u8],
    /// How many have been read.
    at: usize,
}

impl<'a> Reader<'a> {
    /// The next `count` bytes.
    fn take(&mut self, count: usize) -> Result<&'a [u8], Malformed> {
        let end = self.at.checked_add(count).ok_or(Malformed::CutShort)?;
        let taken = self.bytes.get(self.at..end).ok_or(Malformed::CutShort)?;
        self.at = end;
        Ok(taken)
    }

    /// The next byte.
    fn u8(&mut self) -> Result<u8, Malformed> {
        self.take(1)?.first().copied().ok_or(Malformed::CutShort)
    }

    /// The next two bytes, as a number.
    fn u16(&mut self) -> Result<u16, Malformed> {
        let two = <[u8; 2]>::try_from(self.take(2)?).map_err(|_| Malformed::CutShort)?;
        Ok(u16::from_be_bytes(two))
    }

    /// The next four bytes, as a number.
    fn u32(&mut self) -> Result<u32, Malformed> {
        let four = <[u8; 4]>::try_from(self.take(4)?).map_err(|_| Malformed::CutShort)?;
        Ok(u32::from_be_bytes(four))
    }

    /// Everything not yet read.
    fn rest(&self) -> &'a [u8] {
        self.bytes.get(self.at..).unwrap_or_default()
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;

    /// A request written and read back is the request that was written.
    #[test]
    fn a_message_survives_being_written_and_read() {
        let message = Message::request(0x000b, 7)
            .with(
                Group::Operation,
                "printer-uri",
                vec![Value::uri("ipp://localhost/printers/a")],
            )
            .with(
                Group::Operation,
                "requested-attributes",
                vec![
                    Value::keyword("printer-state"),
                    Value::keyword("printer-state-reasons"),
                ],
            )
            .with(
                Group::Printer,
                "printer-is-accepting-jobs",
                vec![Value::Boolean(true)],
            )
            .with(Group::Printer, "printer-state", vec![Value::Enum(3)])
            .carrying(b"%PDF-1.7".to_vec());
        let read = Message::read(&message.written().unwrap()).unwrap();
        assert_eq!(read, message);
        assert_eq!(
            read.attribute(Group::Operation, "requested-attributes")
                .unwrap()
                .texts()
                .collect::<Vec<_>>(),
            ["printer-state", "printer-state-reasons"]
        );
        assert_eq!(read.data(), b"%PDF-1.7");
    }

    /// The bytes are the ones RFC 8010 says, checked against its own example
    /// of a request's first attribute.
    #[test]
    fn the_bytes_are_the_ones_the_specification_gives() {
        let written = Message::request(0x0002, 1).written().unwrap();
        assert_eq!(
            written.get(..9).unwrap(),
            [0x02, 0x00, 0x00, 0x02, 0x00, 0x00, 0x00, 0x01, 0x01]
        );
        let charset = [
            0x47, 0x00, 0x12, b'a', b't', b't', b'r', b'i', b'b', b'u', b't', b'e', b's', b'-',
            b'c', b'h', b'a', b'r', b's', b'e', b't', 0x00, 0x05, b'u', b't', b'f', b'-', b'8',
        ];
        assert_eq!(written.get(9..9 + charset.len()).unwrap(), charset);
        assert_eq!(written.last(), Some(&0x03));
    }

    /// A message cut anywhere short of its end is refused, never read as the
    /// part that arrived.
    #[test]
    fn a_message_cut_short_is_refused_wherever_it_is_cut() {
        let written = Message::request(0x000b, 9)
            .with(Group::Printer, "printer-state", vec![Value::Enum(5)])
            .written()
            .unwrap();
        for length in 0..written.len() {
            assert_eq!(
                Message::read(written.get(..length).unwrap()),
                Err(Malformed::CutShort),
                "{length}"
            );
        }
    }

    /// Nonsense in the places structure lives is refused by name.
    #[test]
    fn a_message_that_does_not_hold_together_is_refused() {
        assert_eq!(
            Message::read(&[9, 0, 0, 0, 0, 0, 0, 1, 3]),
            Err(Malformed::AnotherVersion)
        );
        assert_eq!(
            Message::read(&[2, 0, 0, 0, 0, 0, 0, 1, 0x44, 0, 1, b'a', 0, 0, 3]),
            Err(Malformed::OutsideAGroup)
        );
        assert_eq!(
            Message::read(&[2, 0, 0, 0, 0, 0, 0, 1, 1, 0x44, 0, 0, 0, 0, 3]),
            Err(Malformed::AValueWithNoAttribute)
        );
    }

    /// A value of a type nothing here reads is kept as it came, so an answer
    /// with more in it than was asked for still reads.
    #[test]
    fn a_value_of_a_type_nothing_here_reads_is_kept_as_it_came() {
        let bytes = [
            2, 0, 0, 0, 0, 0, 0, 1, 4, 0x31, 0, 1, b'd', 0, 3, 1, 2, 3, 0x35, 0, 1, b't', 0, 8, 0,
            2, b'e', b'n', 0, 2, b'h', b'i', 3,
        ];
        let read = Message::read(&bytes).unwrap();
        assert_eq!(
            read.attribute(Group::Printer, "d").unwrap().values(),
            [Value::Other {
                tag: 0x31,
                bytes: vec![1, 2, 3]
            }]
        );
        assert_eq!(
            read.attribute(Group::Printer, "t").unwrap().text(),
            Some("hi")
        );
    }
}
